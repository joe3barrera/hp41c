/// Comprehensive logging system for HP-41C calculator debugging
/// 
/// Provides granular control over different types of logging to help debug
/// calculator behavior without cluttering the display.

use std::fmt::Write;

/// Logger configuration and state
#[derive(Debug, Clone)]
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
        }
    }
    
    /// Log an input keystroke
    pub fn log_keystroke(&self, key: &str) {
        if self.enabled && self.log_input {
            println!("[INPUT] Key: '{}'", key);
        }
    }
    
    /// Log a flag change
    pub fn log_flag_change(&self, flag_name: &str, old_value: bool, new_value: bool) {
        if self.enabled && self.log_flags {
            println!("[FLAG] {} changed: {} -> {}", flag_name, old_value, new_value);
        }
    }
    
    /// Log the current stack state
    pub fn log_stack_state(&self, stack: &[f64; 4], context: &str) {
        if self.enabled && self.log_stack {
            println!("[STACK] {}: T:{:10.4} Z:{:10.4} Y:{:10.4} X:{:10.4}", 
                    context, stack[3], stack[2], stack[1], stack[0]);
        }
    }
    
    /// Log stack operation details
    pub fn log_stack_operation(&self, operation: &str, before: &[f64; 4], after: &[f64; 4]) {
        if self.enabled && self.log_stack {
            println!("[STACK] Operation: {}", operation);
            println!("[STACK]   Before: T:{:10.4} Z:{:10.4} Y:{:10.4} X:{:10.4}", 
                    before[3], before[2], before[1], before[0]);
            println!("[STACK]   After:  T:{:10.4} Z:{:10.4} Y:{:10.4} X:{:10.4}", 
                    after[3], after[2], after[1], after[0]);
        }
    }
    
    /// Log command parsing state
    pub fn log_command_state(&self, state: &str, context: &str) {
        if self.enabled && self.log_commands {
            println!("[CMD] {}: {}", context, state);
        }
    }
    
    /// Log command execution
    pub fn log_command_execution(&self, command: &str, args: &Option<Vec<String>>, result: &str) {
        if self.enabled && self.log_commands {
            let args_str = match args {
                Some(args) => format!(" {}", args.join(" ")),
                None => String::new(),
            };
            println!("[CMD] Execute: {}{} -> {}", command, args_str, result);
        }
    }
    
    /// Log programming mode operations
    pub fn log_programming(&self, operation: &str, details: &str) {
        if self.enabled && self.log_programming {
            println!("[PRGM] {}: {}", operation, details);
        }
    }
    
    /// Log storage register operations
    pub fn log_storage_operation(&self, operation: &str, register: usize, value: f64) {
        if self.enabled && self.log_storage {
            println!("[STORAGE] {} register {:02}: {}", operation, register, value);
        }
    }
    
    /// Log input state changes
    pub fn log_input_state(&self, entering: bool, eex_mode: bool, display: &str) {
        if self.enabled && self.log_flags {
            println!("[INPUT] State: entering={}, eex={}, display='{}'", 
                    entering, eex_mode, display);
        }
    }
    
    /// Log a general debug message with category
    pub fn log_debug(&self, category: &str, message: &str) {
        if self.enabled {
            println!("[{}] {}", category, message);
        }
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
        *self = Logger::new();
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience macro for conditional logging
#[macro_export]
macro_rules! debug_log {
    ($logger:expr, $category:expr, $($arg:tt)*) => {
        if $logger.enabled {
            println!("[{}] {}", $category, format!($($arg)*));
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

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
}