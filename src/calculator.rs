/// HP-41C Calculator Core - Clean and Focused with Comprehensive Logging
/// 
/// This module contains the main HP41CCalculator that coordinates between
/// all calculator subsystems. The command system has been moved to separate
/// modules for better organization. Now includes integrated logging for debugging.

use crate::programming::ProgrammingMode;
use crate::display::DisplayFormatter;
#[cfg(test)]
use crate::display::DisplayMode;
use crate::stack::Stack;
use crate::input::InputState;
use crate::execution::execute_command;
use crate::parser::{CommandParser, ParseResult};
use crate::logger::Logger;  // NEW: Import logger

/// Maximum number of storage registers
const NUM_STORAGE_REGISTERS: usize = 100;

/// HP-41C Calculator State with Integrated Logging
/// 
/// ## Keystroke-by-Keystroke Processing
/// 
/// This calculator processes each individual keystroke immediately, just like
/// the original HP-41C hardware:
/// 
/// - User presses 'f' → process single keystroke → update display
/// - User presses 'i' → process single keystroke → update display  
/// - User presses 'x' → process single keystroke → "fix" command ready for argument
/// - User presses '4' → process single keystroke → "fix 4" executes immediately
/// 
/// Each call to `process_input(key)` receives exactly ONE character/keystroke.
/// Now includes comprehensive logging for debugging calculator behavior.
#[derive(Debug)]
pub struct HP41CCalculator {
    // Core components
    stack: Stack,
    input: InputState,
    programming: ProgrammingMode,
    display_formatter: DisplayFormatter,
    
    // Command processing
    command_parser: CommandParser,
    
    // Storage
    storage_registers: [f64; NUM_STORAGE_REGISTERS],
    
    // UI state
    show_flags: bool,
    
    // NEW: Integrated logger
    logger: Logger,
}

impl HP41CCalculator {
    /// Create a new calculator instance
    pub fn new() -> Self {
        HP41CCalculator {
            stack: Stack::new(),
            input: InputState::new(),
            programming: ProgrammingMode::new(),
            display_formatter: DisplayFormatter::new(),
            command_parser: CommandParser::new(),
            storage_registers: [0.0; NUM_STORAGE_REGISTERS],
            show_flags: false,
            logger: Logger::new(),  // Default: minimal logging
        }
    }
    
    /// Create a calculator with debug logging enabled
    pub fn new_with_debug_logging() -> Self {
        let mut calc = Self::new();
        calc.logger = Logger::debug_all();
        calc.logger.log_debug("INIT", "Calculator created with debug logging enabled");
        calc
    }
    
    /// Get mutable reference to logger for configuration
    pub fn logger_mut(&mut self) -> &mut Logger {
        &mut self.logger
    }
    
    /// Get reference to logger for status
    pub fn logger(&self) -> &Logger {
        &self.logger
    }

    /// Execute a command with the given arguments (for internal use)
    pub fn execute_command(&mut self, command: &str, args: Option<Vec<String>>) -> Result<Option<String>, String> {
        // Log command execution attempt
        self.logger.log_command_execution(command, &args, "starting");
        
        // Capture stack state before execution
        let stack_before = self.stack.get_registers();
        
        let result = execute_command(
            command,
            args.clone(),
            &mut self.stack,
            &mut self.input,
            &mut self.programming,
            &mut self.display_formatter,
            &mut self.storage_registers,
        ).map_err(|e| e.to_string());
        
        // Log the result and any stack changes
        match &result {
            Ok(Some(msg)) => {
                self.logger.log_command_execution(command, &args, msg);
            }
            Ok(None) => {
                self.logger.log_command_execution(command, &args, "completed");
            }
            Err(e) => {
                self.logger.log_command_execution(command, &args, &format!("ERROR: {}", e));
            }
        }
        
        // Log stack changes if they occurred
        let stack_after = self.stack.get_registers();
        if stack_before != stack_after {
            self.logger.log_stack_operation(&format!("{} command", command), &stack_before, &stack_after);
        }
        
        result
    }

    /// Process a single keystroke with comprehensive logging
    /// 
    /// ## CRITICAL: Single Keystroke Processing
    /// 
    /// This method processes exactly ONE keystroke at a time, just like the original HP-41C:
    /// 
    /// - `process_input("f")` - User pressed the 'f' key
    /// - `process_input("i")` - User pressed the 'i' key  
    /// - `process_input("x")` - User pressed the 'x' key → "fix" command ready
    /// - `process_input("4")` - User pressed the '4' key → "fix 4" executes
    /// 
    /// NOT like a command line where you'd call `process_input("fix 4")` all at once.
    pub fn process_input(&mut self, key: &str) -> Result<Option<String>, String> {
        // Log every keystroke
        self.logger.log_keystroke(key);
        
        // Log current state before processing
        self.log_current_state("before processing");
        
        let result = match key {
            // Special keys that bypass command parsing
            ":" => self.toggle_programming_mode(),
            "F" => Ok(self.toggle_flags()),
            "\u{8}" | "\u{7f}" => self.handle_backspace(),
            
            // Numbers and decimal go to number entry
            "." | "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => {
                self.handle_digit(key)
            }
            
            // Everything else goes through unified command processing
            _ => self.handle_command_input(key),
        };
        
        // Log state after processing
        self.log_current_state("after processing");
        
        result
    }
    
    /// Log current calculator state (helper method)
    fn log_current_state(&self, context: &str) {
        self.logger.log_stack_state(&self.stack.get_registers(), context);
        self.logger.log_input_state(
            self.input.is_entering(), 
            self.input.is_eex_mode(), 
            &self.input.get_display_string()
        );
        self.logger.log_command_state(&self.command_parser.get_current_state(), context);
    }

    /// Handle command input using the unified parser
    fn handle_command_input(&mut self, input: &str) -> Result<Option<String>, String> {
        self.logger.log_command_state(&self.command_parser.get_current_state(), "before input");
        
        match input {
            " " => {
                // Space forces manual completion or acts as argument separator
                if self.command_parser.is_building() {
                    self.logger.log_debug("PARSER", "Space pressed - forcing completion");
                    match self.command_parser.force_complete() {
                        ParseResult::Complete { command, args } => {
                            self.execute_command(&command, args)
                        }
                        ParseResult::Invalid(msg) => Err(msg),
                        ParseResult::Incomplete => Ok(None),
                    }
                } else {
                    self.logger.log_debug("PARSER", "Space pressed with no command - ignored");
                    Ok(None) // Space with no command does nothing
                }
            }
            
            "enter" => {
                // Enter can either complete a command or do ENTER operation
                if self.command_parser.is_building() {
                    self.logger.log_debug("PARSER", "Enter pressed - forcing command completion");
                    match self.command_parser.force_complete() {
                        ParseResult::Complete { command, args } => {
                            self.execute_command(&command, args)
                        }
                        ParseResult::Invalid(msg) => Err(msg),
                        ParseResult::Incomplete => Ok(None),
                    }
                } else {
                    // No command building, do regular ENTER
                    self.logger.log_debug("STACK", "Enter pressed - performing ENTER operation");
                    self.handle_enter()
                }
            }
            
            _ => {
                // All other input goes to the command parser
                match self.command_parser.add_input(input) {
                    ParseResult::Complete { command, args } => {
                        self.logger.log_debug("PARSER", &format!("Command completed: {} {:?}", command, args));
                        self.execute_command(&command, args)
                    }
                    ParseResult::Invalid(msg) => {
                        self.logger.log_debug("PARSER", &format!("Invalid input: {}", msg));
                        Err(msg)
                    }
                    ParseResult::Incomplete => {
                        self.logger.log_debug("PARSER", "Command building continues");
                        Ok(None) // Keep building
                    }
                }
            }
        }
    }

    /// Get the current display (for UI)
    pub fn get_display(&self) -> String {
        let mut lines = Vec::with_capacity(8);
        
        // Stack display (4 lines)
        self.add_stack_display(&mut lines);
        
        // Status line (now includes logging status)
        lines.push(self.build_status_line());
        
        // Program line
        lines.push(self.build_program_line());
        
        // Command reference (2 lines)
        lines.push("sin cos tan asin acos atan log ln exp sqrt".to_string());
        let cmd_line = if self.show_flags {
            "pi inv arc  clx clr chs  +/-*^ ! ⌫  : lbl gto xeq sto rcl  F L"
        } else {
            "pi inv arc  clx clr chs  +/-*^ ! ⌫  : fix sci eng sto rcl  F L(log)"
        };
        lines.push(cmd_line.to_string());
        
        lines.join("\n")
    }

    // === Private Implementation Details ===

    fn toggle_programming_mode(&mut self) -> Result<Option<String>, String> {
        let was_on = self.programming.is_programming;
        self.programming.toggle_programming_mode();
        
        // Log the flag change
        self.logger.log_flag_change("programming_mode", was_on, self.programming.is_programming);
        self.logger.log_programming("mode_toggle", 
            &format!("Programming mode {}", if self.programming.is_programming { "ON" } else { "OFF" }));
        
        Ok(Some(if was_on {
            "Programming mode OFF".to_string()
        } else {
            "Programming mode ON".to_string()
        }))
    }

    fn toggle_flags(&mut self) -> Option<String> {
        let old_value = self.show_flags;
        self.show_flags = !self.show_flags;
        
        // Log the flag change
        self.logger.log_flag_change("show_flags", old_value, self.show_flags);
        
        None
    }

    fn handle_backspace(&mut self) -> Result<Option<String>, String> {
        self.logger.log_debug("INPUT", "Backspace pressed");
        
        if self.command_parser.is_building() {
            self.logger.log_debug("PARSER", "Clearing command buffer");
            // TODO: Add backspace support to command parser
            self.command_parser.clear();
        } else if self.input.is_entering() {
            self.logger.log_debug("INPUT", "Handling backspace during number entry");
            let stack_before = self.stack.get_registers();
            if let Some(value) = self.input.handle_backspace() {
                self.stack.set_x(value);
                let stack_after = self.stack.get_registers();
                if stack_before != stack_after {
                    self.logger.log_stack_operation("backspace", &stack_before, &stack_after);
                }
            }
        }
        Ok(None)
    }

    fn handle_digit(&mut self, key: &str) -> Result<Option<String>, String> {
        if self.programming.is_programming && !self.command_parser.is_building() {
            self.logger.log_programming("digit_entry", &format!("Adding digit '{}' to program", key));
            self.programming.add_instruction(key, None, key);
            Ok(None)
        } else if self.command_parser.is_building() {
            // Digit might be an argument to a command
            self.logger.log_debug("PARSER", &format!("Adding digit '{}' as command argument", key));
            self.handle_command_input(key)
        } else {
            // Regular number entry
            self.logger.log_debug("INPUT", &format!("Number entry: digit '{}'", key));
            
            let stack_before = self.stack.get_registers();
            let should_lift_before = self.stack.should_lift();
            
            if !self.input.is_entering() && self.stack.should_lift() {
                self.logger.log_debug("STACK", "Lifting stack for new number entry");
                self.stack.lift();
            }
            let ch = key.chars().next().unwrap();
            match self.input.handle_digit(ch) {
                Ok(Some(value)) => {
                    self.stack.set_x(value);
                    self.stack.set_lift_flag(false);
                    
                    let stack_after = self.stack.get_registers();
                    if stack_before != stack_after || should_lift_before != self.stack.should_lift() {
                        self.logger.log_stack_operation("digit_entry", &stack_before, &stack_after);
                        self.logger.log_flag_change("stack_lift", should_lift_before, self.stack.should_lift());
                    }
                    
                    Ok(None)
                }
                Ok(None) => Ok(None),
                Err(e) => Err(e.to_string()),
            }
        }
    }

    fn handle_enter(&mut self) -> Result<Option<String>, String> {
        self.logger.log_debug("STACK", "ENTER operation");
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
        
        // Add logging status (compact format)
        parts.push(self.logger.get_config_string());
        
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
    
    /// NEW: Toggle logging on key press 'L'
    pub fn toggle_logging(&mut self) -> Option<String> {
        let was_enabled = self.logger.enabled;
        let now_enabled = self.logger.toggle_enabled();
        
        Some(format!("Logging {}", if now_enabled { "ON" } else { "OFF" }))
    }
    
    /// NEW: Configure logger with preset configurations
    pub fn configure_logger(&mut self, preset: &str) -> Option<String> {
        match preset {
            "all" => {
                self.logger = Logger::debug_all();
                Some("Debug logging: ALL enabled".to_string())
            }
            "minimal" => {
                self.logger = Logger::minimal();
                Some("Debug logging: MINIMAL (flags + stack)".to_string())
            }
            "off" => {
                self.logger = Logger::new();
                self.logger.enabled = false;
                Some("Debug logging: DISABLED".to_string())
            }
            _ => Some("Unknown logging preset".to_string())
        }
    }
}

impl Default for HP41CCalculator {
    fn default() -> Self {
        Self::new()
    }
}

// Test-only methods
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
        let stack_before = self.stack.get_registers();
        self.stack.set_x(value);
        let stack_after = self.stack.get_registers();
        self.logger.log_stack_operation("test_set_x", &stack_before, &stack_after);
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
    
    /// NEW: Test helper to enable debug logging
    pub fn test_enable_debug_logging(&mut self) {
        self.logger = Logger::debug_all();
    }
    
    /// NEW: Test helper to configure specific logging
    pub fn test_configure_logging(&mut self, flags: bool, stack: bool, input: bool, commands: bool) {
        self.logger.set_flags(flags, stack, input, commands);
    }
}