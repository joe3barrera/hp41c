pub mod programming;
pub mod display;
pub mod commands;
pub mod calculator;
pub mod stack;
pub mod math;
pub mod input;
pub mod error;
pub mod execution;

// Modular command system
pub mod registry;
pub mod parser;

// NEW: Logging system
pub mod logger;

#[cfg(test)]
mod tests;

// Main calculator
pub use calculator::HP41CCalculator;

// Command system (clean, modular exports)
pub use registry::{CommandRegistry, CommandSpec, ArgumentPattern, AutoExecuteRule};
pub use parser::{CommandParser, ParseResult};

// Core components
pub use programming::{ProgrammingMode, ProgramInstruction};
pub use display::{DisplayMode, DisplayFormatter};
pub use error::{CalculatorError, CalculatorResult};
pub use stack::Stack;
pub use math::*;
pub use input::InputState;

// NEW: Logger exports
pub use logger::Logger;