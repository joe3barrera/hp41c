pub mod programming;
pub mod display;
pub mod commands;
pub mod calculator;
pub mod stack;
pub mod math;
pub mod input;
pub mod error;
pub mod execution;

#[cfg(test)]
mod tests;

// NEW: Clean exports without deprecated backwards compatibility
pub use calculator::{HP41CCalculator, CommandSpec, ArgumentPattern, AutoExecuteRule, ParseResult, CommandRegistry, CommandParser};
pub use programming::{ProgrammingMode, ProgramInstruction};
pub use display::{DisplayMode, DisplayFormatter};
pub use error::{CalculatorError, CalculatorResult};
pub use stack::Stack;
pub use math::*;
pub use input::InputState;
