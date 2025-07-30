/// Command Registry System for HP-41C
/// 
/// Provides declarative command specifications and registry management.
/// This replaces the old hardcoded command logic with a clean, data-driven approach.

use std::collections::HashMap;

/// Specification for how a command should be parsed and executed
#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub name: String,
    pub arg_pattern: ArgumentPattern,
    pub auto_execute: AutoExecuteRule,
    pub description: Option<String>,
}

/// Defines what kind of arguments a command expects
#[derive(Debug, Clone)]
pub enum ArgumentPattern {
    /// Command takes no arguments (e.g., SIN, COS, ENTER)
    None,
    
    /// Single digit 0-9 (e.g., FIX 4, SCI 2)
    SingleDigit,
    
    /// Register number 00-99 (e.g., STO 15, RCL 07)
    Register,
    
    /// Label: single letter A-Z or number 0-9 (e.g., LBL A, GTO 5)
    Label,
    
    /// Alpha string for program names (e.g., XEQ "MYPROG")
    Alpha,
    
    /// Custom validation function
    Custom(fn(&str) -> bool),
}

/// When should the command execute
#[derive(Debug, Clone)]
pub enum AutoExecuteRule {
    /// Execute immediately when command is complete (e.g., SIN, +)
    Immediate,
    
    /// Execute when arguments are complete (e.g., FIX 4, STO 15)
    OnComplete,
    
    /// Only execute on space or enter (e.g., some complex commands)
    Manual,
}

/// Registry of all known commands with their specifications
#[derive(Debug)]
pub struct CommandRegistry {
    specs: HashMap<String, CommandSpec>,
}

impl CommandRegistry {
    /// Create a new registry with all HP-41C commands
    pub fn new() -> Self {
        let mut registry = CommandRegistry {
            specs: HashMap::new(),
        };
        registry.register_all_commands();
        registry
    }
    
    /// Register all HP-41C commands
    fn register_all_commands(&mut self) {
        // Math functions - no arguments, execute immediately
        for &cmd in &["sin", "cos", "tan", "asin", "acos", "atan", 
                      "log", "ln", "exp", "sqrt", "inv", "chs"] {
            self.register(CommandSpec {
                name: cmd.to_string(),
                arg_pattern: ArgumentPattern::None,
                auto_execute: AutoExecuteRule::Immediate,
                description: Some(format!("{} function", cmd.to_uppercase())),
            });
        }
        
        // Stack operations - no arguments, execute immediately  
        for &cmd in &["enter", "swap", "clx", "clr"] {
            self.register(CommandSpec {
                name: cmd.to_string(),
                arg_pattern: ArgumentPattern::None,
                auto_execute: AutoExecuteRule::Immediate,
                description: Some(format!("{} operation", cmd.to_uppercase())),
            });
        }
        
        // Arithmetic operators - no arguments, execute immediately
        for &cmd in &["+", "-", "*", "/", "^", "!"] {
            self.register(CommandSpec {
                name: cmd.to_string(),
                arg_pattern: ArgumentPattern::None,
                auto_execute: AutoExecuteRule::Immediate,
                description: Some("Arithmetic operation".to_string()),
            });
        }
        
        // Display modes - single digit argument, auto-execute on complete
        for &cmd in &["fix", "sci", "eng"] {
            self.register(CommandSpec {
                name: cmd.to_string(),
                arg_pattern: ArgumentPattern::SingleDigit,
                auto_execute: AutoExecuteRule::OnComplete,
                description: Some(format!("{} display mode", cmd.to_uppercase())),
            });
        }
        
        // Storage operations - register argument, auto-execute on complete
        for &cmd in &["sto", "rcl"] {
            self.register(CommandSpec {
                name: cmd.to_string(),
                arg_pattern: ArgumentPattern::Register,
                auto_execute: AutoExecuteRule::OnComplete,
                description: Some(format!("{} operation", cmd.to_uppercase())),
            });
        }
        
        // Programming commands with labels
        for &cmd in &["lbl", "gto"] {
            self.register(CommandSpec {
                name: cmd.to_string(),
                arg_pattern: ArgumentPattern::Label,
                auto_execute: AutoExecuteRule::OnComplete,
                description: Some(format!("{} programming command", cmd.to_uppercase())),
            });
        }
        
        // Program execution
        self.register(CommandSpec {
            name: "xeq".to_string(),
            arg_pattern: ArgumentPattern::Alpha,
            auto_execute: AutoExecuteRule::OnComplete,
            description: Some("Execute program".to_string()),
        });
        
        // Programming control - no args, immediate
        for &cmd in &["rtn", "sst", "bst", "prgm"] {
            self.register(CommandSpec {
                name: cmd.to_string(),
                arg_pattern: ArgumentPattern::None,
                auto_execute: AutoExecuteRule::Immediate,
                description: Some(format!("{} programming command", cmd.to_uppercase())),
            });
        }
        
        // Constants - no arguments, execute immediately
        for &cmd in &["pi"] {
            self.register(CommandSpec {
                name: cmd.to_string(),
                arg_pattern: ArgumentPattern::None,
                auto_execute: AutoExecuteRule::Immediate,
                description: Some("Mathematical constant".to_string()),
            });
        }
        
        // Special commands
        self.register(CommandSpec {
            name: "eex".to_string(),
            arg_pattern: ArgumentPattern::None,
            auto_execute: AutoExecuteRule::Immediate,
            description: Some("Enter exponent".to_string()),
        });
        
        self.register(CommandSpec {
            name: "arc".to_string(),
            arg_pattern: ArgumentPattern::None,
            auto_execute: AutoExecuteRule::Immediate,
            description: Some("Arc mode prefix".to_string()),
        });
    }
    
    /// Register a single command specification
    pub fn register(&mut self, spec: CommandSpec) {
        self.specs.insert(spec.name.clone(), spec);
    }
    
    /// Get specification for a command
    pub fn get_spec(&self, command: &str) -> Option<&CommandSpec> {
        self.specs.get(command)
    }
    
    /// Get all registered command names
    pub fn get_command_names(&self) -> Vec<&String> {
        self.specs.keys().collect()
    }
    
    /// Get all command specifications (for advanced use)
    pub fn get_all_specs(&self) -> &HashMap<String, CommandSpec> {
        &self.specs
    }
    
    /// Check if a command exists
    pub fn has_command(&self, command: &str) -> bool {
        self.specs.contains_key(command)
    }
    
    /// Get commands by category (for help systems, etc.)
    pub fn get_commands_by_pattern(&self, pattern: &ArgumentPattern) -> Vec<&CommandSpec> {
        self.specs.values()
            .filter(|spec| std::mem::discriminant(&spec.arg_pattern) == std::mem::discriminant(pattern))
            .collect()
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = CommandRegistry::new();
        assert!(registry.has_command("sin"));
        assert!(registry.has_command("sto"));
        assert!(registry.has_command("fix"));
        assert!(!registry.has_command("invalid"));
    }

    #[test]
    fn test_command_specs() {
        let registry = CommandRegistry::new();
        
        let sin_spec = registry.get_spec("sin").unwrap();
        assert_eq!(sin_spec.name, "sin");
        assert!(matches!(sin_spec.arg_pattern, ArgumentPattern::None));
        assert!(matches!(sin_spec.auto_execute, AutoExecuteRule::Immediate));
        
        let sto_spec = registry.get_spec("sto").unwrap();
        assert_eq!(sto_spec.name, "sto");
        assert!(matches!(sto_spec.arg_pattern, ArgumentPattern::Register));
        assert!(matches!(sto_spec.auto_execute, AutoExecuteRule::OnComplete));
    }

    #[test]
    fn test_commands_by_pattern() {
        let registry = CommandRegistry::new();
        
        let math_commands = registry.get_commands_by_pattern(&ArgumentPattern::None);
        assert!(math_commands.iter().any(|spec| spec.name == "sin"));
        assert!(math_commands.iter().any(|spec| spec.name == "cos"));
        
        let storage_commands = registry.get_commands_by_pattern(&ArgumentPattern::Register);
        assert!(storage_commands.iter().any(|spec| spec.name == "sto"));
        assert!(storage_commands.iter().any(|spec| spec.name == "rcl"));
    }

    #[test]
    fn test_command_count() {
        let registry = CommandRegistry::new();
        let command_names = registry.get_command_names();
        assert!(command_names.len() > 30, "Should have plenty of commands registered");
    }
}
