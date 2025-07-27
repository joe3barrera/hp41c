use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ProgramInstruction {
    pub line_number: i32,
    pub command: String,
    pub arguments: Vec<String>,
}

impl ProgramInstruction {
    pub fn new(line_number: i32, command: String, arguments: Vec<String>) -> Self {
        ProgramInstruction {
            line_number,
            command,
            arguments,
        }
    }
}

impl std::fmt::Display for ProgramInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.arguments.is_empty() {
            write!(f, "{} {}", self.command, self.arguments.join(" "))
        } else {
            write!(f, "{}", self.command)
        }
    }
}

#[derive(Debug)]
pub struct ProgrammingMode {
    pub program: Vec<ProgramInstruction>,
    
    // Execution state
    pub program_counter: usize,        // Index into program[] for execution
    pub is_running: bool,
    pub subroutine_stack: Vec<usize>,
    
    // Editing state  
    pub edit_position: usize,          // Index into program[] for editing
    pub is_programming: bool,
    
    // Shared state
    pub labels: HashMap<String, i32>,
    pub current_line: i32,             // For auto-numbering new instructions
}

impl ProgrammingMode {
    pub fn new() -> Self {
        ProgrammingMode {
            program: Vec::new(),
            program_counter: 0,
            is_running: false,
            subroutine_stack: Vec::new(),
            edit_position: 0,
            is_programming: false,
            labels: HashMap::new(),
            current_line: 1,
        }
    }

    pub fn toggle_programming_mode(&mut self) -> bool {
        self.is_programming = !self.is_programming;
        if !self.is_programming {
            self.rebuild_label_table();
        } else {
            // When entering programming mode, position at end of program
            self.edit_position = self.program.len();
        }
        self.is_programming
    }

    // SST behavior depends on current mode
    pub fn sst_execute(&mut self, calc: &mut crate::calculator::HP41CCalculator) -> Result<Option<String>, String> {
        // Run mode: execute one instruction and pause
        if self.program_counter < self.program.len() {
            let instruction = self.program[self.program_counter].clone();
            self.program_counter += 1;
            self.is_running = false; // Pause after single step
            
            // Execute the instruction
            calc.execute_command(&instruction.command, Some(instruction.arguments))?;
            Ok(Some(format!("SST: {}", instruction)))
        } else {
            Ok(Some(".END. 49".to_string()))
        }
    }
    
    pub fn sst_edit(&mut self) -> Result<Option<String>, String> {
        // Programming mode: navigate to next step for editing
        if self.edit_position < self.program.len() {
            self.edit_position += 1;
            if self.edit_position < self.program.len() {
                let instruction = &self.program[self.edit_position];
                Ok(Some(format!("{:02} {}", instruction.line_number, instruction)))
            } else {
                Ok(Some(format!("{:02} .END.", self.current_line)))
            }
        } else {
            Ok(Some(format!("{:02} .END.", self.current_line)))
        }
    }

    // BST behavior depends on current mode  
    pub fn bst_execute(&mut self) -> Result<Option<String>, String> {
        // Run mode: back up one program step (don't execute)
        if self.program_counter > 0 {
            self.program_counter -= 1;
            let instruction = &self.program[self.program_counter];
            Ok(Some(format!("BST: {:02} {}", instruction.line_number, instruction)))
        } else {
            Ok(Some("Beginning of program".to_string()))
        }
    }
    
    pub fn bst_edit(&mut self) -> Result<Option<String>, String> {
        // Programming mode: navigate to previous step for editing
        if self.edit_position > 0 {
            self.edit_position -= 1;
            let instruction = &self.program[self.edit_position];
            Ok(Some(format!("{:02} {}", instruction.line_number, instruction)))
        } else {
            Ok(Some("Beginning of program".to_string()))
        }
    }

    pub fn add_instruction(&mut self, command: &str, arguments: Option<Vec<String>>, _raw_input: &str) -> bool {
        if !self.is_programming {
            return false;
        }

        let args = arguments.unwrap_or_default();
        let instruction = ProgramInstruction::new(
            self.current_line,
            command.to_uppercase(),
            args.iter().map(|s| s.to_uppercase()).collect(),
        );

        // Insert at current edit position
        self.insert_at_edit_position(instruction);
        self.current_line += 1;
        self.edit_position += 1; // Move to next position after insertion
        true
    }

    pub fn insert_at_edit_position(&mut self, instruction: ProgramInstruction) {
        if self.edit_position >= self.program.len() {
            // Insert at end
            self.program.push(instruction);
        } else {
            // Insert in middle, shift everything else down
            self.program.insert(self.edit_position, instruction);
        }
        
        // Renumber all instructions after insertion
        self.renumber_program();
    }

    pub fn delete_current_instruction(&mut self) -> Result<Option<String>, String> {
        if !self.is_programming {
            return Err("Not in programming mode".to_string());
        }
        
        if self.edit_position < self.program.len() {
            let deleted = self.program.remove(self.edit_position);
            self.renumber_program();
            
            // Stay at same position, but show what's now there
            if self.edit_position < self.program.len() {
                let current = &self.program[self.edit_position];
                Ok(Some(format!("Deleted: {} | Now: {:02} {}", deleted, current.line_number, current)))
            } else {
                Ok(Some(format!("Deleted: {} | At end", deleted)))
            }
        } else {
            Err("No instruction to delete".to_string())
        }
    }

    fn renumber_program(&mut self) {
        for (i, instruction) in self.program.iter_mut().enumerate() {
            instruction.line_number = (i + 1) as i32;
        }
        self.current_line = (self.program.len() + 1) as i32;
        self.rebuild_label_table();
    }

    pub fn insert_at_line(&mut self, instruction: ProgramInstruction) {
        let insert_pos = self.program.iter().position(|existing| {
            existing.line_number >= instruction.line_number
        });

        match insert_pos {
            Some(pos) => {
                if self.program[pos].line_number == instruction.line_number {
                    self.program[pos] = instruction;
                } else {
                    self.program.insert(pos, instruction);
                }
            }
            None => self.program.push(instruction),
        }
    }

    pub fn rebuild_label_table(&mut self) {
        self.labels.clear();
        for instruction in &self.program {
            if instruction.command == "LBL" && !instruction.arguments.is_empty() {
                self.labels.insert(instruction.arguments[0].clone(), instruction.line_number);
            }
        }
    }

    pub fn goto_label(&mut self, label: &str) -> bool {
        if let Some(&target_line) = self.labels.get(&label.to_uppercase()) {
            for (i, instruction) in self.program.iter().enumerate() {
                if instruction.line_number >= target_line {
                    if self.is_programming {
                        self.edit_position = i;
                    } else {
                        self.program_counter = i;
                    }
                    return true;
                }
            }
        }
        false
    }

    pub fn execute_subroutine(&mut self, label: &str) -> bool {
        if self.goto_label(label) {
            self.subroutine_stack.push(self.program_counter);
            true
        } else {
            false
        }
    }

    pub fn return_from_subroutine(&mut self) -> bool {
        if let Some(return_addr) = self.subroutine_stack.pop() {
            self.program_counter = return_addr;
            true
        } else {
            self.is_running = false;
            false
        }
    }

    pub fn clear_program(&mut self) {
        self.program.clear();
        self.labels.clear();
        self.program_counter = 0;
        self.edit_position = 0;
        self.current_line = 1;
        self.is_running = false;
        self.subroutine_stack.clear();
    }

    pub fn get_current_instruction(&self) -> Option<&ProgramInstruction> {
        if self.is_programming {
            // In programming mode, show instruction at edit position
            if self.edit_position < self.program.len() {
                Some(&self.program[self.edit_position])
            } else {
                None
            }
        } else {
            // In run mode, show instruction at program counter
            self.program.get(self.program_counter)
        }
    }

    pub fn get_current_step_display(&self) -> String {
        if self.is_programming {
            if self.edit_position < self.program.len() {
                let instruction = &self.program[self.edit_position];
                format!("{:02} {}", instruction.line_number, instruction)
            } else {
                format!("{:02} .END.", self.current_line)
            }
        } else if self.program_counter < self.program.len() {
            let instruction = &self.program[self.program_counter];
            format!("{:02} {}", instruction.line_number, instruction)
        } else {
            ".END.".to_string()
        }
    }
}