use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use chrono::Utc;
use log::{debug, info, warn, error, LevelFilter};
use fern::Dispatch;

use crate::config::{LoggingConfig, ConfigManager};

pub struct Logger {
    log_dir: PathBuf,
    max_size_mb: u64,
    current_log_file: Option<PathBuf>,
}

impl Logger {
    pub fn new(config: &LoggingConfig) -> Result<Self> {
        let log_dir = PathBuf::from(&config.log_dir);

        // Create log directory if it doesn't exist
        if !log_dir.exists() {
            fs::create_dir_all(&log_dir)
                .with_context(|| format!("Failed to create log directory: {:?}", log_dir))?;
        }

        let mut logger = Logger {
            log_dir,
            max_size_mb: config.max_log_size_mb,
            current_log_file: None,
        };

        // Clean up old logs if needed
        if config.file_rotation {
            logger.cleanup_old_logs()?;
        }

        Ok(logger)
    }

    pub fn setup_logging(config: &LoggingConfig) -> Result<()> {
        let log_level = Self::parse_log_level(&config.level)?;

        let mut dispatch = Dispatch::new()
            .level(log_level)
            .format(|out, message, record| {
                out.finish(format_args!(
                    "[{}] [{}] [{}] [{}] {}",
                    Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                    record.level(),
                    record.target(),
                    std::thread::current().name().unwrap_or("main"),
                    message
                ))
            });

        // Add console output if enabled
        if config.console {
            dispatch = dispatch.chain(std::io::stdout());
        }

        // Add file output if file rotation is enabled
        if config.file_rotation {
            let log_file_path = Self::create_log_file_path(&config.log_dir)?;
            dispatch = dispatch.chain(
                fern::log_file(&log_file_path)
                    .with_context(|| format!("Failed to create log file: {:?}", log_file_path))?
            );

            info!("Logging to file: {:?}", log_file_path);
        }

        dispatch.apply()
            .with_context(|| "Failed to initialize logger")?;

        info!("Logger initialized with level: {}", config.level);
        Ok(())
    }

    fn parse_log_level(level: &str) -> Result<LevelFilter> {
        match level.to_lowercase().as_str() {
            "error" => Ok(LevelFilter::Error),
            "warn" | "warning" => Ok(LevelFilter::Warn),
            "info" => Ok(LevelFilter::Info),
            "debug" => Ok(LevelFilter::Debug),
            "trace" => Ok(LevelFilter::Trace),
            _ => Err(anyhow::anyhow!("Invalid log level: {}", level)),
        }
    }

    fn create_log_file_path(log_dir: &str) -> Result<PathBuf> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let filename = format!("regis_{}.log", timestamp);
        let log_dir_path = PathBuf::from(log_dir);

        // Ensure log directory exists
        if !log_dir_path.exists() {
            fs::create_dir_all(&log_dir_path)
                .with_context(|| format!("Failed to create log directory: {:?}", log_dir_path))?;
        }

        Ok(log_dir_path.join(filename))
    }

    pub fn cleanup_old_logs(&self) -> Result<()> {
        debug!("Cleaning up old log files in {:?}", self.log_dir);

        if !self.log_dir.exists() {
            return Ok(());
        }

        // Get all log files sorted by modification time (newest first)
        let mut log_files = Vec::new();
        for entry in fs::read_dir(&self.log_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() &&
               path.extension().and_then(|s| s.to_str()) == Some("log") &&
               path.file_name().and_then(|s| s.to_str()).unwrap_or("").starts_with("regis_") {

                let metadata = fs::metadata(&path)?;
                let size = metadata.len();
                let modified = metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);

                log_files.push((path, size, modified));
            }
        }

        // Sort by modification time (newest first)
        log_files.sort_by(|a, b| b.2.cmp(&a.2));

        // Calculate total size and remove old files if over limit
        let max_size_bytes = self.max_size_mb * 1024 * 1024;
        let mut current_size: u64 = 0;
        let mut files_to_remove = Vec::new();

        for (path, size, _) in log_files {
            if current_size + size > max_size_bytes {
                files_to_remove.push(path);
            } else {
                current_size += size;
            }
        }

        // Remove excess files
        for path in files_to_remove {
            match fs::remove_file(&path) {
                Ok(_) => info!("Removed old log file: {:?}", path),
                Err(e) => warn!("Failed to remove old log file {:?}: {}", path, e),
            }
        }

        if current_size > 0 {
            debug!("Log directory size after cleanup: {:.2} MB", current_size as f64 / (1024.0 * 1024.0));
        }

        Ok(())
    }

    pub fn get_log_files(&self) -> Result<Vec<PathBuf>> {
        let mut log_files = Vec::new();

        if !self.log_dir.exists() {
            return Ok(log_files);
        }

        for entry in fs::read_dir(&self.log_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() &&
               path.extension().and_then(|s| s.to_str()) == Some("log") &&
               path.file_name().and_then(|s| s.to_str()).unwrap_or("").starts_with("regis_") {
                log_files.push(path);
            }
        }

        // Sort by modification time (newest first)
        log_files.sort_by(|a, b| {
            let a_modified = fs::metadata(a).and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            let b_modified = fs::metadata(b).and_then(|m| m.modified()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            b_modified.cmp(&a_modified)
        });

        Ok(log_files)
    }
}

// Tauri commands for frontend logging
#[tauri::command]
pub async fn log_frontend_debug(message: String) -> Result<(), String> {
    debug!("[FRONTEND] {}", message);
    Ok(())
}

#[tauri::command]
pub async fn log_frontend_info(message: String) -> Result<(), String> {
    info!("[FRONTEND] {}", message);
    Ok(())
}

#[tauri::command]
pub async fn log_frontend_warn(message: String) -> Result<(), String> {
    warn!("[FRONTEND] {}", message);
    Ok(())
}

#[tauri::command]
pub async fn log_frontend_error(message: String) -> Result<(), String> {
    error!("[FRONTEND] {}", message);
    Ok(())
}

#[tauri::command]
pub async fn get_log_files_list() -> Result<Vec<String>, String> {
    // This would need access to the Logger instance, but for simplicity,
    // let's read from the default log directory
    let log_dir = PathBuf::from("logs");

    if !log_dir.exists() {
        return Ok(Vec::new());
    }

    let mut log_files = Vec::new();

    match fs::read_dir(&log_dir) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_file() &&
                       path.extension().and_then(|s| s.to_str()) == Some("log") &&
                       path.file_name().and_then(|s| s.to_str()).unwrap_or("").starts_with("regis_") {
                        if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                            log_files.push(filename.to_string());
                        }
                    }
                }
            }
        }
        Err(e) => return Err(format!("Failed to read log directory: {}", e)),
    }

    // Sort by name (which includes timestamp)
    log_files.sort();
    log_files.reverse(); // Newest first

    Ok(log_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_log_level_parsing() {
        assert_eq!(Logger::parse_log_level("debug").unwrap(), LevelFilter::Debug);
        assert_eq!(Logger::parse_log_level("info").unwrap(), LevelFilter::Info);
        assert_eq!(Logger::parse_log_level("warn").unwrap(), LevelFilter::Warn);
        assert_eq!(Logger::parse_log_level("error").unwrap(), LevelFilter::Error);

        assert!(Logger::parse_log_level("invalid").is_err());
    }

    #[test]
    fn test_log_file_creation() {
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().to_str().unwrap();

        let log_file = Logger::create_log_file_path(log_dir).unwrap();
        assert!(log_file.starts_with(temp_dir.path()));
        assert!(log_file.extension().unwrap() == "log");
    }

    #[test]
    fn test_logger_creation() {
        let temp_dir = TempDir::new().unwrap();
        let log_dir = temp_dir.path().to_str().unwrap().to_string();

        let config = LoggingConfig {
            level: "debug".to_string(),
            file_path: None,
            console: true,
            log_dir,
            max_log_size_mb: 10,
            file_rotation: true,
        };

        let logger = Logger::new(&config).unwrap();
        assert_eq!(logger.max_size_mb, 10);
        assert!(logger.log_dir.exists());
    }
}