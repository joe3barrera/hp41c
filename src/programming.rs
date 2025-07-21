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
    pub program_counter: usize,
    pub is_running: bool,
    pub is_programming: bool,
    pub subroutine_stack: Vec<usize>,
    pub labels: HashMap<String, i32>,
    pub current_line: i32,
}

impl ProgrammingMode {
    pub fn new() -> Self {
        ProgrammingMode {
            program: Vec::new(),
            program_counter: 0,
            is_running: false,
            is_programming: false,
            subroutine_stack: Vec::new(),
            labels: HashMap::new(),
            current_line: 1,
        }
    }

    pub fn toggle_programming_mode(&mut self) -> bool {
        self.is_programming = !self.is_programming;
        if !self.is_programming {
            self.rebuild_label_table();
        }
        self.is_programming
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

        self.insert_at_line(instruction);
        self.current_line += 1;
        true
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
                    self.program_counter = i;
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
        self.current_line = 1;
        self.is_running = false;
        self.subroutine_stack.clear();
    }

    pub fn get_current_instruction(&self) -> Option<&ProgramInstruction> {
        self.program.get(self.program_counter)
    }
}