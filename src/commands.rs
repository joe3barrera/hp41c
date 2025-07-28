/// Updated commands module - now much simpler!
/// 
/// The old CommandTrie has been replaced by the data-driven command system
/// built into the calculator. This module now just provides helper functions
/// and exports for backwards compatibility.

use std::collections::HashMap;

// Re-export the command system types for external use
pub use crate::calculator::{
    CommandSpec, ArgumentPattern, AutoExecuteRule, ParseResult, 
    CommandRegistry, CommandParser
};

/// Initialize command trie - DEPRECATED, kept for backwards compatibility
/// 
/// The new system uses CommandRegistry instead, which is automatically
/// initialized in the CommandParser. This function now returns an empty
/// trie to avoid breaking existing code.
#[deprecated(note = "Use CommandRegistry in the calculator instead")]
pub fn initialize_command_trie() -> CommandTrie {
    CommandTrie::new()
}

/// Command trie for backwards compatibility - much simpler now!
#[derive(Debug, Clone)]
pub struct CommandTrie {
    children: HashMap<char, CommandTrie>,
    is_command: bool,
    command_name: Option<String>,
}

impl CommandTrie {
    pub fn new() -> Self {
        CommandTrie {
            children: HashMap::new(),
            is_command: false,
            command_name: None,
        }
    }

    /// Insert method kept for backwards compatibility but does nothing
    #[deprecated(note = "Commands are now registered in CommandRegistry")]
    pub fn insert(&mut self, _command: &str) {
        // No-op for backwards compatibility
    }

    /// Search method kept for backwards compatibility
    #[deprecated(note = "Use CommandParser instead")]
    pub fn search(&self, _prefix: &str) -> (bool, bool, Option<String>) {
        // Always return "not found" to encourage migration to new system
        (false, false, None)
    }
}

/// Helper function to check if a string is a valid HP-41C command
/// 
/// This uses the new command registry system under the hood.
pub fn is_valid_command(command: &str) -> bool {
    let registry = CommandRegistry::new();
    registry.get_spec(&command.to_lowercase()).is_some()
}

/// Get all available command names
/// 
/// Returns a list of all commands supported by the calculator.
pub fn get_all_commands() -> Vec<String> {
    let registry = CommandRegistry::new();
    registry.get_command_names().into_iter().cloned().collect()
}

/// Get command specification for a given command
/// 
/// Returns the command spec if the command exists, None otherwise.
pub fn get_command_spec(command: &str) -> Option<CommandSpec> {
    let registry = CommandRegistry::new();
    registry.get_spec(&command.to_lowercase()).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_validation() {
        assert!(is_valid_command("sin"));
        assert!(is_valid_command("STO"));
        assert!(is_valid_command("fix"));
        assert!(!is_valid_command("invalid"));
    }

    #[test]
    fn test_get_all_commands() {
        let commands = get_all_commands();
        assert!(commands.contains(&"sin".to_string()));
        assert!(commands.contains(&"sto".to_string()));
        assert!(commands.contains(&"fix".to_string()));
        assert!(commands.len() > 20); // Should have plenty of commands
    }

    #[test]
    fn test_get_command_spec() {
        let sin_spec = get_command_spec("sin").unwrap();
        assert_eq!(sin_spec.name, "sin");
        assert!(matches!(sin_spec.arg_pattern, ArgumentPattern::None));
        assert!(matches!(sin_spec.auto_execute, AutoExecuteRule::Immediate));

        let sto_spec = get_command_spec("sto").unwrap();
        assert_eq!(sto_spec.name, "sto");
        assert!(matches!(sto_spec.arg_pattern, ArgumentPattern::Register));
        assert!(matches!(sto_spec.auto_execute, AutoExecuteRule::OnComplete));

        assert!(get_command_spec("invalid").is_none());
    }

    #[test]
    fn test_backwards_compatibility() {
        // Old CommandTrie should still compile but not do much
        let mut trie = initialize_command_trie();
        trie.insert("sin");
        let (valid, complete, _) = trie.search("sin");
        assert!(!valid); // Should return false to encourage migration
        assert!(!complete);
    }
}
