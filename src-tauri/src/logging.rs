//! Logging module for MT2 Draft Assistant
//!
//! Provides structured logging to file and console.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use chrono::Local;
use log::{LevelFilter, Metadata, Record};

/// Custom logger that writes to both file and console
pub struct AppLogger {
    log_file: Mutex<std::fs::File>,
    max_level: LevelFilter,
}

impl AppLogger {
    pub fn new(log_dir: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let log_path = log_dir.join("mt2_draft_assistant.log");
        
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)?;
        
        Ok(Self {
            log_file: Mutex::new(file),
            max_level: LevelFilter::Info,
        })
    }
}

impl log::Log for AppLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.max_level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
        let log_line = format!(
            "[{}] {:<5} [{}] {}\n",
            timestamp,
            record.level(),
            record.target(),
            record.args()
        );

        // Write to file
        if let Ok(mut file) = self.log_file.lock() {
            let _ = file.write_all(log_line.as_bytes());
            let _ = file.flush();
        }

        // Also print to console for development
        println!("{}", log_line.trim());
    }

    fn flush(&self) {
        if let Ok(mut file) = self.log_file.lock() {
            let _ = file.flush();
        }
    }
}

/// Initialize the logging system
pub fn init() {
    // Determine log directory
    let log_dir = if let Some(data_dir) = dirs::data_dir() {
        data_dir.join("com.mt2.overlay")
    } else {
        PathBuf::from(".")
    };

    // Create log directory if it doesn't exist
    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!("Failed to create log directory: {}", e);
        return;
    }

    // Create and set logger
    match AppLogger::new(log_dir) {
        Ok(logger) => {
            let max_level = logger.max_level;
            if let Err(e) = log::set_boxed_logger(Box::new(logger)) {
                eprintln!("Failed to set logger: {}", e);
                return;
            }
            log::set_max_level(max_level);
        }
        Err(e) => {
            eprintln!("Failed to create logger: {}", e);
        }
    }
}

/// Log an error with context
#[macro_export]
macro_rules! log_error {
    ($context:expr, $error:expr) => {
        log::error!("{}: {}", $context, $error);
    };
}

/// Log an info message with context
#[macro_export]
macro_rules! log_info {
    ($context:expr, $message:expr) => {
        log::info!("{}: {}", $context, $message);
    };
}

/// Log a debug message with context (only in debug builds)
#[macro_export]
macro_rules! log_debug {
    ($context:expr, $message:expr) => {
        log::debug!("{}: {}", $context, $message);
    };
}

/// Log a warning with context
#[macro_export]
macro_rules! log_warn {
    ($context:expr, $message:expr) => {
        log::warn!("{}: {}", $context, $message);
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_logger_creation() {
        let temp_dir = TempDir::new().unwrap();
        let logger = AppLogger::new(temp_dir.path().to_path_buf());
        assert!(logger.is_ok());
    }

    #[test]
    fn test_log_file_created() {
        let temp_dir = TempDir::new().unwrap();
        let logger = AppLogger::new(temp_dir.path().to_path_buf()).unwrap();
        
        // Log a message
        log::set_boxed_logger(Box::new(logger)).ok();
        log::set_max_level(LevelFilter::Info);
        log::info!("Test message");
        
        // Check file exists
        let log_file = temp_dir.path().join("mt2_draft_assistant.log");
        assert!(log_file.exists());
        
        // Check content
        let content = fs::read_to_string(log_file).unwrap();
        assert!(content.contains("Test message"));
    }
}
