// -- https://github.com/StormWorld0/smflog
// -- https://pypi.org/project/smflog
// -- GPLv2 License
// -- Author: zxelzy
use dirs;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::fs::DirBuilderExt;
use crate::errors::PrintResult; 

pub fn log_dir(app_name: &str) -> PrintResult<PathBuf> {
    let mut base_path = if cfg!(target_os = "macos") {
        // MacOS: ~/Library/Logs
        dirs::home_dir()
            .map(|h| h.join("Library").join("Logs"))
            .ok_or_else(|| Error::new(ErrorKind::NotFound, "macOS environment: HOME dir not found"))?
    } else if cfg!(target_os = "windows") {
        // Windows: C:\Users\<User>\AppData\Local\<app_name>\Logs
        dirs::data_local_dir()
            .ok_or_else(|| Error::new(ErrorKind::NotFound, "Windows environment: LOCALAPPDATA not found"))?;
    } else {
        // Linux/Unix: XDG_STATE_HOME fallback ke ~/.local/share
        dirs::state_dir()
            .or_else(|| dirs::data_local_dir())
            .ok_or_else(|| Error::new(ErrorKind::NotFound, "Unix environment: XDG_STATE_HOME not found"))?
    };

    base_path.push(app_name);
    setup_dir(base_path)
}

fn setup_dir(path: PathBuf) -> PrintResult<PathBuf> {
    if !path.exists() {
        let mut builder = fs::DirBuilder::new();
        builder.recursive(true);

        #[cfg(unix)]
        builder.mode(0o700);
        builder.create(&path)?; 
    }
    
    Ok(path)
}
