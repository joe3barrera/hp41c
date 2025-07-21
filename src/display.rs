#[derive(Debug, Clone, PartialEq)]
pub enum DisplayMode {
    Fix,  // FIX mode - fixed decimal places
    Sci,  // SCI mode - scientific notation
    Eng,  // ENG mode - engineering notation (powers of 3)
}

#[derive(Debug)]
pub struct DisplayFormatter {
    pub mode: DisplayMode,
    pub digits: usize,
}

impl DisplayFormatter {
    pub fn new() -> Self {
        DisplayFormatter {
            mode: DisplayMode::Fix,
            digits: 4,  // HP-41C default
        }
    }

    pub fn format_number(&self, value: f64, width: usize) -> String {
        // Standard number formatting using HP-41C display modes
        if value == 0.0 {
            return match self.mode {
                DisplayMode::Fix => {
                    if self.digits == 0 {
                        "0".to_string()
                    } else {
                        format!("0.{}", "0".repeat(self.digits))
                    }
                }
                DisplayMode::Sci => format!("0.{}E+00", "0".repeat(self.digits)),
                DisplayMode::Eng => format!("0.{}E+00", "0".repeat(self.digits)),
            };
        }

        let formatted = match self.mode {
            DisplayMode::Fix => {
                format!("{:.1$}", value, self.digits)
            }
            DisplayMode::Sci => {
                format!("{:.1$e}", value, self.digits)
            }
            DisplayMode::Eng => {
                // Engineering notation: exponent is multiple of 3
                let log_val = value.abs().log10();
                let exp_eng = (log_val / 3.0).floor() as i32 * 3;
                let mantissa = value / 10.0_f64.powi(exp_eng);
                format!("{:.1$}E{2:+03}", mantissa, self.digits, exp_eng)
            }
        };

        // Truncate if too long for display width
        if formatted.len() > width {
            formatted[..width].to_string()
        } else {
            formatted
        }
    }

    pub fn get_mode_string(&self) -> String {
        match self.mode {
            DisplayMode::Fix => format!("FIX {}", self.digits),
            DisplayMode::Sci => format!("SCI {}", self.digits),
            DisplayMode::Eng => format!("ENG {}", self.digits),
        }
    }
}