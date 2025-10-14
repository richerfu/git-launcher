use std::io;
use std::path::Path;
use std::process::Command;

pub struct FileOpener;

impl FileOpener {
    /// 使用指定程序打开文件
    pub fn open_with(program: &str, path: &str) -> io::Result<()> {
        let path_obj = Path::new(path);

        if !path_obj.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("文件不存在: {}", path),
            ));
        }

        #[cfg(target_os = "macos")]
        let status = Command::new("open")
            .arg("-a")
            .arg(program)
            .arg(path)
            .status()?;

        if !status.success() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("程序 {} 执行失败", program),
            ));
        }

        Ok(())
    }
}
