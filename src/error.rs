/// Error types for the HP-41C calculator emulator
/// 
/// Consolidates all error handling into proper Rust error types
/// instead of using String errors throughout.

use std::fmt;

/// Main error type for calculator operations
#[derive(Debug, Clone, PartialEq)]
pub enum CalculatorError {
    /// Stack-related errors
    Stack(StackError),
    /// Input/number entry errors
    Input(InputError),
    /// Command execution errors
    Command(CommandError),
    /// Programming mode errors
    Programming(ProgrammingError),
    /// Storage register errors
    Storage(StorageError),
}

/// Errors that can occur during stack operations
#[derive(Debug, Clone, PartialEq)]
pub enum StackError {
    /// Division by zero attempted
    DivisionByZero,
    /// Mathematical error (NaN or infinity result)
    MathError(String),
    /// Stack underflow (not enough values for operation)
    Underflow,
}

/// Errors that can occur during input processing
#[derive(Debug, Clone, PartialEq)]
pub enum InputError {
    /// Invalid number format
    InvalidNumber(String),
    /// Number too large to represent
    Overflow,
    /// Invalid digit in current mode
    InvalidDigit(char),
}

/// Errors that can occur during command execution
#[derive(Debug, Clone, PartialEq)]
pub enum CommandError {
    /// Unknown command
    UnknownCommand(String),
    /// Command requires arguments but none provided
    MissingArgument(String),
    /// Invalid argument for command
    InvalidArgument { command: String, argument: String },
    /// Command not allowed in current mode
    NotAllowed(String),
}

/// Errors specific to programming mode
#[derive(Debug, Clone, PartialEq)]
pub enum ProgrammingError {
    /// Label not found
    LabelNotFound(String),
    /// Program memory full
    MemoryFull,
    /// No program in memory
    NoProgram,
    /// Invalid line number
    InvalidLine(i32),
    /// Stack overflow in subroutine calls
    SubroutineStackOverflow,
}

/// Errors related to storage registers
#[derive(Debug, Clone, PartialEq)]
pub enum StorageError {
    /// Invalid register number
    InvalidRegister(usize),
    /// Register arithmetic error
    ArithmeticError(String),
}

// Display implementations for all error types

impl fmt::Display for CalculatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CalculatorError::Stack(e) => write!(f, "Stack error: {}", e),
            CalculatorError::Input(e) => write!(f, "Input error: {}", e),
            CalculatorError::Command(e) => write!(f, "Command error: {}", e),
            CalculatorError::Programming(e) => write!(f, "Programming error: {}", e),
            CalculatorError::Storage(e) => write!(f, "Storage error: {}", e),
        }
    }
}

impl fmt::Display for StackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StackError::DivisionByZero => write!(f, "Division by zero"),
            StackError::MathError(msg) => write!(f, "Math error: {}", msg),
            StackError::Underflow => write!(f, "Stack underflow"),
        }
    }
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputError::InvalidNumber(s) => write!(f, "Invalid number: {}", s),
            InputError::Overflow => write!(f, "Number overflow"),
            InputError::InvalidDigit(c) => write!(f, "Invalid digit: '{}'", c),
        }
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::UnknownCommand(cmd) => write!(f, "Unknown command: {}", cmd),
            CommandError::MissingArgument(cmd) => write!(f, "{} requires argument", cmd),
            CommandError::InvalidArgument { command, argument } => {
                write!(f, "Invalid argument '{}' for {}", argument, command)
            }
            CommandError::NotAllowed(msg) => write!(f, "Not allowed: {}", msg),
        }
    }
}

impl fmt::Display for ProgrammingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProgrammingError::LabelNotFound(lbl) => write!(f, "Label {} not found", lbl),
            ProgrammingError::MemoryFull => write!(f, "Program memory full"),
            ProgrammingError::NoProgram => write!(f, "No program in memory"),
            ProgrammingError::InvalidLine(n) => write!(f, "Invalid line number: {}", n),
            ProgrammingError::SubroutineStackOverflow => write!(f, "Subroutine stack overflow"),
        }
    }
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::InvalidRegister(n) => write!(f, "Invalid register: {}", n),
            StorageError::ArithmeticError(msg) => write!(f, "Register arithmetic: {}", msg),
        }
    }
}

// Implement std::error::Error for all types
impl std::error::Error for CalculatorError {}
impl std::error::Error for StackError {}
impl std::error::Error for InputError {}
impl std::error::Error for CommandError {}
impl std::error::Error for ProgrammingError {}
impl std::error::Error for StorageError {}

// From implementations for ergonomic error conversion

impl From<StackError> for CalculatorError {
    fn from(err: StackError) -> Self {
        CalculatorError::Stack(err)
    }
}

impl From<InputError> for CalculatorError {
    fn from(err: InputError) -> Self {
        CalculatorError::Input(err)
    }
}

impl From<CommandError> for CalculatorError {
    fn from(err: CommandError) -> Self {
        CalculatorError::Command(err)
    }
}

impl From<ProgrammingError> for CalculatorError {
    fn from(err: ProgrammingError) -> Self {
        CalculatorError::Programming(err)
    }
}

impl From<StorageError> for CalculatorError {
    fn from(err: StorageError) -> Self {
        CalculatorError::Storage(err)
    }
}

/// Type alias for Results in the calculator
pub type CalculatorResult<T> = Result<T, CalculatorError>;