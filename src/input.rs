/// Input processing for HP-41C calculator
/// 
/// Handles number entry including decimal points and EEX (Enter Exponent) mode.
/// Maintains the state of number entry and provides display formatting.

//use std::fmt;
use crate::error::InputError;

/// Manages the state of number input
#[derive(Debug, Clone)]
pub struct InputState {
    /// Whether we're currently entering a number
    entering_number: bool,
    /// The string being built during number entry
    number_entry_string: String,
    /// Whether we're in EEX (Enter Exponent) mode
    eex_mode: bool,
    /// Digits entered for the exponent
    eex_digits: String,
}

/// Maximum length for number entry (to prevent overflow)
const MAX_ENTRY_LENGTH: usize = 15;
const MAX_EEX_DIGITS: usize = 3;

impl InputState {
    /// Create a new input state
    pub fn new() -> Self {
        InputState {
            entering_number: false,
            number_entry_string: String::new(),
            eex_mode: false,
            eex_digits: String::new(),
        }
    }

    /// Check if currently entering a number
    pub fn is_entering(&self) -> bool {
        self.entering_number
    }

    /// Check if in EEX mode
    pub fn is_eex_mode(&self) -> bool {
        self.eex_mode
    }

    /// Start number entry
    pub fn begin_entry(&mut self) {
        self.entering_number = true;
        self.number_entry_string.clear();
        self.eex_mode = false;
        self.eex_digits.clear();
    }

    /// Clear all input state
    pub fn clear(&mut self) {
        self.entering_number = false;
        self.number_entry_string.clear();
        self.eex_mode = false;
        self.eex_digits.clear();
    }

    /// Enter EEX mode
    pub fn enter_eex_mode(&mut self) -> Result<(), InputError> {
        if !self.entering_number {
            self.begin_entry();
            self.number_entry_string.push('0');
        }
        self.eex_mode = true;
        self.eex_digits.clear();
        Ok(())
    }

    /// Handle a digit or decimal point input
    pub fn handle_digit(&mut self, key: char) -> Result<Option<f64>, InputError> {
        // Validate input
        if !key.is_ascii_digit() && key != '.' {
            return Ok(None);
        }

        if !self.entering_number {
            self.begin_entry();
        }

        if self.eex_mode {
            self.handle_eex_digit(key)
        } else {
            self.handle_mantissa_digit(key)
        }
    }

    /// Handle digit input for mantissa
    fn handle_mantissa_digit(&mut self, key: char) -> Result<Option<f64>, InputError> {
        if key == '.' {
            // Only one decimal point allowed
            if self.number_entry_string.contains('.') {
                return Ok(None);
            }
            // If empty, start with "0."
            if self.number_entry_string.is_empty() {
                self.number_entry_string.push('0');
            }
            self.number_entry_string.push('.');
            Ok(None) // Can't parse incomplete decimal
        } else {
            // Prevent overflow
            if self.number_entry_string.len() >= MAX_ENTRY_LENGTH {
                return Err(InputError::Overflow);
            }

            // Replace single "0" with new digit
            if self.number_entry_string == "0" {
                self.number_entry_string = key.to_string();
            } else {
                self.number_entry_string.push(key);
            }

            self.try_parse()
        }
    }

    /// Handle digit input in EEX mode
    fn handle_eex_digit(&mut self, key: char) -> Result<Option<f64>, InputError> {
        if !key.is_ascii_digit() {
            return Ok(None);
        }

        if self.eex_digits.len() >= MAX_EEX_DIGITS {
            return Err(InputError::Overflow);
        }

        self.eex_digits.push(key);
        self.try_parse()
    }

    /// Try to parse the current input as a number
    fn try_parse(&self) -> Result<Option<f64>, InputError> {
        let number_str = self.build_number_string();
        
        match number_str.parse::<f64>() {
            Ok(value) => {
                if value.is_infinite() {
                    Err(InputError::Overflow)
                } else {
                    Ok(Some(value))
                }
            }
            Err(_) => {
                // Special case: trailing decimal is OK
                if self.number_entry_string.ends_with('.') && !self.eex_mode {
                    Ok(None)
                } else {
                    Err(InputError::InvalidNumber(number_str))
                }
            }
        }
    }

    /// Build the complete number string for parsing
    fn build_number_string(&self) -> String {
        if self.eex_mode && !self.eex_digits.is_empty() {
            let mantissa = if self.number_entry_string.is_empty() {
                "0"
            } else {
                &self.number_entry_string
            };
            format!("{}E{}", mantissa, self.eex_digits)
        } else {
            self.number_entry_string.clone()
        }
    }

    /// Handle backspace during number entry
    pub fn handle_backspace(&mut self) -> Option<f64> {
        if self.eex_mode && !self.eex_digits.is_empty() {
            self.eex_digits.pop();
            if self.eex_digits.is_empty() {
                self.eex_mode = false;
            }
        } else if self.eex_mode {
            self.eex_mode = false;
        } else if !self.number_entry_string.is_empty() {
            self.number_entry_string.pop();
            if self.number_entry_string.is_empty() {
                self.clear();
                return Some(0.0);
            }
        } else {
            self.clear();
            return Some(0.0);
        }

        // Try to parse what remains
        self.try_parse().unwrap_or(Some(0.0))
    }

    /// Get display string for current number entry
    pub fn get_display_string(&self) -> String {
        if !self.entering_number {
            return String::new();
        }

        let mut display = self.number_entry_string.clone();
        
        if self.eex_mode {
            display.push_str(" E");
            if !self.eex_digits.is_empty() {
                display.push_str(&self.eex_digits);
            }
        }
        
        // Add underscore cursor
        display.push('_');
        display
    }

    /// Get the raw entry string (for testing)
    #[cfg(test)]
    pub fn get_entry_string(&self) -> &str {
        &self.number_entry_string
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digit_entry() {
        let mut input = InputState::new();
        
        assert_eq!(input.handle_digit('1').unwrap(), Some(1.0));
        assert_eq!(input.handle_digit('2').unwrap(), Some(12.0));
        assert_eq!(input.handle_digit('3').unwrap(), Some(123.0));
        assert_eq!(input.get_entry_string(), "123");
    }

    #[test]
    fn test_decimal_entry() {
        let mut input = InputState::new();
        
        input.handle_digit('1').unwrap();
        input.handle_digit('.').unwrap();
        input.handle_digit('5').unwrap();
        
        assert_eq!(input.get_entry_string(), "1.5");
        assert_eq!(input.get_display_string(), "1.5_");
    }

    #[test]
    fn test_leading_decimal() {
        let mut input = InputState::new();
        
        input.handle_digit('.').unwrap();
        assert_eq!(input.get_entry_string(), "0.");
        
        input.handle_digit('5').unwrap();
        assert_eq!(input.try_parse().unwrap(), Some(0.5));
    }

    #[test]
    fn test_eex_mode() {
        let mut input = InputState::new();
        
        input.handle_digit('1').unwrap();
        input.handle_digit('.').unwrap();
        input.handle_digit('5').unwrap();
        
        input.enter_eex_mode().unwrap();
        assert!(input.is_eex_mode());
        
        input.handle_digit('2').unwrap();
        assert_eq!(input.try_parse().unwrap(), Some(1.5e2));
        assert_eq!(input.get_display_string(), "1.5 E2_");
    }

    #[test]
    fn test_backspace() {
        let mut input = InputState::new();
        
        input.handle_digit('1').unwrap();
        input.handle_digit('2').unwrap();
        input.handle_digit('3').unwrap();
        
        assert_eq!(input.handle_backspace(), Some(12.0));
        assert_eq!(input.get_entry_string(), "12");
        
        assert_eq!(input.handle_backspace(), Some(1.0));
        assert_eq!(input.handle_backspace(), Some(0.0));
        assert!(!input.is_entering());
    }

    #[test]
    fn test_overflow_protection() {
        let mut input = InputState::new();
        
        // Try to enter too many digits
        for _ in 0..MAX_ENTRY_LENGTH {
            input.handle_digit('9').unwrap();
        }
        
        // Next digit should fail
        assert!(matches!(
            input.handle_digit('9'),
            Err(InputError::Overflow)
        ));
    }
}
