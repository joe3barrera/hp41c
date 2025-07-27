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

pub use calculator::HP41CCalculator;
pub use programming::{ProgrammingMode, ProgramInstruction};
pub use display::{DisplayMode, DisplayFormatter};
pub use commands::{CommandTrie, initialize_command_trie};
pub use error::{CalculatorError, CalculatorResult};
pub use stack::Stack;
pub use math::*;
pub use input::InputState;
