/// HP-41C Calculator Core
/// 
/// This module coordinates between all the calculator subsystems to provide
/// a complete HP-41C emulation. Unlike a Norwegian Blue HP-6S, this calculator
/// is definitely not pining for the fjords.

use crate::programming::ProgrammingMode;
use crate::display::DisplayFormatter;
#[cfg(test)]
use crate::display::DisplayMode;
use crate::commands::{CommandTrie, initialize_command_trie};
use crate::stack::Stack;
use crate::input::InputState;
use crate::execution::execute_command;

/// Maximum number of storage registers
const NUM_STORAGE_REGISTERS: usize = 100;

/// HP-41C Calculator State
#[derive(Debug)]
pub struct HP41CCalculator {
    // Core components
    stack: Stack,
    input: InputState,
    programming: ProgrammingMode,
    display_formatter: DisplayFormatter,
    
    // Command processing
    command_buffer: String,
    command_trie: CommandTrie,
    
    // Storage
    storage_registers: [f64; NUM_STORAGE_REGISTERS],
    
    // UI state
    show_flags: bool,
}

impl HP41CCalculator {
    /// Create a new calculator instance
    pub fn new() -> Self {
        HP41CCalculator {
            stack: Stack::new(),
            input: InputState::new(),
            programming: ProgrammingMode::new(),
            display_formatter: DisplayFormatter::new(),
            command_buffer: String::new(),
            command_trie: initialize_command_trie(),
            storage_registers: [0.0; NUM_STORAGE_REGISTERS],
            show_flags: false,
        }
    }

    /// Execute a command with the given arguments (for internal use)
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

    /// Process a single keystroke
    pub fn process_input(&mut self, key: &str) -> Result<Option<String>, String> {
        match key {
            // Special keys
            ":" => self.toggle_programming_mode(),
            "F" => Ok(self.toggle_flags()),
            "\u{8}" | "\u{7f}" => self.handle_backspace(),
            
            // Operators (may be commands or operations)
            "+" | "-" | "*" | "/" | "^" | "!" => self.handle_operator(key),
            
            // Numbers and decimal
            "." | "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => self.handle_digit(key),
            
            // Enter key
            "enter" => self.handle_enter(),
            
            // Space (completes commands)
            " " => self.handle_space(),
            
            // Letters (build commands)
            c if c.chars().all(|ch| ch.is_ascii_alphabetic()) => self.handle_letter(c),
            
            _ => Err(format!("Unknown key '{}'", key)),
        }
    }

    /// Get the current display (for UI)
    pub fn get_display(&self) -> String {
        let mut lines = Vec::with_capacity(8);
        
        // Stack display (4 lines)
        self.add_stack_display(&mut lines);
        
        // Status line
        lines.push(self.build_status_line());
        
        // Program line
        lines.push(self.build_program_line());
        
        // Command reference (2 lines)
        lines.push("sin cos tan asin acos atan log ln exp sqrt".to_string());
        lines.push(if self.show_flags {
            "pi inv arc  clx clr chs  +/-*^ ! ⌫  : lbl gto xeq sto rcl  F"
        } else {
            "pi inv arc  clx clr chs  +/-*^ ! ⌫  : fix sci eng sto rcl  F(flags)"
        }.to_string());
        
        lines.join("\n")
    }

    /// Process a complete command
    pub fn process_command(&mut self) -> Result<Option<String>, String> {
        if self.command_buffer.is_empty() {
            return Ok(None);
        }

        let buffer = self.command_buffer.clone();
        let parts: Vec<&str> = buffer.split_whitespace().collect();
        let command = parts[0];
        let args = if parts.len() > 1 {
            Some(parts[1..].iter().map(|&s| s.to_string()).collect())
        } else {
            None
        };

        self.command_buffer.clear();
        
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

    // === Private Implementation Details ===

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
        if !self.command_buffer.is_empty() {
            self.command_buffer.pop();
        } else if self.input.is_entering() {
            if let Some(value) = self.input.handle_backspace() {
                self.stack.set_x(value);
            }
        }
        Ok(None)
    }

    fn handle_operator(&mut self, op: &str) -> Result<Option<String>, String> {
        if self.programming.is_programming {
            self.programming.add_instruction(op, None, op);
            Ok(None)
        } else {
            self.process_input_as_command(op)
        }
    }

    fn handle_digit(&mut self, key: &str) -> Result<Option<String>, String> {
        if self.programming.is_programming && self.command_buffer.is_empty() {
            self.programming.add_instruction(key, None, key);
            Ok(None)
        } else if !self.command_buffer.is_empty() {
            self.command_buffer.push_str(key);
            self.check_auto_execute()
        } else {
            // Number entry
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
        if !self.command_buffer.is_empty() {
            self.process_command()
        } else {
            self.process_input_as_command("enter")
        }
    }

    fn handle_space(&mut self) -> Result<Option<String>, String> {
        if !self.command_buffer.is_empty() {
            // Don't execute if command buffer ends with space (waiting for arguments)
            if self.command_buffer.ends_with(' ') {
                Ok(None)
            } else {
                self.process_command()
            }
        } else {
            Ok(None)
        }
    }

    fn handle_letter(&mut self, letter: &str) -> Result<Option<String>, String> {
        self.command_buffer.push_str(&letter.to_lowercase());
        
        let (valid, complete, _) = self.command_trie.search(&self.command_buffer);
        
        if !valid {
            self.command_buffer.pop();
            Err("Invalid command".to_string())
        } else if complete && !self.needs_arguments(&self.command_buffer) {
            self.process_command()
        } else if self.needs_arguments(&self.command_buffer) {
            self.command_buffer.push(' ');
            Ok(None)
        } else {
            Ok(None)
        }
    }

    fn process_input_as_command(&mut self, cmd: &str) -> Result<Option<String>, String> {
        self.command_buffer = cmd.to_string();
        self.process_command()
    }

    fn needs_arguments(&self, cmd: &str) -> bool {
        matches!(cmd, "lbl" | "gto" | "xeq" | "fix" | "sci" | "eng" | "sto" | "rcl")
    }

    fn check_auto_execute(&mut self) -> Result<Option<String>, String> {
        let parts: Vec<&str> = self.command_buffer.split_whitespace().collect();
        if parts.len() >= 2 {
            match parts[0] {
                "fix" | "sci" | "eng" if parts[1].len() == 1 => self.process_command(),
                "sto" | "rcl" if parts[1].len() >= 1 => self.process_command(),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
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
        let mut parts = vec![self.build_command_display()];
        
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

    fn build_command_display(&self) -> String {
        let display = match self.command_buffer.as_str() {
            "fix " => "CMD: [fix _]",
            "sci " => "CMD: [sci _]",
            "eng " => "CMD: [eng _]",
            "sto " => "CMD: [sto __]",
            "rcl " => "CMD: [rcl __]",
            buffer if buffer.starts_with("sto ") || buffer.starts_with("rcl ") => {
                let parts: Vec<&str> = buffer.split_whitespace().collect();
                if parts.len() == 2 && parts[1].len() == 1 {
                    return format!("CMD: [{} {}_]", parts[0], parts[1]);
                }
                return format!("CMD: [{}]", buffer);
            }
            buffer => return format!("CMD: [{}]", buffer),
        };
        display.to_string()
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

// Provide read-only access for tests
#[cfg(test)]
impl HP41CCalculator {
    pub fn test_get_stack(&self) -> [f64; 4] {
        self.stack.get_registers()
    }
    
    pub fn test_get_storage(&self, register: usize) -> Option<f64> {
        self.storage_registers.get(register).copied()
    }
    
    pub fn test_get_command_buffer(&self) -> &str {
        &self.command_buffer
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
        self.command_buffer.clear();
    }
    
    pub fn test_get_show_flags(&self) -> bool {
        self.show_flags
    }
    
    pub fn test_add_program_instruction(&mut self, cmd: &str, args: Option<Vec<String>>) {
        self.programming.add_instruction(cmd, args, cmd);
    }

    pub fn process_command_string(&mut self, cmd: &str) -> Result<Option<String>, String> {
        self.command_buffer = cmd.to_string();
        self.process_command()
    }
}