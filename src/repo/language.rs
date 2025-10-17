use std::{fs, path::PathBuf};

pub struct LanguageAnalyzer {
    repo_path: String,
}

impl LanguageAnalyzer {
    pub fn new<S: Into<String>>(repo_path: S) -> Self {
        Self {
            repo_path: repo_path.into(),
        }
    }

    fn resolve_git_ignore(&self) -> anyhow::Result<Vec<String>> {
        let repo_path = PathBuf::from(&self.repo_path);
        let git_ignore_path = repo_path.join(".gitignore");
        if !git_ignore_path.exists() {
            return Ok(Vec::new());
        }
        let git_ignore_content = fs::read_to_string(git_ignore_path)?;
        Ok(git_ignore_content
            .lines()
            .map(|line| line.to_string())
            .collect())
    }

    pub fn language(&self) -> anyhow::Result<(String, u32)> {
        let git_ignore = self.resolve_git_ignore()?;

        let config = tokei::Config::default();
        let mut lang = tokei::Languages::new();

        lang.get_statistics(
            &[self.repo_path.as_str()],
            &git_ignore.iter().map(String::as_str).collect::<Vec<&str>>(),
            &config,
        );

        let max_lang = lang
            .iter()
            .max_by_key(|(_, language)| language.code)
            .map(|(lang_type, language)| (lang_type.to_string(), language.code))
            .unwrap_or(("unknown".to_string(), 0));

        Ok((max_lang.0.to_string(), max_lang.1 as u32))
    }
}

pub fn get_language_icon(language: &str) -> String {
    let language = language.to_lowercase();
    match language.as_str() {
        "javascript" => "javascript".to_string(),
        "python" => "python".to_string(),
        "rust" => "rust".to_string(),
        "java" => "java".to_string(),
        "c++" => "c++".to_string(),
        "c#" => "c#".to_string(),
        "c" => "c".to_string(),
        "dart" => "dart".to_string(),
        "go" => "go".to_string(),
        "php" => "php".to_string(),
        "zig" => "zig".to_string(),
        "swift" => "swift".to_string(),
        "typescript" => "typescript".to_string(),
        "kotlin" => "kotlin".to_string(),
        "json" => "json".to_string(),
        "sass" => "sass".to_string(),
        "markdown" => "markdown".to_string(),
        "objective-c" => "oc".to_string(),
        "css" => "css".to_string(),

        _ => "default".to_string(),
    }
}
