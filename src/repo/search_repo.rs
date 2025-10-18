use futures::stream::{self, StreamExt};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::sync::{Semaphore, mpsc};

use crate::config::GitLauncherConfig;

#[derive(Debug, Clone)]
pub struct GitProject {
    pub full_path: PathBuf,
    pub folder_name: String,
}

#[derive(Debug, Clone)]
pub struct GitFinderConfig {
    pub ignored_dirs: HashSet<String>,
    pub max_depth: Option<usize>,
    pub max_concurrent_tasks: usize,
}

impl GitFinderConfig {
    pub fn new(config: GitLauncherConfig) -> Self {
        Self {
            ignored_dirs: config.ignore_dirs.into_iter().collect(),
            max_depth: Some(config.max_depth),
            max_concurrent_tasks: config.max_concurrent_tasks,
        }
    }
}

pub struct GitProjectFinder {
    config: Arc<GitFinderConfig>,
    semaphore: Arc<Semaphore>,
}

impl GitProjectFinder {
    pub fn new(config: GitFinderConfig) -> Self {
        let max_permits = config.max_concurrent_tasks;
        Self {
            config: Arc::new(config),
            semaphore: Arc::new(Semaphore::new(max_permits)),
        }
    }

    /// 查找指定目录下的所有Git项目
    pub async fn find_git_projects(
        &self,
        root_path: impl AsRef<Path>,
    ) -> anyhow::Result<Vec<GitProject>, anyhow::Error> {
        let root = root_path.as_ref().to_path_buf();

        // 使用通道收集结果
        let (tx, mut rx) = mpsc::unbounded_channel::<GitProject>();

        // 启动扫描任务
        let scan_handle = {
            let tx = tx.clone();
            let finder = Self {
                config: Arc::clone(&self.config),
                semaphore: Arc::clone(&self.semaphore),
            };

            tokio::spawn(async move { finder.scan_directory_with_channel(root, 0, tx).await })
        };

        // 关闭发送端，这样接收端就知道何时停止
        drop(tx);

        // 收集所有结果
        let mut projects = Vec::new();
        while let Some(project) = rx.recv().await {
            projects.push(project);
        }

        // 等待扫描完成
        scan_handle.await??;

        // 过滤掉作为子模块的项目
        let filtered_projects = self.filter_submodules(projects).await?;

        Ok(filtered_projects)
    }

    /// 使用通道的递归扫描
    async fn scan_directory_with_channel(
        &self,
        dir_path: PathBuf,
        current_depth: usize,
        tx: mpsc::UnboundedSender<GitProject>,
    ) -> anyhow::Result<(), anyhow::Error> {
        // 检查深度限制
        if let Some(max_depth) = self.config.max_depth {
            if current_depth > max_depth {
                return Ok(());
            }
        }

        // 获取信号量许可，限制并发数
        let _permit = self.semaphore.acquire().await?;

        let mut entries = match fs::read_dir(&dir_path).await {
            Ok(entries) => entries,
            Err(_) => return Ok(()), // 忽略无法读取的目录
        };

        let mut subdirs = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let entry_path = entry.path();

            if !entry_path.is_dir() {
                continue;
            }

            let dir_name = match entry_path.file_name() {
                Some(name) => name.to_string_lossy().to_string(),
                None => continue,
            };

            // 检查是否为忽略的目录
            if self.config.ignored_dirs.contains(&dir_name) {
                continue;
            }

            // 检查是否为Git项目
            if self.is_git_repository(&entry_path).await {
                let project = GitProject {
                    full_path: entry_path.clone(),
                    folder_name: dir_name,
                };

                // 发送到通道，如果发送失败说明接收端已关闭
                if tx.send(project).is_err() {
                    break;
                }
            } else {
                // 收集子目录用于后续处理
                subdirs.push(entry_path);
            }
        }

        // 释放当前许可，然后处理子目录
        drop(_permit);

        // 使用 futures stream 来并发处理子目录，但仍然控制并发数
        let futures = subdirs.into_iter().map(|subdir| {
            let tx = tx.clone();
            let finder = Self {
                config: Arc::clone(&self.config),
                semaphore: Arc::clone(&self.semaphore),
            };

            async move {
                finder
                    .scan_directory_with_channel(subdir, current_depth + 1, tx)
                    .await
            }
        });

        // 并发执行但限制数量
        let results: Vec<_> = stream::iter(futures)
            .buffer_unordered(self.config.max_concurrent_tasks)
            .collect()
            .await;

        // 检查是否有错误（可选择如何处理错误）
        for result in results {
            if let Err(e) = result {
                eprintln!("扫描子目录时出错: {}", e);
            }
        }

        Ok(())
    }

    /// 检查目录是否为Git仓库
    async fn is_git_repository(&self, path: &Path) -> bool {
        let git_dir = path.join(".git");
        match fs::metadata(&git_dir).await {
            Ok(metadata) => metadata.is_dir() || metadata.is_file(),
            Err(_) => false,
        }
    }

    /// 过滤掉作为子模块的项目
    async fn filter_submodules(
        &self,
        projects: Vec<GitProject>,
    ) -> anyhow::Result<Vec<GitProject>, anyhow::Error> {
        // 使用 stream 并发检查所有项目
        let futures = projects.into_iter().map(|project| async move {
            if self.is_submodule(&project.full_path).await {
                None
            } else {
                Some(project)
            }
        });

        let results: Vec<Option<GitProject>> = stream::iter(futures)
            .buffer_unordered(self.config.max_concurrent_tasks)
            .collect()
            .await;

        Ok(results.into_iter().flatten().collect())
    }

    /// 检查Git项目是否为子模块
    async fn is_submodule(&self, project_path: &Path) -> bool {
        let git_path = project_path.join(".git");

        match fs::metadata(&git_path).await {
            Ok(metadata) => {
                if metadata.is_file() {
                    if let Ok(content) = fs::read_to_string(&git_path).await {
                        content.trim().starts_with("gitdir:")
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            Err(_) => false,
        }
    }
}

impl GitProjectFinder {
    pub fn builder(config: GitLauncherConfig) -> GitFinderConfigBuilder {
        GitFinderConfigBuilder::new(config)
    }
}

pub struct GitFinderConfigBuilder {
    ignored_dirs: HashSet<String>,
    max_depth: Option<usize>,
    max_concurrent_tasks: usize,
}

impl GitFinderConfigBuilder {
    pub fn new(config: GitLauncherConfig) -> Self {
        Self {
            ignored_dirs: config.ignore_dirs.into_iter().collect(),
            max_depth: Some(config.max_depth),
            max_concurrent_tasks: config.max_concurrent_tasks,
        }
    }

    pub fn ignore_dir<S: Into<String>>(mut self, dir: S) -> Self {
        self.ignored_dirs.insert(dir.into());
        self
    }

    pub fn ignore_dirs<I, S>(mut self, dirs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for dir in dirs {
            self.ignored_dirs.insert(dir.into());
        }
        self
    }

    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = Some(depth);
        self
    }

    pub fn unlimited_depth(mut self) -> Self {
        self.max_depth = None;
        self
    }

    pub fn max_concurrent_tasks(mut self, count: usize) -> Self {
        self.max_concurrent_tasks = count;
        self
    }

    pub fn build(self) -> GitProjectFinder {
        let config = GitFinderConfig {
            ignored_dirs: self.ignored_dirs,
            max_depth: self.max_depth,
            max_concurrent_tasks: self.max_concurrent_tasks,
        };
        GitProjectFinder::new(config)
    }
}
