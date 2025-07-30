/// Updated commands module - clean version without backwards compatibility
/// 
/// The old CommandTrie has been completely replaced by the data-driven command system
/// built into the calculator. This module now provides clean helper functions.

// Re-export the command system types
pub use crate::calculator::{
    CommandSpec, ArgumentPattern, AutoExecuteRule, ParseResult, 
    CommandRegistry, CommandParser
};

/// Helper function to check if a string is a valid HP-41C command
pub fn is_valid_command(command: &str) -> bool {
    let registry = CommandRegistry::new();
    registry.get_spec(&command.to_lowercase()).is_some()
}

/// Get all available command names
pub fn get_all_commands() -> Vec<String> {
    let registry = CommandRegistry::new();
    registry.get_command_names().into_iter().cloned().collect()
}

/// Get command specification for a given command
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
        assert!(commands.len() > 20);
    }

    #[test]
    fn test_command_spec_retrieval() {
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
}
