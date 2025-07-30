/// HP-41C Calculator Core - Updated with Data-Driven Command System
/// 
/// IMPORTANT: This is keystroke-by-keystroke processing, NOT command-line!

use crate::programming::ProgrammingMode;
use crate::display::DisplayFormatter;
#[cfg(test)]
use crate::display::DisplayMode;
use crate::stack::Stack;
use crate::input::InputState;
use crate::execution::execute_command;
use std::collections::HashMap;

/// Maximum number of storage registers
const NUM_STORAGE_REGISTERS: usize = 100;

// ===== COMMAND SYSTEM =====

#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub name: String,
    pub arg_pattern: ArgumentPattern,
    pub auto_execute: AutoExecuteRule,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ArgumentPattern {
    None,
    SingleDigit,
    Register,
    Label,
    Alpha,
    Custom(fn(&str) -> bool),
}

#[derive(Debug, Clone)]
pub enum AutoExecuteRule {
    Immediate,
    OnComplete,
    Manual,
}

#[derive(Debug, Clone)]
pub enum ParseResult {
    Incomplete,
    Complete { command: String, args: Option<Vec<String>> },
    Invalid(String),
}

#[derive(Debug)]
pub struct CommandRegistry {
    specs: HashMap<String, CommandSpec>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        let mut registry = CommandRegistry {
            specs: HashMap::new(),
        };
        registry.register_all_commands();
        registry
    }
    
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
    
    pub fn register(&mut self, spec: CommandSpec) {
        self.specs.insert(spec.name.clone(), spec);
    }
    
    pub fn get_spec(&self, command: &str) -> Option<&CommandSpec> {
        self.specs.get(command)
    }
    
    pub fn get_command_names(&self) -> Vec<&String> {
        self.specs.keys().collect()
    }
}

#[derive(Debug)]
pub struct CommandParser {
    registry: CommandRegistry,
    current_command: String,
    current_args: Vec<String>,
}

impl CommandParser {
    pub fn new() -> Self {
        CommandParser {
            registry: CommandRegistry::new(),
            current_command: String::new(),
            current_args: Vec::new(),
        }
    }
    
    pub fn clear(&mut self) {
        self.current_command.clear();
        self.current_args.clear();
    }
    
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
    
    fn start_command(&mut self, input: &str) -> ParseResult {
        let input_lower = input.to_lowercase();
        
        if let Some(spec) = self.registry.get_spec(&input_lower) {
            self.current_command = input_lower;
            
            if matches!(spec.arg_pattern, ArgumentPattern::None) {
                // Command executes immediately - clear state and return complete
                let command = self.current_command.clone();
                self.clear(); // ← FIX: Clear state for immediate commands!
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
    
    fn continue_building_command(&mut self, input: &str) -> ParseResult {
        let input_lower = input.to_lowercase();
        let new_command = format!("{}{}", self.current_command, input_lower);
        
        if let Some(spec) = self.registry.get_spec(&new_command) {
            self.current_command = new_command;
            
            if matches!(spec.arg_pattern, ArgumentPattern::None) {
                // Command executes immediately - clear state and return complete
                let command = self.current_command.clone();
                self.clear(); // ← FIX: Clear state for immediate commands here too!
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
    
    fn could_be_command_prefix(&self, prefix: &str) -> bool {
        self.registry.get_command_names().iter().any(|cmd| cmd.starts_with(prefix))
    }
    
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
    
    pub fn is_building(&self) -> bool {
        !self.current_command.is_empty()
    }
}

// ===== CALCULATOR =====

#[derive(Debug)]
pub struct HP41CCalculator {
    stack: Stack,
    input: InputState,
    programming: ProgrammingMode,
    display_formatter: DisplayFormatter,
    command_parser: CommandParser,
    storage_registers: [f64; NUM_STORAGE_REGISTERS],
    show_flags: bool,
}

impl HP41CCalculator {
    pub fn new() -> Self {
        HP41CCalculator {
            stack: Stack::new(),
            input: InputState::new(),
            programming: ProgrammingMode::new(),
            display_formatter: DisplayFormatter::new(),
            command_parser: CommandParser::new(),
            storage_registers: [0.0; NUM_STORAGE_REGISTERS],
            show_flags: false,
        }
    }

    pub fn execute_command(&mut self, command: &str, args: Option<Vec<String>>) -> Result<Option<String>, String> {
        execute_command(
            command,
            args,
            &mut self.stack,
            &mut self.input,
            &mut self.programming,
            &mut self.display_formatter,
            &mut self.storage_registers,
        ).map_err(|e| e.to_string())
    }

    /// CRITICAL: This processes ONE keystroke at a time!
    pub fn process_input(&mut self, key: &str) -> Result<Option<String>, String> {
        match key {
            ":" => self.toggle_programming_mode(),
            "F" => Ok(self.toggle_flags()),
            "\u{8}" | "\u{7f}" => self.handle_backspace(),
            "." | "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                self.handle_digit(key)
            }
            _ => self.handle_command_input(key),
        }
    }

    fn handle_command_input(&mut self, input: &str) -> Result<Option<String>, String> {
        match input {
            " " => {
                if self.command_parser.is_building() {
                    match self.command_parser.force_complete() {
                        ParseResult::Complete { command, args } => {
                            self.execute_command(&command, args)
                        }
                        ParseResult::Invalid(msg) => Err(msg),
                        ParseResult::Incomplete => Ok(None),
                    }
                } else {
                    Ok(None)
                }
            }
            
            "enter" => {
                if self.command_parser.is_building() {
                    match self.command_parser.force_complete() {
                        ParseResult::Complete { command, args } => {
                            self.execute_command(&command, args)
                        }
                        ParseResult::Invalid(msg) => Err(msg),
                        ParseResult::Incomplete => Ok(None),
                    }
                } else {
                    self.handle_enter()
                }
            }
            
            _ => {
                match self.command_parser.add_input(input) {
                    ParseResult::Complete { command, args } => {
                        self.execute_command(&command, args)
                    }
                    ParseResult::Invalid(msg) => Err(msg),
                    ParseResult::Incomplete => Ok(None),
                }
            }
        }
    }

    pub fn get_display(&self) -> String {
        let mut lines = Vec::with_capacity(8);
        
        self.add_stack_display(&mut lines);
        lines.push(self.build_status_line());
        lines.push(self.build_program_line());
        lines.push("sin cos tan asin acos atan log ln exp sqrt".to_string());
        lines.push(if self.show_flags {
            "pi inv arc  clx clr chs  +/-*^ ! ⌫  : lbl gto xeq sto rcl  F"
        } else {
            "pi inv arc  clx clr chs  +/-*^ ! ⌫  : fix sci eng sto rcl  F(flags)"
        }.to_string());
        
        lines.join("\n")
    }

    fn toggle_programming_mode(&mut self) -> Result<Option<String>, String> {
        let was_on = self.programming.is_programming;
        self.programming.toggle_programming_mode();
        Ok(Some(if was_on {
            "Programming mode OFF".to_string()
        } else {
            "Programming mode ON".to_string()
        }))
    }

    fn toggle_flags(&mut self) -> Option<String> {
        self.show_flags = !self.show_flags;
        None
    }

    fn handle_backspace(&mut self) -> Result<Option<String>, String> {
        if self.command_parser.is_building() {
            self.command_parser.clear();
        } else if self.input.is_entering() {
            if let Some(value) = self.input.handle_backspace() {
                self.stack.set_x(value);
            }
        }
        Ok(None)
    }

    fn handle_digit(&mut self, key: &str) -> Result<Option<String>, String> {
        if self.programming.is_programming && !self.command_parser.is_building() {
            self.programming.add_instruction(key, None, key);
            Ok(None)
        } else if self.command_parser.is_building() {
            self.handle_command_input(key)
        } else {
            if !self.input.is_entering() && self.stack.should_lift() {
                self.stack.lift();
            }
            let ch = key.chars().next().unwrap();
            match self.input.handle_digit(ch) {
                Ok(Some(value)) => {
                    self.stack.set_x(value);
                    self.stack.set_lift_flag(false);
                    Ok(None)
                }
                Ok(None) => Ok(None),
                Err(e) => Err(e.to_string()),
            }
        }
    }

    fn handle_enter(&mut self) -> Result<Option<String>, String> {
        self.execute_command("enter", None)
    }

    fn add_stack_display(&self, lines: &mut Vec<String>) {
        let registers = self.stack.get_registers();
        let names = ["T:", "Z:", "Y:", "X:"];
        
        for i in 0..4 {
            let value = registers[3 - i];
            let formatted = if i == 3 && self.input.is_entering() {
                self.input.get_display_string()
            } else {
                self.display_formatter.format_number(value, 35)
            };
            lines.push(format!("{} {:<35}", names[i], formatted));
        }
    }

    fn build_status_line(&self) -> String {
        let mut parts = vec![self.command_parser.get_current_state()];
        
        if self.show_flags {
            parts.push(format!("EN:{}", if self.input.is_entering() { 1 } else { 0 }));
            parts.push(format!("EEX:{}", if self.input.is_eex_mode() { 1 } else { 0 }));
            parts.push(format!("SL:{}", if self.stack.should_lift() { 1 } else { 0 }));
        }
        
        parts.push(self.display_formatter.get_mode_string());
        
        if self.programming.is_programming {
            parts.push("PRGM".to_string());
            parts.push(format!("L{:02}", self.programming.current_line));
        }
        
        parts.join(" ")
    }

    fn build_program_line(&self) -> String {
        if self.programming.is_programming {
            if let Some(instr) = self.programming.get_current_instruction() {
                format!(">{:02} {}", instr.line_number, instr)
            } else {
                format!(">{:02} _", self.programming.current_line)
            }
        } else if !self.programming.program.is_empty() {
            if let Some(instr) = self.programming.get_current_instruction() {
                format!(" {:02} {}", instr.line_number, instr)
            } else {
                format!(" {:02} END", self.programming.program_counter + 1)
            }
        } else {
            String::new()
        }
    }
}

impl Default for HP41CCalculator {
    fn default() -> Self {
        Self::new()
    }
}

// Test methods
#[cfg(test)]
impl HP41CCalculator {
    pub fn test_get_stack(&self) -> [f64; 4] {
        self.stack.get_registers()
    }
    
    pub fn test_get_storage(&self, register: usize) -> Option<f64> {
        self.storage_registers.get(register).copied()
    }
    
    pub fn test_get_command_buffer(&self) -> String {
        self.command_parser.get_current_state()
    }
    
    pub fn test_is_programming(&self) -> bool {
        self.programming.is_programming
    }
    
    pub fn test_get_program_counter(&self) -> usize {
        self.programming.program_counter
    }
    
    pub fn test_get_current_line(&self) -> i32 {
        self.programming.current_line
    }
    
    pub fn test_get_display_mode(&self) -> &DisplayMode {
        &self.display_formatter.mode
    }
    
    pub fn test_get_display_digits(&self) -> usize {
        self.display_formatter.digits
    }
    
    pub fn test_is_input_entering(&self) -> bool {
        self.input.is_entering()
    }
    
    pub fn test_get_program_length(&self) -> usize {
        self.programming.program.len()
    }
    
    pub fn test_set_x_register(&mut self, value: f64) {
        self.stack.set_x(value);
    }
    
    pub fn test_clear_command_buffer(&mut self) {
        self.command_parser.clear();
    }
    
    pub fn test_get_show_flags(&self) -> bool {
        self.show_flags
    }
    
    pub fn test_add_program_instruction(&mut self, cmd: &str, args: Option<Vec<String>>) {
        self.programming.add_instruction(cmd, args, cmd);
    }

    pub fn process_command_string(&mut self, cmd: &str) -> Result<Option<String>, String> {
        self.command_parser.clear();
        match self.command_parser.add_input(cmd) {
            ParseResult::Complete { command, args } => {
                self.execute_command(&command, args)
            }
            _ => {
                match self.command_parser.force_complete() {
                    ParseResult::Complete { command, args } => {
                        self.execute_command(&command, args)
                    }
                    ParseResult::Invalid(msg) => Err(msg),
                    ParseResult::Incomplete => Err("Command incomplete".to_string()),
                }
            }
        }
    }
}
