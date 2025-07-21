use crate::programming::ProgrammingMode;
use crate::display::{DisplayMode, DisplayFormatter};
use crate::commands::{CommandTrie, initialize_command_trie};

#[derive(Debug)]
pub struct HP41CCalculator {
    pub stack: [f64; 4],
    pub stack_lifted: bool,
    pub display_value: f64,
    pub entering_number: bool,
    pub number_entry_string: String,
    pub command_buffer: String,
    pub eex_mode: bool,
    pub eex_digits: String,
    pub arc_mode: bool,
    pub programming: ProgrammingMode,
    pub command_trie: CommandTrie,
    pub show_flags: bool,
    pub display_formatter: DisplayFormatter,
    pub storage_registers: [f64; 100],
}

impl HP41CCalculator {
    pub fn new() -> Self {
        HP41CCalculator {
            stack: [0.0; 4],
            stack_lifted: false,
            display_value: 0.0,
            entering_number: false,
            number_entry_string: String::new(),
            command_buffer: String::new(),
            eex_mode: false,
            eex_digits: String::new(),
            arc_mode: false,
            programming: ProgrammingMode::new(),
            command_trie: initialize_command_trie(),
            show_flags: false,
            display_formatter: DisplayFormatter::new(),
            storage_registers: [0.0; 100],
        }
    }

    pub fn lift_stack(&mut self) {
        self.stack[3] = self.stack[2]; // T = Z
        self.stack[2] = self.stack[1]; // Z = Y
        self.stack[1] = self.stack[0]; // Y = X
    }

    pub fn binary_operation(&mut self, operation: &str) -> Result<(), String> {
        let y_val = self.stack[1];
        let x_val = self.stack[0];

        let result = match operation {
            "+" => y_val + x_val,
            "-" => y_val - x_val,
            "*" => y_val * x_val,
            "/" => {
                if x_val == 0.0 {
                    return Err("Division by zero".to_string());
                }
                y_val / x_val
            }
            "^" => y_val.powf(x_val),
            _ => return Err(format!("Unknown operation {}", operation)),
        };

        // Drop stack and put result in X
        // HP behavior: stack drops, T gets copy of original T (not 0)
        self.stack[0] = result;
        self.stack[1] = self.stack[2]; // Y = Z
        self.stack[2] = self.stack[3]; // Z = T
        self.stack[3] = self.stack[3]; // T = T (stays the same, gets duplicated)

        self.display_value = result;
        self.stack_lifted = true;
        self.entering_number = false;
        self.number_entry_string.clear(); // Clear any number entry
        Ok(())
    }

    pub fn execute_command(&mut self, command: &str, args: Option<Vec<String>>) -> Result<Option<String>, String> {
        let args = args.unwrap_or_default();
        let command = command.to_lowercase();

        match command.as_str() {
            "lbl" => {
                if self.programming.is_programming && !args.is_empty() {
                    self.programming.add_instruction("LBL", Some(args.clone()), &format!("LBL {}", args[0]));
                    Ok(None)
                } else if args.is_empty() {
                    Err("LBL requires argument".to_string())
                } else {
                    Ok(None)
                }
            }
            "gto" => {
                if !args.is_empty() {
                    if self.programming.goto_label(&args[0]) {
                        Ok(None)
                    } else {
                        Err(format!("Label {} not found", args[0]))
                    }
                } else {
                    Err("GTO requires argument".to_string())
                }
            }
            "xeq" => {
                if !args.is_empty() {
                    if self.programming.execute_subroutine(&args[0]) {
                        self.programming.is_running = true;
                        Ok(None)
                    } else {
                        Err(format!("Label {} not found", args[0]))
                    }
                } else {
                    Err("XEQ requires argument".to_string())
                }
            }
            "rtn" => {
                if self.programming.is_programming {
                    self.programming.add_instruction("RTN", None, "RTN");
                } else {
                    self.programming.return_from_subroutine();
                }
                Ok(None)
            }
            "sst" => {
                Ok(Some("SST not implemented".to_string()))
            }
            "bst" => {
                Ok(Some("BST not implemented".to_string()))
            }
            "prgm" => {
                self.programming.clear_program();
                Ok(Some("Program cleared".to_string()))
            }
            _ => {
                // If in programming mode, record most commands
                if self.programming.is_programming && 
                   !matches!(command.as_str(), "gto" | "xeq" | "sst" | "bst" | "prgm") {
                    self.programming.add_instruction(&command.to_uppercase(), Some(args), &self.command_buffer);
                    return Ok(None);
                }

                // Mathematical and stack operations
                match command.as_str() {
                    "enter" => {
                        self.lift_stack();
                        self.stack_lifted = false; // Key: ENTER doesn't set stack_lifted!
                        self.entering_number = false;
                        self.number_entry_string.clear(); // Clear the entry string
                        Ok(None)
                    }
                    "swap" => {
                        self.stack.swap(0, 1);
                        self.display_value = self.stack[0];
                        Ok(None)
                    }
                    "clx" => {
                        self.stack[0] = 0.0;
                        self.display_value = 0.0;
                        self.entering_number = false;
                        Ok(None)
                    }
                    "clr" => {
                        self.stack = [0.0; 4];
                        self.display_value = 0.0;
                        self.entering_number = false;
                        Ok(None)
                    }
                    "chs" => {
                        self.stack[0] = -self.stack[0];
                        self.display_value = self.stack[0];
                        Ok(None)
                    }
                    "arc" => {
                        self.arc_mode = true;
                        Ok(None)
                    }
                    "eex" => {
                        self.eex_mode = true;
                        self.eex_digits.clear();
                        Ok(None)
                    }
                    "fix" => {
                        if !args.is_empty() {
                            if let Ok(digits) = args[0].parse::<usize>() {
                                if digits <= 9 {
                                    self.display_formatter.mode = DisplayMode::Fix;
                                    self.display_formatter.digits = digits;
                                    Ok(Some(format!("FIX {}", digits)))
                                } else {
                                    Err("FIX digits must be 0-9".to_string())
                                }
                            } else {
                                Err("FIX requires numeric argument".to_string())
                            }
                        } else {
                            Err("FIX requires argument (0-9)".to_string())
                        }
                    }
                    "sci" => {
                        if !args.is_empty() {
                            if let Ok(digits) = args[0].parse::<usize>() {
                                if digits <= 9 {
                                    self.display_formatter.mode = DisplayMode::Sci;
                                    self.display_formatter.digits = digits;
                                    Ok(Some(format!("SCI {}", digits)))
                                } else {
                                    Err("SCI digits must be 0-9".to_string())
                                }
                            } else {
                                Err("SCI requires numeric argument".to_string())
                            }
                        } else {
                            Err("SCI requires argument (0-9)".to_string())
                        }
                    }
                    "eng" => {
                        if !args.is_empty() {
                            if let Ok(digits) = args[0].parse::<usize>() {
                                if digits <= 9 {
                                    self.display_formatter.mode = DisplayMode::Eng;
                                    self.display_formatter.digits = digits;
                                    Ok(Some(format!("ENG {}", digits)))
                                } else {
                                    Err("ENG digits must be 0-9".to_string())
                                }
                            } else {
                                Err("ENG requires numeric argument".to_string())
                            }
                        } else {
                            Err("ENG requires argument (0-9)".to_string())
                        }
                    }
                    "sto" => {
                        if !args.is_empty() {
                            if let Ok(register) = args[0].parse::<usize>() {
                                if register < 100 {
                                    self.storage_registers[register] = self.stack[0];
                                    Ok(Some(format!("STO {}", register)))
                                } else {
                                    Err("Register must be 00-99".to_string())
                                }
                            } else {
                                Err("Invalid register number".to_string())
                            }
                        } else {
                            Err("STO requires register number".to_string())
                        }
                    }
                    "rcl" => {
                        if !args.is_empty() {
                            if let Ok(register) = args[0].parse::<usize>() {
                                if register < 100 {
                                    if self.stack_lifted {
                                        self.lift_stack();
                                    }
                                    self.stack[0] = self.storage_registers[register];
                                    self.display_value = self.stack[0];
                                    self.stack_lifted = true;
                                    Ok(Some(format!("RCL {}", register)))
                                } else {
                                    Err("Register must be 00-99".to_string())
                                }
                            } else {
                                Err("Invalid register number".to_string())
                            }
                        } else {
                            Err("RCL requires register number".to_string())
                        }
                    }
                    // Unary mathematical functions
                    cmd @ ("sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "log" | "ln" | "exp" | "sqrt" | "inv") => {
                        let x_val = self.stack[0];
                        
                        let mut actual_cmd = cmd;
                        if self.arc_mode && matches!(cmd, "sin" | "cos" | "tan") {
                            actual_cmd = match cmd {
                                "sin" => "asin",
                                "cos" => "acos", 
                                "tan" => "atan",
                                _ => cmd,
                            };
                            self.arc_mode = false;
                        }

                        let result = match actual_cmd {
                            "sin" => x_val.sin(),
                            "cos" => x_val.cos(),
                            "tan" => x_val.tan(),
                            "asin" => x_val.asin(),
                            "acos" => x_val.acos(),
                            "atan" => x_val.atan(),
                            "log" => x_val.log10(),
                            "ln" => x_val.ln(),
                            "exp" => x_val.exp(),
                            "sqrt" => x_val.sqrt(),
                            "inv" => 1.0 / x_val,
                            _ => return Err("Unknown function".to_string()),
                        };

                        if result.is_nan() || result.is_infinite() {
                            return Err("Math error".to_string());
                        }

                        self.stack[0] = result;
                        self.display_value = self.stack[0];
                        self.stack_lifted = true;
                        Ok(None)
                    }
                    "pi" => {
                        // Set PI value directly
                        if self.stack_lifted {
                            self.lift_stack();
                        }
                        self.stack[0] = std::f64::consts::PI;
                        self.display_value = self.stack[0];
                        self.stack_lifted = true;
                        self.entering_number = false;
                        Ok(None)
                    }
                    "pow" => {
                        self.binary_operation("^")?;
                        Ok(None)
                    }
                    _ => Err(format!("Unknown command '{}'", command)),
                }
            }
        }
    }

    pub fn process_input(&mut self, key: &str) -> Result<Option<String>, String> {
        match key {
            ":" => {
                let was_programming = self.programming.is_programming;
                self.programming.toggle_programming_mode();
                if was_programming {
                    Ok(Some("Programming mode OFF".to_string()))
                } else {
                    Ok(Some("Programming mode ON".to_string()))
                }
            }
            "R/S" => {
                self.programming.is_running = !self.programming.is_running;
                if self.programming.is_running {
                    Ok(Some("Running...".to_string()))
                } else {
                    Ok(Some("Stopped".to_string()))
                }
            }
            key if key == "\u{8}" || key == "\u{7f}" => { // Backspace or Delete
                if !self.command_buffer.is_empty() {
                    self.command_buffer.pop();
                    Ok(None)
                } else if self.entering_number {
                    self.handle_number_backspace();
                    Ok(None)
                } else {
                    Ok(None)
                }
            }
            // In programming mode, store most operations instead of executing them
            key if matches!(key, "+" | "-" | "*" | "/" | "^") => {
                if self.programming.is_programming {
                    self.programming.add_instruction(key, None, key);
                    Ok(None)
                } else {
                    self.binary_operation(key)?;
                    Ok(None)
                }
            }
            "!" => {
                if self.programming.is_programming {
                    self.programming.add_instruction("!", None, "!");
                    Ok(None)
                } else {
                    let x_val = self.stack[0];
                    if x_val < 0.0 || x_val > 170.0 {
                        return Err("Factorial domain error".to_string());
                    }
                    let result = gamma(x_val + 1.0);
                    self.stack[0] = result;
                    self.display_value = self.stack[0];
                    self.stack_lifted = true;
                    Ok(None)
                }
            }
            // Handle digits and decimal point FIRST, before command processing
            key if key.len() == 1 && (key.chars().next().unwrap().is_ascii_digit() || key == ".") => {
                if self.programming.is_programming && self.command_buffer.is_empty() {
                    // In programming mode with no command buffer, store digits as program steps
                    self.programming.add_instruction(key, None, key);
                    Ok(None)
                } else if !self.command_buffer.is_empty() {
                    // If we have a command buffer, add digits to it (for arguments like "LBL 01")
                    self.command_buffer.push_str(key);
                    
                    // Check if this completes a command with arguments
                    let parts: Vec<&str> = self.command_buffer.trim().split_whitespace().collect();
                    if parts.len() >= 2 {
                        let command = parts[0];
                        // For FIX, SCI, ENG - execute immediately when we get the digit
                        if matches!(command, "fix" | "sci" | "eng") && parts.len() == 2 {
                            return self.process_command();
                        }
                        // For STO, RCL - execute immediately when we get 2 digits
                        if matches!(command, "sto" | "rcl") && parts.len() == 2 && parts[1].len() == 2 {
                            return self.process_command();
                        }
                    }
                    Ok(None)
                } else {
                    // Normal calculator mode - process digits for number input
                    self.handle_digit_input(key)?;
                    Ok(None)
                }
            }
            // Handle enter command specifically
            "enter" => {
                if self.programming.is_programming {
                    self.programming.add_instruction("ENTER", None, "ENTER");
                    Ok(None)
                } else {
                    self.lift_stack();
                    self.stack_lifted = false; // Key: ENTER doesn't set stack_lifted!
                    self.entering_number = false;
                    self.number_entry_string.clear(); // Clear the entry string
                    Ok(None)
                }
            }
            // Handle other commands
            key if key.chars().all(|c| c.is_ascii_alphabetic()) || key == " " => {
                if key == " " {
                    if !self.command_buffer.is_empty() {
                        return self.process_command();
                    }
                    Ok(None)
                } else {
                    self.command_buffer.push_str(&key.to_lowercase());
                    
                    let (is_valid, is_complete, _) = self.command_trie.search(&self.command_buffer);
                    
                    if !is_valid {
                        self.command_buffer.pop();
                        Err("Invalid command".to_string())
                    } else if is_complete && !matches!(self.command_buffer.as_str(), "lbl" | "gto" | "xeq" | "fix" | "sci" | "eng" | "sto" | "rcl") {
                        // Command is complete and doesn't need arguments - execute immediately
                        self.process_command()
                    } else if matches!(self.command_buffer.as_str(), "fix" | "sci" | "eng" | "sto" | "rcl") {
                        // These commands need arguments - add a space automatically
                        self.command_buffer.push(' ');
                        Ok(None)
                    } else {
                        // Command needs arguments or isn't complete yet
                        Ok(None)
                    }
                }
            }
            _ => Err(format!("Unknown key '{}'", key)),
        }
    }

    fn handle_number_backspace(&mut self) {
        if self.eex_mode {
            if !self.eex_digits.is_empty() {
                self.eex_digits.pop();
            } else {
                self.eex_mode = false;
            }
        } else if !self.number_entry_string.is_empty() {
            self.number_entry_string.pop();
            if self.number_entry_string.is_empty() {
                self.display_value = 0.0;
                self.stack[0] = 0.0;
                self.entering_number = false;
            } else {
                // Re-parse the remaining string
                if let Ok(value) = self.number_entry_string.parse::<f64>() {
                    self.display_value = value;
                    self.stack[0] = self.display_value;
                } else {
                    self.display_value = 0.0;
                    self.stack[0] = 0.0;
                    self.entering_number = false;
                }
            }
        } else {
            self.display_value = 0.0;
            self.stack[0] = 0.0;
            self.entering_number = false;
        }
    }

    fn handle_digit_input(&mut self, key: &str) -> Result<(), String> {
        if !self.entering_number {
            if self.stack_lifted {
                self.lift_stack();
            }
            self.stack_lifted = false;
            self.number_entry_string.clear();
            self.entering_number = true;
        }

        if self.eex_mode {
            if key.chars().all(|c| c.is_ascii_digit()) {
                self.eex_digits.push_str(key);
                // For EEX mode, we'll still need to format properly
                let display_string = &self.number_entry_string;
                let mantissa_str = if display_string.is_empty() { "0" } else { display_string };
                let new_display = format!("{}E{}", mantissa_str, self.eex_digits);
                if let Ok(value) = new_display.parse::<f64>() {
                    self.display_value = value;
                    self.stack[0] = self.display_value;
                }
            }
        } else {
            if key == "." && !self.number_entry_string.contains('.') {
                self.number_entry_string.push('.');
            } else if key.chars().all(|c| c.is_ascii_digit()) {
                if self.number_entry_string == "0" {
                    self.number_entry_string = key.to_string();
                } else {
                    self.number_entry_string.push_str(key);
                }
            } else {
                return Ok(());
            }

            // Parse and store the value, but don't change the display string
            match self.number_entry_string.parse::<f64>() {
                Ok(value) => {
                    self.display_value = value;
                    self.stack[0] = self.display_value;
                }
                Err(_) => return Err("Invalid number".to_string()),
            }
        }

        Ok(())
    }

    pub fn process_command(&mut self) -> Result<Option<String>, String> {
        if self.command_buffer.is_empty() {
            return Ok(None);
        }

        // Clone the command buffer to avoid borrowing issues
        let command_buffer = self.command_buffer.clone();
        let parts: Vec<&str> = command_buffer.trim().split_whitespace().collect();
        let command = parts[0];
        let args = if parts.len() > 1 {
            Some(parts[1..].iter().map(|s| s.to_string()).collect())
        } else {
            None
        };

        let (_, is_complete, _) = self.command_trie.search(command);
        if !is_complete {
            self.command_buffer.clear();
            return Err(format!("Unknown command '{}'", command));
        }

        let result = self.execute_command(command, args)?;
        self.command_buffer.clear();
        self.entering_number = false;

        Ok(result)
    }

    pub fn get_display(&self) -> String {
        let mut lines = Vec::new();

        // Display 4-level stack (T, Z, Y, X)
        let register_names = ["T:", "Z:", "Y:", "X:"];
        for i in 0..4 {
            let value = self.stack[3 - i]; // T=3, Z=2, Y=1, X=0
            let register_index = 3 - i;  // Actual register index: T=3, Z=2, Y=1, X=0
            let formatted_value = self.format_number_for_register(value, register_index, 35);
            lines.push(format!("{} {:<35}", register_names[i], formatted_value));
        }

        // Status line
        let mut status_parts = Vec::new();
        
        // Special handling for commands waiting for arguments
        let cmd_display = if matches!(self.command_buffer.as_str(), "fix" | "sci" | "eng") {
            format!("CMD: [{} _]", self.command_buffer)
        } else if matches!(self.command_buffer.as_str(), "sto" | "rcl") {
            format!("CMD: [{} __]", self.command_buffer)
        } else if self.command_buffer.starts_with("sto ") || self.command_buffer.starts_with("rcl ") {
            let parts: Vec<&str> = self.command_buffer.split_whitespace().collect();
            if parts.len() == 2 && parts[1].len() == 1 {
                // Show one digit and underscore for second digit
                format!("CMD: [{} {}_]", parts[0], parts[1])
            } else {
                format!("CMD: [{}]", self.command_buffer)
            }
        } else {
            format!("CMD: [{}]", self.command_buffer)
        };
        status_parts.push(cmd_display);
        
        if self.show_flags {
            status_parts.push(format!("EN:{}", if self.entering_number { 1 } else { 0 }));
            status_parts.push(format!("EEX:{}", if self.eex_mode { 1 } else { 0 }));
            status_parts.push(format!("SL:{}", if self.stack_lifted { 1 } else { 0 }));
            if self.arc_mode {
                status_parts.push("ARC".to_string());
            }
        }

        // Always show display mode
        status_parts.push(self.display_formatter.get_mode_string());
        
        if self.programming.is_programming {
            status_parts.push("PRGM".to_string());
            status_parts.push(format!("L{:02}", self.programming.current_line));
        }
        if self.programming.is_running {
            status_parts.push("RUN".to_string());
            if self.programming.get_current_instruction().is_some() {
                status_parts.push(format!("PC{:02}", self.programming.program_counter));
            }
        }

        lines.push(status_parts.join(" "));

        // Program display line
        if self.programming.is_programming {
            if let Some(current_instr) = self.programming.get_current_instruction() {
                lines.push(format!(">{:02} {}", current_instr.line_number, current_instr));
            } else {
                lines.push(format!(">{:02} _", self.programming.current_line));
            }
        } else {
            lines.push("".to_string());
        }

        // Command reference lines
        lines.push("sin cos tan asin acos atan log ln exp sqrt".to_string());
        if self.show_flags {
            lines.push("pi inv arc  clx clr chs  +/-*^ ! ⌫  : lbl gto xeq sto rcl  F".to_string());
        } else {
            lines.push("pi inv arc  clx clr chs  +/-*^ ! ⌫  : fix sci eng sto rcl  F(flags)".to_string());
        }

        // Pad to exactly 8 rows
        while lines.len() < 8 {
            lines.push("".to_string());
        }

        lines.into_iter().take(8).collect::<Vec<_>>().join("\n")
    }

    fn format_number_for_register(&self, value: f64, register_index: usize, width: usize) -> String {
        // Only show raw input string for X register (index 0) when entering a number
        if self.entering_number && !self.number_entry_string.is_empty() && register_index == 0 {
            return format!("{}_", self.number_entry_string);  // Add underscore for number entry
        }
        
        // Otherwise format the computed value normally
        self.display_formatter.format_number(value, width)
    }
}

// Simple gamma function approximation for factorial
fn gamma(x: f64) -> f64 {
    if x == 1.0 {
        1.0
    } else if x < 1.0 {
        gamma(x + 1.0) / x
    } else {
        (x - 1.0) * gamma(x - 1.0)
    }
}