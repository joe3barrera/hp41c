/// Command execution for the HP-41C calculator
/// 
/// Handles the execution of all calculator commands including math functions,
/// stack operations, programming commands, and storage operations.

use crate::stack::Stack;
use crate::input::InputState;
use crate::math::{execute_math_function, factorial};
use crate::programming::ProgrammingMode;
use crate::display::{DisplayMode, DisplayFormatter};
use crate::error::{CalculatorError, CommandError, StorageError, ProgrammingError};

/// Execute a calculator command
pub fn execute_command(
    command: &str,
    args: Option<Vec<String>>,
    stack: &mut Stack,
    input: &mut InputState,
    programming: &mut ProgrammingMode,
    display: &mut DisplayFormatter,
    storage: &mut [f64],
) -> Result<Option<String>, CalculatorError> {
    let command = command.to_lowercase();
    
    match command.as_str() {
        // Arithmetic operators
        "+" => {
            stack.add()?;
            input.clear();
            Ok(None)
        }
        "-" => {
            stack.subtract()?;
            input.clear();
            Ok(None)
        }
        "*" => {
            stack.multiply()?;
            input.clear();
            Ok(None)
        }
        "/" => {
            stack.divide()?;
            input.clear();
            Ok(None)
        }
        "^" => {
            stack.power()?;
            input.clear();
            Ok(None)
        }

        // Math functions
        "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | 
        "log" | "ln" | "exp" | "sqrt" | "inv" => {
            execute_math_command(&command, stack, input)
        }
        
        // Stack operations
        "enter" => execute_enter(stack, input),
        "swap" => execute_swap(stack),
        "clx" => execute_clear_x(stack, input),
        "clr" => execute_clear_all(stack, input),
        "chs" => execute_change_sign(stack),
        
        // Constants
        "pi" => execute_pi(stack, input),
        "pow" => execute_power(stack, input),
        
        // Programming
        "lbl" | "gto" | "xeq" | "rtn" | "sst" | "bst" | "prgm" => {
            execute_programming_command(&command, args, programming, stack)
        }
        
        // Display modes
        "fix" | "sci" | "eng" => {
            execute_display_command(&command, args, display)
        }
        
        // Storage
        "sto" | "rcl" => {
            execute_storage_command(&command, args, stack, storage)
        }
        
        // Special
	 "!" => execute_factorial(stack, input),
        "eex" => execute_eex(input),
        "arc" => Ok(Some("ARC mode not implemented".to_string())),
        
        _ => Err(CommandError::UnknownCommand(command).into()),
    }
}

// Math command execution
fn execute_math_command(
    function: &str,
    stack: &mut Stack,
    input: &mut InputState,
) -> Result<Option<String>, CalculatorError> {
    let result = execute_math_function(function, stack.x())?;
    stack.set_x(result);
    stack.set_lift_flag(true);
    input.clear();
    Ok(None)
}

// Stack operations
fn execute_enter(stack: &mut Stack, input: &mut InputState) -> Result<Option<String>, CalculatorError> {
    stack.lift();
    stack.set_lift_flag(false);
    input.clear();
    Ok(None)
}

fn execute_swap(stack: &mut Stack) -> Result<Option<String>, CalculatorError> {
    stack.swap();
    Ok(None)
}

fn execute_clear_x(stack: &mut Stack, input: &mut InputState) -> Result<Option<String>, CalculatorError> {
    stack.clear_x();
    input.clear();
    Ok(None)
}

fn execute_clear_all(stack: &mut Stack, input: &mut InputState) -> Result<Option<String>, CalculatorError> {
    stack.clear_all();
    input.clear();
    Ok(None)
}

fn execute_change_sign(stack: &mut Stack) -> Result<Option<String>, CalculatorError> {
    stack.change_sign();
    Ok(None)
}

// Constants and special operations
fn execute_pi(stack: &mut Stack, input: &mut InputState) -> Result<Option<String>, CalculatorError> {
    if stack.should_lift() {
        stack.lift();
    }
    stack.set_x(std::f64::consts::PI);
    stack.set_lift_flag(true);
    input.clear();
    Ok(None)
}

fn execute_power(stack: &mut Stack, input: &mut InputState) -> Result<Option<String>, CalculatorError> {
    stack.power()?;
    input.clear();
    Ok(None)
}

fn execute_eex(input: &mut InputState) -> Result<Option<String>, CalculatorError> {
    input.enter_eex_mode()?;
    Ok(None)
}

fn execute_factorial(stack: &mut Stack, input: &mut InputState) -> Result<Option<String>, CalculatorError> {
    let result = factorial(stack.x())?;
    stack.set_x(result);
    stack.set_lift_flag(true);
    input.clear();
    Ok(None)
}

// Programming commands
fn execute_programming_command(
    command: &str,
    args: Option<Vec<String>>,
    programming: &mut ProgrammingMode,
    _stack: &mut Stack,
) -> Result<Option<String>, CalculatorError> {
    match command {
        "lbl" => {
            if programming.is_programming {
                let args = args.ok_or(CommandError::MissingArgument("LBL".to_string()))?;
                programming.add_instruction("LBL", Some(args.clone()), &format!("LBL {}", args[0]));
                Ok(None)
            } else {
                Ok(None)
            }
        }
        
        "gto" => {
            let args = args.ok_or(CommandError::MissingArgument("GTO".to_string()))?;
            if programming.goto_label(&args[0]) {
                Ok(None)
            } else {
                Err(ProgrammingError::LabelNotFound(args[0].clone()).into())
            }
        }
        
        "xeq" => {
            let args = args.ok_or(CommandError::MissingArgument("XEQ".to_string()))?;
            if programming.execute_subroutine(&args[0]) {
                programming.is_running = true;
                Ok(None)
            } else {
                Err(ProgrammingError::LabelNotFound(args[0].clone()).into())
            }
        }
        
        "rtn" => {
            if programming.is_programming {
                programming.add_instruction("RTN", None, "RTN");
            } else {
                programming.return_from_subroutine();
            }
            Ok(None)
        }
        
        "sst" => {
            if programming.program.is_empty() {
                Err(ProgrammingError::NoProgram.into())
            } else {
                programming.program_counter = (programming.program_counter + 1) % programming.program.len();
                if let Some(instr) = programming.get_current_instruction() {
                    Ok(Some(format!("{:02} {}", instr.line_number, instr)))
                } else {
                    Ok(Some("End of program".to_string()))
                }
            }
        }
        
        "bst" => {
            if programming.program.is_empty() {
                Err(ProgrammingError::NoProgram.into())
            } else {
                if programming.program_counter > 0 {
                    programming.program_counter -= 1;
                } else {
                    programming.program_counter = programming.program.len() - 1;
                }
                if let Some(instr) = programming.get_current_instruction() {
                    Ok(Some(format!("{:02} {}", instr.line_number, instr)))
                } else {
                    Ok(Some("Start of program".to_string()))
                }
            }
        }
        
        "prgm" => {
            programming.clear_program();
            Ok(Some("Program cleared".to_string()))
        }
        
        _ => unreachable!(),
    }
}

// Display mode commands
fn execute_display_command(
    command: &str,
    args: Option<Vec<String>>,
    display: &mut DisplayFormatter,
) -> Result<Option<String>, CalculatorError> {
    let args = args.ok_or(CommandError::MissingArgument(command.to_uppercase()))?;
    let digits = args[0].parse::<usize>()
        .map_err(|_| CommandError::InvalidArgument {
            command: command.to_uppercase(),
            argument: args[0].clone(),
        })?;
    
    if digits > 9 {
        return Err(CommandError::InvalidArgument {
            command: command.to_uppercase(),
            argument: args[0].clone(),
        }.into());
    }

    display.mode = match command {
        "fix" => DisplayMode::Fix,
        "sci" => DisplayMode::Sci,
        "eng" => DisplayMode::Eng,
        _ => unreachable!(),
    };
    display.digits = digits;
    
    Ok(Some(format!("{} {}", command.to_uppercase(), digits)))
}

// Storage commands
fn execute_storage_command(
    command: &str,
    args: Option<Vec<String>>,
    stack: &mut Stack,
    storage: &mut [f64],
) -> Result<Option<String>, CalculatorError> {
    let args = args.ok_or(CommandError::MissingArgument(command.to_uppercase()))?;
    let register = args[0].parse::<usize>()
        .map_err(|_| StorageError::InvalidRegister(0))?;
    
    if register >= storage.len() {
        return Err(StorageError::InvalidRegister(register).into());
    }

    match command {
        "sto" => {
            storage[register] = stack.x();
            Ok(Some(format!("STO {:02}", register)))
        }
        "rcl" => {
            if stack.should_lift() {
                stack.lift();
            }
            stack.set_x(storage[register]);
            stack.set_lift_flag(true);
            Ok(Some(format!("RCL {:02}", register)))
        }
        _ => unreachable!(),
    }
}
