/// HP-41C Stack Operations
/// 
/// The HP-41C uses a 4-level RPN stack (X, Y, Z, T registers)
/// with specific lift and drop behaviors that this module faithfully emulates.
/// 
/// # Example
/// ```
/// use hp41c::stack::Stack;
/// 
/// let mut stack = Stack::new();
/// stack.set_x(5.0);
/// stack.lift();
/// stack.set_x(3.0);
/// let result = stack.add().unwrap();
/// assert_eq!(result, 8.0);
/// ```

use std::fmt;
use crate::error::StackError;

/// The 4-level RPN stack used in the HP-41C
#[derive(Debug, Clone)]
pub struct Stack {
    /// Stack registers: [X, Y, Z, T]
    registers: [f64; 4],
    /// Flag indicating if the stack should lift on next number entry
    lifted: bool,
}

/// Stack register indices for clarity
const X: usize = 0;
const Y: usize = 1;
const Z: usize = 2;
const T: usize = 3;

impl Stack {
    /// Create a new stack with all registers set to 0.0
    pub fn new() -> Self {
        Stack {
            registers: [0.0; 4],
            lifted: false,
        }
    }

    /// Get the value in the X register (bottom of stack)
    pub fn x(&self) -> f64 {
        self.registers[X]
    }

    /// Get the value in the Y register
    pub fn y(&self) -> f64 {
        self.registers[Y]
    }

    /// Get the value in the Z register
    pub fn z(&self) -> f64 {
        self.registers[Z]
    }

    /// Get the value in the T register (top of stack)
    pub fn t(&self) -> f64 {
        self.registers[T]
    }

    /// Set the X register value directly (used for number entry)
    pub fn set_x(&mut self, value: f64) {
        self.registers[X] = value;
    }

    /// Check if stack should lift on next entry
    pub fn should_lift(&self) -> bool {
        self.lifted
    }

    /// Set the lift flag
    pub fn set_lift_flag(&mut self, value: bool) {
        self.lifted = value;
    }

    /// Push a value onto the stack (respecting lift flag)
    pub fn push(&mut self, value: f64) {
        if self.lifted {
            self.lift();
        }
        self.registers[X] = value;
        self.lifted = true;  // Next push should lift
    }

    /// Lift the stack (push values up)
    /// X → Y, Y → Z, Z → T, T is lost
    pub fn lift(&mut self) {
        self.registers[T] = self.registers[Z];
        self.registers[Z] = self.registers[Y];
        self.registers[Y] = self.registers[X];
    }

    /// Drop the stack (after binary operation)
    /// HP-41C behavior: Y → X, Z → Y, T → Z, T remains unchanged
    /// This means the old T value is duplicated into Z
    fn drop(&mut self) {
        self.registers[X] = self.registers[Y];
        self.registers[Y] = self.registers[Z];
        self.registers[Z] = self.registers[T];
        // T remains unchanged (the duplication happens above)
    }

    /// Perform addition (Y + X)
    pub fn add(&mut self) -> Result<f64, StackError> {
        self.binary_operation(|y, x| y + x)
    }

    /// Perform subtraction (Y - X)
    pub fn subtract(&mut self) -> Result<f64, StackError> {
        self.binary_operation(|y, x| y - x)
    }

    /// Perform multiplication (Y * X)
    pub fn multiply(&mut self) -> Result<f64, StackError> {
        self.binary_operation(|y, x| y * x)
    }

    /// Perform division (Y / X)
    pub fn divide(&mut self) -> Result<f64, StackError> {
        if self.registers[X] == 0.0 {
            Err(StackError::DivisionByZero)
        } else {
            self.binary_operation(|y, x| y / x)
        }
    }

    /// Perform power operation (Y ^ X)
    pub fn power(&mut self) -> Result<f64, StackError> {
        self.binary_operation(|y, x| y.powf(x))
    }

    /// Generic binary operation handler
    fn binary_operation<F>(&mut self, op: F) -> Result<f64, StackError>
    where
        F: Fn(f64, f64) -> f64,
    {
        let result = op(self.registers[Y], self.registers[X]);
        
        // Check for invalid results
        if result.is_nan() {
            return Err(StackError::MathError("Invalid calculation".to_string()));
        }
        if result.is_infinite() {
            return Err(StackError::MathError("Overflow".to_string()));
        }

        // Store result and drop stack
        self.drop();
        self.registers[X] = result;
        self.lifted = true;

        Ok(result)
    }

    /// Swap X and Y registers
    pub fn swap(&mut self) {
        self.registers.swap(X, Y);
    }

    /// Clear X register only
    pub fn clear_x(&mut self) {
        self.registers[X] = 0.0;
    }

    /// Clear entire stack
    pub fn clear_all(&mut self) {
        self.registers = [0.0; 4];
        self.lifted = false;
    }

    /// Change sign of X register
    pub fn change_sign(&mut self) {
        self.registers[X] = -self.registers[X];
    }

    /// Get a copy of all registers (for display/debugging)
    pub fn get_registers(&self) -> [f64; 4] {
        self.registers
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Stack {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "T:{:10.4} Z:{:10.4} Y:{:10.4} X:{:10.4}",
               self.registers[T], self.registers[Z], 
               self.registers[Y], self.registers[X])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_lift() {
        let mut stack = Stack::new();
        stack.registers = [1.0, 2.0, 3.0, 4.0];
        
        stack.lift();
        
        assert_eq!(stack.get_registers(), [1.0, 1.0, 2.0, 3.0]);
    }

    #[test]
    fn test_stack_drop_preserves_t() {
        let mut stack = Stack::new();
        stack.registers = [0.0, 2.0, 3.0, 4.0];
        
        // Perform an operation that causes drop
        let result = stack.add().unwrap();
        
        assert_eq!(result, 2.0);  // 2 + 0 = 2
        assert_eq!(stack.x(), 2.0);
        assert_eq!(stack.y(), 3.0);
        assert_eq!(stack.z(), 4.0);  // Old T value
        assert_eq!(stack.t(), 4.0);  // T unchanged
    }

    #[test]
    fn test_division_by_zero() {
        let mut stack = Stack::new();
        stack.set_x(5.0);  // Y will be 5
        stack.lift();      // Now Y=5, X=5
        stack.set_x(0.0);  // Now Y=5, X=0
        
        // This should try to compute 5/0
        assert_eq!(stack.divide(), Err(StackError::DivisionByZero));
    }

    #[test]
    fn test_swap() {
        let mut stack = Stack::new();
        stack.registers = [1.0, 2.0, 3.0, 4.0];
        
        stack.swap();
        
        assert_eq!(stack.x(), 2.0);
        assert_eq!(stack.y(), 1.0);
        assert_eq!(stack.z(), 3.0);
        assert_eq!(stack.t(), 4.0);
    }
}