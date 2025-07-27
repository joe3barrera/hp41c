/// Mathematical operations for HP-41C
/// 
/// Provides all mathematical functions including trigonometric, logarithmic,
/// and other scientific functions with proper error handling.

use crate::error::StackError;

/// Maximum value for factorial calculation
const FACTORIAL_MAX: f64 = 170.0;

/// Execute a mathematical function on a value
/// 
/// # Arguments
/// * `function` - The function name (e.g., "sin", "cos", "log")
/// * `x` - The input value
/// 
/// # Returns
/// The result of the calculation or an error
pub fn execute_math_function(function: &str, x: f64) -> Result<f64, StackError> {
    let result = match function {
        "sin" => x.sin(),
        "cos" => x.cos(),
        "tan" => x.tan(),
        "asin" => validate_asin_acos_input(x)?.asin(),
        "acos" => validate_asin_acos_input(x)?.acos(),
        "atan" => x.atan(),
        "log" => validate_positive(x, "log")?.log10(),
        "ln" => validate_positive(x, "ln")?.ln(),
        "exp" => x.exp(),
        "sqrt" => validate_non_negative(x, "sqrt")?.sqrt(),
        "inv" => invert(x)?,
        _ => return Err(StackError::MathError(format!("Unknown function '{}'", function))),
    };

    validate_result(result, function)
}

/// Validate input for asin/acos (must be in [-1, 1])
fn validate_asin_acos_input(x: f64) -> Result<f64, StackError> {
    if x < -1.0 || x > 1.0 {
        Err(StackError::MathError("Input must be in range [-1, 1]".to_string()))
    } else {
        Ok(x)
    }
}

/// Validate positive input for log functions
fn validate_positive(x: f64, function: &str) -> Result<f64, StackError> {
    if x <= 0.0 {
        Err(StackError::MathError(format!("{} requires positive input", function)))
    } else {
        Ok(x)
    }
}

/// Validate non-negative input for sqrt
fn validate_non_negative(x: f64, function: &str) -> Result<f64, StackError> {
    if x < 0.0 {
        Err(StackError::MathError(format!("{} requires non-negative input", function)))
    } else {
        Ok(x)
    }
}

/// Calculate 1/x with division by zero check
fn invert(x: f64) -> Result<f64, StackError> {
    if x == 0.0 {
        Err(StackError::DivisionByZero)
    } else {
        Ok(1.0 / x)
    }
}

/// Validate the result of a calculation
fn validate_result(result: f64, function: &str) -> Result<f64, StackError> {
    if result.is_nan() {
        Err(StackError::MathError(format!("{}: Invalid result", function)))
    } else if result.is_infinite() {
        Err(StackError::MathError(format!("{}: Overflow", function)))
    } else {
        Ok(result)
    }
}

/// Calculate factorial using gamma function
/// 
/// # Arguments
/// * `x` - The input value (must be non-negative and <= 170)
/// 
/// # Returns
/// The factorial of x or an error
pub fn factorial(x: f64) -> Result<f64, StackError> {
    if x < 0.0 {
        Err(StackError::MathError("Factorial requires non-negative input".to_string()))
    } else if x > FACTORIAL_MAX {
        Err(StackError::MathError(format!("Factorial input must be <= {}", FACTORIAL_MAX)))
    } else if x.fract() != 0.0 {
        Err(StackError::MathError("Factorial requires integer input".to_string()))
    } else {
        Ok(gamma(x + 1.0))
    }
}

/// Simple gamma function approximation for factorial
/// 
/// This is a recursive implementation suitable for small integer values.
/// For production use, consider Stirling's approximation or lgamma.
fn gamma(x: f64) -> f64 {
    if x == 1.0 {
        1.0
    } else if x < 1.0 {
        gamma(x + 1.0) / x
    } else {
        (x - 1.0) * gamma(x - 1.0)
    }
}

/// Convert degrees to radians
pub fn deg_to_rad(degrees: f64) -> f64 {
    degrees * std::f64::consts::PI / 180.0
}

/// Convert radians to degrees  
pub fn rad_to_deg(radians: f64) -> f64 {
    radians * 180.0 / std::f64::consts::PI
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trig_functions() {
        // Test at key angles
        assert!((execute_math_function("sin", 0.0).unwrap() - 0.0).abs() < 1e-10);
        assert!((execute_math_function("cos", 0.0).unwrap() - 1.0).abs() < 1e-10);
        assert!((execute_math_function("sin", std::f64::consts::PI / 2.0).unwrap() - 1.0).abs() < 1e-10);
        assert!((execute_math_function("cos", std::f64::consts::PI).unwrap() - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn test_inverse_trig() {
        assert!(execute_math_function("asin", 2.0).is_err());
        assert!(execute_math_function("acos", -2.0).is_err());
        assert!((execute_math_function("asin", 1.0).unwrap() - std::f64::consts::PI / 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_log_functions() {
        assert!((execute_math_function("log", 100.0).unwrap() - 2.0).abs() < 1e-10);
        assert!((execute_math_function("ln", std::f64::consts::E).unwrap() - 1.0).abs() < 1e-10);
        assert!(execute_math_function("log", -1.0).is_err());
        assert!(execute_math_function("ln", 0.0).is_err());
    }

    #[test]
    fn test_sqrt() {
        assert_eq!(execute_math_function("sqrt", 4.0).unwrap(), 2.0);
        assert_eq!(execute_math_function("sqrt", 0.0).unwrap(), 0.0);
        assert!(execute_math_function("sqrt", -1.0).is_err());
    }

    #[test]
    fn test_factorial() {
        assert_eq!(factorial(0.0).unwrap(), 1.0);
        assert_eq!(factorial(5.0).unwrap(), 120.0);
        assert_eq!(factorial(10.0).unwrap(), 3628800.0);
        assert!(factorial(-1.0).is_err());
        assert!(factorial(171.0).is_err());
        assert!(factorial(5.5).is_err()); // Non-integer
    }

    #[test]
    fn test_invert() {
        assert_eq!(execute_math_function("inv", 2.0).unwrap(), 0.5);
        assert_eq!(execute_math_function("inv", -4.0).unwrap(), -0.25);
        assert!(matches!(
            execute_math_function("inv", 0.0),
            Err(StackError::DivisionByZero)
        ));
    }
}