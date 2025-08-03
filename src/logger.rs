/// Comprehensive logging system for HP-41C calculator debugging
/// 
/// Provides granular control over different types of logging to help debug
/// calculator behavior. Now supports both console and file output.

use std::fmt::Write;
use std::fs::{File, OpenOptions};
use std::io::{Write as IoWrite, BufWriter};
use std::path::{Path, PathBuf};

/// Logger configuration and state with file output support
#[derive(Debug)]
pub struct Logger {
    /// Log flag changes (show_flags, stack lift, input modes, etc.)
    pub log_flags: bool,
    
    /// Log stack state after every modification
    pub log_stack: bool,
    
    /// Log every input keystroke
    pub log_input: bool,
    
    /// Log command parsing and execution
    pub log_commands: bool,
    
    /// Log programming mode operations
    pub log_programming: bool,
    
    /// Log storage register operations
    pub log_storage: bool,
    
    /// Enable/disable all logging at once
    pub enabled: bool,
    
    /// Optional file writer for logging to file
    file_writer: Option<BufWriter<File>>,
    
    /// Path to log file (for display purposes)
    log_file_path: Option<PathBuf>,
}

impl Logger {
    /// Create a new logger with default settings (all disabled)
    pub fn new() -> Self {
        Logger {
            log_flags: false,
            log_stack: false,
            log_input: false,
            log_commands: false,
            log_programming: false,
            log_storage: false,
            enabled: true,
            file_writer: None,
            log_file_path: None,
        }
    }
    
    /// Create a logger with all debugging enabled
    pub fn debug_all() -> Self {
        Logger {
            log_flags: true,
            log_stack: true,
            log_input: true,
            log_commands: true,
            log_programming: true,
            log_storage: true,
            enabled: true,
            file_writer: None,
            log_file_path: None,
        }
    }
    
    /// Create a logger with only flag and stack logging
    pub fn minimal() -> Self {
        Logger {
            log_flags: true,
            log_stack: true,
            log_input: false,
            log_commands: false,
            log_programming: false,
            log_storage: false,
            enabled: true,
            file_writer: None,
            log_file_path: None,
        }
    }
    
    /// Enable file logging to specified path
    pub fn enable_file_logging<P: AsRef<Path>>(&mut self, path: P) -> Result<(), std::io::Error> {
        let path = path.as_ref();
        
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Open file for append
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
            
        self.file_writer = Some(BufWriter::new(file));
        self.log_file_path = Some(path.to_path_buf());
        
        // Write header to log file
        self.write_to_file(&format!("\n=== HP-41C Calculator Log Session Started ===\n"))?;
        
        Ok(())
    }
    
    /// Disable file logging
    pub fn disable_file_logging(&mut self) -> Result<(), std::io::Error> {
        if let Some(mut writer) = self.file_writer.take() {
            Logger::write_to_file_direct(&mut writer, &format!("=== HP-41C Calculator Log Session Ended ===\n"))?;
            writer.flush()?;
        }
        self.log_file_path = None;
        Ok(())
    }
    
    /// Get the current log file path
    pub fn get_log_file_path(&self) -> Option<&Path> {
        self.log_file_path.as_deref()
    }
    
    /// Write a message to the log file (if enabled)
    fn write_to_file(&mut self, message: &str) -> Result<(), std::io::Error> {
        if let Some(writer) = &mut self.file_writer {
            Logger::write_to_file_direct(writer, message)?;
        }
        Ok(())
    }
    
    /// Helper to write to file writer
    fn write_to_file_direct(writer: &mut BufWriter<File>, message: &str) -> Result<(), std::io::Error> {
        writeln!(writer, "{}", message)?;
        writer.flush()?;
        Ok(())
    }
    
    /// Log a message to both console and file
    fn log_message(&mut self, message: &str) {
        if self.enabled {
            println!("{}", message);
            let _ = self.write_to_file(message); // Ignore file errors for now
        }
    }
    
    /// Log an input keystroke
    pub fn log_keystroke(&mut self, key: &str) {
        if self.log_input {
            self.log_message(&format!("[INPUT] Key: '{}'", key));
        }
    }
    
    /// Log a flag change
    pub fn log_flag_change(&mut self, flag_name: &str, old_value: bool, new_value: bool) {
        if self.log_flags {
            self.log_message(&format!("[FLAG] {} changed: {} -> {}", flag_name, old_value, new_value));
        }
    }
    
    /// Log the current stack state
    pub fn log_stack_state(&mut self, stack: &[f64; 4], context: &str) {
        if self.log_stack {
            self.log_message(&format!("[STACK] {}: T:{:10.4} Z:{:10.4} Y:{:10.4} X:{:10.4}", 
                    context, stack[3], stack[2], stack[1], stack[0]));
        }
    }
    
    /// Log stack operation details
    pub fn log_stack_operation(&mut self, operation: &str, before: &[f64; 4], after: &[f64; 4]) {
        if self.log_stack {
            self.log_message(&format!("[STACK] Operation: {}", operation));
            self.log_message(&format!("[STACK]   Before: T:{:10.4} Z:{:10.4} Y:{:10.4} X:{:10.4}", 
                    before[3], before[2], before[1], before[0]));
            self.log_message(&format!("[STACK]   After:  T:{:10.4} Z:{:10.4} Y:{:10.4} X:{:10.4}", 
                    after[3], after[2], after[1], after[0]));
        }
    }
    
    /// Log command parsing state
    pub fn log_command_state(&mut self, state: &str, context: &str) {
        if self.log_commands {
            self.log_message(&format!("[CMD] {}: {}", context, state));
        }
    }
    
    /// Log command execution
    pub fn log_command_execution(&mut self, command: &str, args: &Option<Vec<String>>, result: &str) {
        if self.log_commands {
            let args_str = match args {
                Some(args) => format!(" {}", args.join(" ")),
                None => String::new(),
            };
            self.log_message(&format!("[CMD] Execute: {}{} -> {}", command, args_str, result));
        }
    }
    
    /// Log programming mode operations
    pub fn log_programming(&mut self, operation: &str, details: &str) {
        if self.log_programming {
            self.log_message(&format!("[PRGM] {}: {}", operation, details));
        }
    }
    
    /// Log storage register operations
    pub fn log_storage_operation(&mut self, operation: &str, register: usize, value: f64) {
        if self.log_storage {
            self.log_message(&format!("[STORAGE] {} register {:02}: {}", operation, register, value));
        }
    }
    
    /// Log input state changes
    pub fn log_input_state(&mut self, entering: bool, eex_mode: bool, display: &str) {
        if self.log_flags {
            self.log_message(&format!("[INPUT] State: entering={}, eex={}, display='{}'", 
                    entering, eex_mode, display));
        }
    }
    
    /// Log a general debug message with category
    pub fn log_debug(&mut self, category: &str, message: &str) {
        self.log_message(&format!("[{}] {}", category, message));
    }
    
    /// Get current logging configuration as a string
    pub fn get_config_string(&self) -> String {
        if !self.enabled {
            return "Logging: DISABLED".to_string();
        }
        
        let mut config = String::new();
        write!(&mut config, "Logging: ").unwrap();
        
        let mut active = Vec::new();
        if self.log_flags { active.push("FLAGS"); }
        if self.log_stack { active.push("STACK"); }
        if self.log_input { active.push("INPUT"); }
        if self.log_commands { active.push("COMMANDS"); }
        if self.log_programming { active.push("PROGRAMMING"); }
        if self.log_storage { active.push("STORAGE"); }
        
        if active.is_empty() {
            write!(&mut config, "NONE").unwrap();
        } else {
            write!(&mut config, "{}", active.join("|")).unwrap();
        }
        
        // Add file info if logging to file
        if self.log_file_path.is_some() {
            write!(&mut config, " -> FILE").unwrap();
        }
        
        config
    }
    
    /// Enable/disable specific log types with a convenience method
    pub fn set_flags(&mut self, flags: bool, stack: bool, input: bool, commands: bool) {
        self.log_flags = flags;
        self.log_stack = stack;
        self.log_input = input;
        self.log_commands = commands;
    }
    
    /// Toggle all logging on/off
    pub fn toggle_enabled(&mut self) -> bool {
        self.enabled = !self.enabled;
        self.enabled
    }
    
    /// Reset to default configuration
    pub fn reset(&mut self) {
        let file_writer = self.file_writer.take();
        let log_file_path = self.log_file_path.take();
        
        *self = Logger::new();
        
        // Preserve file logging if it was enabled
        self.file_writer = file_writer;
        self.log_file_path = log_file_path;
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

// Note: The Clone trait is harder to implement now due to BufWriter<File>
// If Clone is needed, we'd need to reopen the file
impl Clone for Logger {
    fn clone(&self) -> Self {
        Logger {
            log_flags: self.log_flags,
            log_stack: self.log_stack,
            log_input: self.log_input,
            log_commands: self.log_commands,
            log_programming: self.log_programming,
            log_storage: self.log_storage,
            enabled: self.enabled,
            file_writer: None, // Can't clone file writers
            log_file_path: self.log_file_path.clone(),
        }
    }
}

/// Convenience macro for conditional logging
#[macro_export]
macro_rules! debug_log {
    ($logger:expr, $category:expr, $($arg:tt)*) => {
        if $logger.enabled {
            $logger.log_debug($category, &format!($($arg)*));
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn test_logger_creation() {
        let logger = Logger::new();
        assert!(!logger.log_flags);
        assert!(!logger.log_stack);
        assert!(logger.enabled);
        
        let debug_logger = Logger::debug_all();
        assert!(debug_logger.log_flags);
        assert!(debug_logger.log_stack);
        assert!(debug_logger.log_input);
    }
    
    #[test]
    fn test_config_string() {
        let mut logger = Logger::new();
        assert_eq!(logger.get_config_string(), "Logging: NONE");
        
        logger.log_flags = true;
        logger.log_stack = true;
        assert!(logger.get_config_string().contains("FLAGS"));
        assert!(logger.get_config_string().contains("STACK"));
        
        logger.enabled = false;
        assert_eq!(logger.get_config_string(), "Logging: DISABLED");
    }
    
    #[test]
    fn test_file_logging() -> Result<(), Box<dyn std::error::Error>> {
        let mut logger = Logger::new();
        let temp_path = PathBuf::from("test_hp41c.log");
        
        // Enable file logging
        logger.enable_file_logging(&temp_path)?;
        assert!(logger.get_log_file_path().is_some());
        
        // Log some messages
        logger.log_flags = true;
        logger.log_flag_change("test_flag", false, true);
        
        // Disable file logging
        logger.disable_file_logging()?;
        
        // Check that file was created and contains content
        let content = fs::read_to_string(&temp_path)?;
        assert!(content.contains("HP-41C Calculator Log Session Started"));
        assert!(content.contains("test_flag"));
        assert!(content.contains("HP-41C Calculator Log Session Ended"));
        
        // Clean up
        fs::remove_file(&temp_path).ok();
        
        Ok(())
    }
}