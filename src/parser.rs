/// Command Parser for HP-41C Keystroke Processing
/// 
/// Handles keystroke-by-keystroke command parsing using the command registry.
/// This is designed for real-time keystroke processing, not command-line input.

use crate::registry::{CommandRegistry, ArgumentPattern, AutoExecuteRule};

/// Result of parsing a command input
#[derive(Debug, Clone)]
pub enum ParseResult {
    /// Still building the command, need more input
    Incomplete,
    
    /// Command is complete and ready to execute
    Complete { command: String, args: Option<Vec<String>> },
    
    /// Invalid input
    Invalid(String),
}

/// Unified command parser that uses specifications
/// 
/// ## Keystroke-by-Keystroke Processing
/// 
/// This parser is designed for real-time keystroke processing, not command-line input.
/// Each call to `add_input()` receives a single character and updates the parsing state:
/// 
/// ```ignore
/// let mut parser = CommandParser::new();
/// 
/// // User types "fix 4" one keystroke at a time:
/// parser.add_input("f");    // → Incomplete (building "f...")
/// parser.add_input("i");    // → Incomplete (building "fi...")  
/// parser.add_input("x");    // → Incomplete (command "fix" ready for args)
/// parser.add_input("4");    // → Complete { command: "fix", args: Some(["4"]) }
/// ```
/// 
/// The parser maintains state across keystrokes until a command is complete.
#[derive(Debug)]
pub struct CommandParser {
    registry: CommandRegistry,
    current_command: String,
    current_args: Vec<String>,
}

impl CommandParser {
    /// Create a new parser
    pub fn new() -> Self {
        CommandParser {
            registry: CommandRegistry::new(),
            current_command: String::new(),
            current_args: Vec::new(),
        }
    }
    
    /// Clear current parsing state
    pub fn clear(&mut self) {
        self.current_command.clear();
        self.current_args.clear();
    }
    
    /// Add input to the current command being built
    pub fn add_input(&mut self, input: &str) -> ParseResult {
        if self.current_command.is_empty() {
            return self.start_command(input);
        }
        
        if self.registry.get_spec(&self.current_command).is_some() {
            self.add_argument(input)
        } else {
            self.continue_building_command(input)
        }
    }
    
    /// Start parsing a new command
    fn start_command(&mut self, input: &str) -> ParseResult {
        let input_lower = input.to_lowercase();
        
        if let Some(spec) = self.registry.get_spec(&input_lower) {
            self.current_command = input_lower;
            
            if matches!(spec.arg_pattern, ArgumentPattern::None) {
                // Command executes immediately - clear state and return complete
                let command = self.current_command.clone();
                self.clear();
                return ParseResult::Complete {
                    command,
                    args: None,
                };
            }
            
            return ParseResult::Incomplete;
        }
        
        self.current_command = input_lower;
        
        if self.could_be_command_prefix(&self.current_command) {
            ParseResult::Incomplete
        } else {
            ParseResult::Invalid(format!("Unknown command: {}", input))
        }
    }
    
    /// Continue building a command name
    fn continue_building_command(&mut self, input: &str) -> ParseResult {
        let input_lower = input.to_lowercase();
        let new_command = format!("{}{}", self.current_command, input_lower);
        
        if let Some(spec) = self.registry.get_spec(&new_command) {
            self.current_command = new_command;
            
            if matches!(spec.arg_pattern, ArgumentPattern::None) {
                // Command executes immediately - clear state and return complete
                let command = self.current_command.clone();
                self.clear();
                return ParseResult::Complete {
                    command,
                    args: None,
                };
            }
            
            return ParseResult::Incomplete;
        }
        
        if self.could_be_command_prefix(&new_command) {
            self.current_command = new_command;
            ParseResult::Incomplete
        } else {
            ParseResult::Invalid(format!("Unknown command: {}", new_command))
        }
    }
    
    /// Check if a string could be the prefix of any valid command
    fn could_be_command_prefix(&self, prefix: &str) -> bool {
        self.registry.get_command_names().iter().any(|cmd| cmd.starts_with(prefix))
    }
    
    /// Add an argument to the current command
    fn add_argument(&mut self, arg: &str) -> ParseResult {
        let spec = self.registry.get_spec(&self.current_command)
            .expect("Command should exist if we got here");
        
        match &spec.arg_pattern {
            ArgumentPattern::Register => {
                // Build up the register number digit by digit
                if self.current_args.is_empty() {
                    // First digit of register number
                    if arg.len() == 1 && arg.chars().next().unwrap().is_ascii_digit() {
                        self.current_args.push(arg.to_string());
                        ParseResult::Incomplete // Wait for second digit
                    } else {
                        ParseResult::Invalid(format!("Register number must be digits, got '{}'", arg))
                    }
                } else {
                    // Second digit of register number - complete the argument
                    if arg.len() == 1 && arg.chars().next().unwrap().is_ascii_digit() {
                        let full_register = format!("{}{}", self.current_args[0], arg);
                        if let Ok(num) = full_register.parse::<u8>() {
                            if num <= 99 {
                                // Complete 2-digit register number
                                self.current_args[0] = full_register;
                                
                                let command = self.current_command.clone();
                                let args = Some(self.current_args.clone());
                                self.clear();
                                ParseResult::Complete { command, args }
                            } else {
                                ParseResult::Invalid(format!("Register number {} too large (max 99)", full_register))
                            }
                        } else {
                            ParseResult::Invalid(format!("Invalid register number: {}", full_register))
                        }
                    } else {
                        ParseResult::Invalid(format!("Register number must be digits, got '{}'", arg))
                    }
                }
            }
            
            _ => {
                // For other argument patterns, validate and complete immediately
                if !self.is_valid_argument(arg, &spec.arg_pattern) {
                    return ParseResult::Invalid(format!("Invalid argument '{}' for {}", arg, self.current_command));
                }
                
                self.current_args.push(arg.to_string());
                
                if self.is_complete(&spec.arg_pattern) {
                    match spec.auto_execute {
                        AutoExecuteRule::OnComplete => {
                            let command = self.current_command.clone();
                            let args = if self.current_args.is_empty() { 
                                None 
                            } else { 
                                Some(self.current_args.clone()) 
                            };
                            
                            self.clear();
                            ParseResult::Complete { command, args }
                        }
                        _ => {
                            ParseResult::Incomplete
                        }
                    }
                } else {
                    ParseResult::Incomplete
                }
            }
        }
    }
    
    /// Check if an argument is valid for the given pattern
    fn is_valid_argument(&self, arg: &str, pattern: &ArgumentPattern) -> bool {
        match pattern {
            ArgumentPattern::None => false,
            
            ArgumentPattern::SingleDigit => {
                arg.len() == 1 && arg.chars().next().unwrap().is_ascii_digit()
            }
            
            ArgumentPattern::Register => {
                // Register validation is now handled in add_argument method
                true
            }
            
            ArgumentPattern::Label => {
                if arg.len() != 1 { return false; }
                let ch = arg.chars().next().unwrap();
                ch.is_ascii_alphanumeric()
            }
            
            ArgumentPattern::Alpha => {
                !arg.is_empty() && arg.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            }
            
            ArgumentPattern::Custom(validator) => {
                validator(arg)
            }
        }
    }
    
    /// Check if we have all required arguments
    fn is_complete(&self, pattern: &ArgumentPattern) -> bool {
        match pattern {
            ArgumentPattern::None => true,
            ArgumentPattern::Register => {
                // Register completion is handled in add_argument method
                false // Never complete here - always handle in add_argument
            }
            _ => !self.current_args.is_empty(),
        }
    }
    
    /// Force completion of current command (for manual execution)
    pub fn force_complete(&mut self) -> ParseResult {
        if self.current_command.is_empty() {
            return ParseResult::Invalid("No command to complete".to_string());
        }
        
        let command = self.current_command.clone();
        let args = if self.current_args.is_empty() { 
            None 
        } else { 
            Some(self.current_args.clone()) 
        };
        
        self.clear();
        ParseResult::Complete { command, args }
    }
    
    /// Get current parsing state for display
    pub fn get_current_state(&self) -> String {
        if self.current_command.is_empty() {
            "CMD: []".to_string()
        } else if self.current_args.is_empty() {
            format!("CMD: [{}]", self.current_command)
        } else {
            // Special display for register numbers being built
            if matches!(self.current_command.as_str(), "sto" | "rcl") && self.current_args.len() == 1 && self.current_args[0].len() == 1 {
                format!("CMD: [{} {}_]", self.current_command, self.current_args[0])
            } else {
                format!("CMD: [{} {}]", self.current_command, self.current_args.join(" "))
            }
        }
    }
    
    /// Check if we're currently building a command
    pub fn is_building(&self) -> bool {
        !self.current_command.is_empty()
    }
    
    /// Get access to the command registry (for advanced use)
    pub fn registry(&self) -> &CommandRegistry {
        &self.registry
    }
}

impl Default for CommandParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_immediate_commands() {
        let mut parser = CommandParser::new();
        
        // SIN should execute immediately
        match parser.add_input("sin") {
            ParseResult::Complete { command, args } => {
                assert_eq!(command, "sin");
                assert_eq!(args, None);
            }
            _ => panic!("SIN should complete immediately"),
        }
    }
    
    #[test]
    fn test_command_building() {
        let mut parser = CommandParser::new();
        
        // Build "fix" letter by letter
        assert!(matches!(parser.add_input("f"), ParseResult::Incomplete));
        assert!(matches!(parser.add_input("i"), ParseResult::Incomplete));
        assert!(matches!(parser.add_input("x"), ParseResult::Incomplete));
        
        // Adding digit should complete
        match parser.add_input("4") {
            ParseResult::Complete { command, args } => {
                assert_eq!(command, "fix");
                assert_eq!(args, Some(vec!["4".to_string()]));
            }
            _ => panic!("FIX 4 should complete"),
        }
    }
    
    #[test]
    fn test_register_building() {
        let mut parser = CommandParser::new();
        
        // Build "sto" command
        assert!(matches!(parser.add_input("s"), ParseResult::Incomplete));
        assert!(matches!(parser.add_input("t"), ParseResult::Incomplete));
        assert!(matches!(parser.add_input("o"), ParseResult::Incomplete));
        
        // First digit should be incomplete
        assert!(matches!(parser.add_input("1"), ParseResult::Incomplete));
        
        // Second digit should complete
        match parser.add_input("5") {
            ParseResult::Complete { command, args } => {
                assert_eq!(command, "sto");
                assert_eq!(args, Some(vec!["15".to_string()]));
            }
            _ => panic!("STO 15 should complete"),
        }
    }
    
    #[test]
    fn test_invalid_commands() {
        let mut parser = CommandParser::new();
        
        // Invalid command should be rejected
        match parser.add_input("xyz") {
            ParseResult::Invalid(_) => {}, // Expected
            _ => panic!("Invalid command should be rejected"),
        }
    }
    
    #[test]
    fn test_display_state() {
        let mut parser = CommandParser::new();
        
        // Empty state
        assert_eq!(parser.get_current_state(), "CMD: []");
        
        // Building command
        assert!(matches!(parser.add_input("f"), ParseResult::Incomplete));
        assert!(matches!(parser.add_input("i"), ParseResult::Incomplete));
        assert!(parser.get_current_state().contains("fi"));
        
        // Complete command waiting for args
        assert!(matches!(parser.add_input("x"), ParseResult::Incomplete));
        assert!(parser.get_current_state().contains("fix"));
    }
    
    #[test]
    fn test_force_complete() {
        let mut parser = CommandParser::new();
        
        // Build partial command
        assert!(matches!(parser.add_input("f"), ParseResult::Incomplete));
        assert!(matches!(parser.add_input("i"), ParseResult::Incomplete));
        assert!(matches!(parser.add_input("x"), ParseResult::Incomplete));
        
        // Force completion should work
        match parser.force_complete() {
            ParseResult::Complete { command, args } => {
                assert_eq!(command, "fix");
                assert_eq!(args, None);
            }
            _ => panic!("Force complete should work"),
        }
    }
}
