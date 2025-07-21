use hp41c::*;

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a calculator and process a sequence of keystrokes
    fn process_keys(keys: &[&str]) -> (HP41CCalculator, Vec<String>) {
        let mut calc = HP41CCalculator::new();
        let mut messages = Vec::new();
        
        for key in keys {
            match calc.process_input(key) {
                Ok(Some(msg)) => messages.push(msg),
                Ok(None) => {},
                Err(msg) => messages.push(format!("ERROR: {}", msg)),
            }
        }
        
        (calc, messages)
    }

    #[test]
    fn test_sto_rcl_command_buffer_display() {
        let mut calc = HP41CCalculator::new();
        
        // Test STO command buffer progression
        calc.process_input("s").unwrap();
        assert_eq!(calc.command_buffer, "s");
        
        calc.process_input("t").unwrap();
        assert_eq!(calc.command_buffer, "st");
        
        calc.process_input("o").unwrap();
        assert_eq!(calc.command_buffer, "sto ");  // Space should be added automatically
        
        // Test display formatting for STO
        let display = calc.get_display();
        assert!(display.contains("CMD: [sto __]"), "Display should show 'CMD: [sto __]' after typing sto, but got: {}", display);
        
        calc.process_input("4").unwrap();
        assert_eq!(calc.command_buffer, "sto 4");
        let display = calc.get_display();
        assert!(display.contains("CMD: [sto 4_]"), "Display should show 'CMD: [sto 4_]' after first digit, but got: {}", display);
        
        // Clear for RCL test
        calc.command_buffer.clear();
        
        // Test RCL command buffer progression
        calc.process_input("r").unwrap();
        calc.process_input("c").unwrap();
        calc.process_input("l").unwrap();
        assert_eq!(calc.command_buffer, "rcl ");  // Space should be added automatically
        
        let display = calc.get_display();
        assert!(display.contains("CMD: [rcl __]"), "Display should show 'CMD: [rcl __]' after typing rcl, but got: {}", display);
    }

    #[test]
    fn test_sto_rcl_execution() {
        let mut calc = HP41CCalculator::new();
        
        // Put 42.0 in X register
        calc.stack[0] = 42.0;
        calc.display_value = 42.0;
        
        // Store in register 05
        calc.process_input("s").unwrap();
        calc.process_input("t").unwrap();
        calc.process_input("o").unwrap();
        calc.process_input("0").unwrap();
        let result = calc.process_input("5").unwrap();
        
        assert_eq!(result, Some("STO 5".to_string()));
        assert_eq!(calc.storage_registers[5], 42.0);
        assert!(calc.command_buffer.is_empty(), "Command buffer should be cleared after execution");
        
        // Clear X register
        calc.stack[0] = 0.0;
        calc.display_value = 0.0;
        
        // Recall from register 05
        calc.process_input("r").unwrap();
        calc.process_input("c").unwrap();
        calc.process_input("l").unwrap();
        calc.process_input("0").unwrap();
        let result = calc.process_input("5").unwrap();
        
        assert_eq!(result, Some("RCL 5".to_string()));
        assert_eq!(calc.stack[0], 42.0);
        assert_eq!(calc.display_value, 42.0);
    }

    #[test]
    fn test_fix_sci_eng_immediate_execution() {
        let mut calc = HP41CCalculator::new();
        
        // Test FIX
        calc.process_input("f").unwrap();
        calc.process_input("i").unwrap();
        calc.process_input("x").unwrap();
        
        // Debug: print the exact command buffer
        println!("Command buffer: '{}'", calc.command_buffer);
        println!("Command buffer len: {}", calc.command_buffer.len());
        assert_eq!(calc.command_buffer, "fix ");
        
        let display = calc.get_display();
        assert!(display.contains("CMD: [fix _]"), "Display should show 'CMD: [fix _]' after typing fix, but got: {}", display);
        
        let result = calc.process_input("6").unwrap();
        assert_eq!(result, Some("FIX 6".to_string()));
        assert_eq!(calc.display_formatter.digits, 6);
        assert_eq!(calc.display_formatter.mode, DisplayMode::Fix);
        
        // Test SCI
        calc.process_input("s").unwrap();
        calc.process_input("c").unwrap();
        calc.process_input("i").unwrap();
        let result = calc.process_input("3").unwrap();
        assert_eq!(result, Some("SCI 3".to_string()));
        assert_eq!(calc.display_formatter.digits, 3);
        assert_eq!(calc.display_formatter.mode, DisplayMode::Sci);
    }

    #[test]
    fn test_immediate_command_execution() {
        let mut calc = HP41CCalculator::new();
        
        // Set up stack for testing
        calc.stack[0] = 0.5;  // X
        calc.display_value = 0.5;
        
        // Test that sin executes immediately after typing 'n'
        calc.process_input("s").unwrap();
        calc.process_input("i").unwrap();
        let result = calc.process_input("n").unwrap();
        
        assert_eq!(result, None);  // sin doesn't return a message
        assert!((calc.stack[0] - 0.5_f64.sin()).abs() < 1e-10);
        assert!(calc.command_buffer.is_empty());
    }

    #[test]
    fn test_number_entry_with_underscore() {
        let mut calc = HP41CCalculator::new();
        
        calc.process_input("1").unwrap();
        calc.process_input("2").unwrap();
        calc.process_input("3").unwrap();
        
        let display = calc.get_display();
        assert!(display.contains("X: 123_"), "Number entry should show underscore");
        
        calc.process_input(".").unwrap();
        calc.process_input("4").unwrap();
        
        let display = calc.get_display();
        assert!(display.contains("X: 123.4_"), "Decimal number entry should show underscore");
    }

    #[test]
    fn test_backspace_in_sto_rcl() {
        let mut calc = HP41CCalculator::new();
        
        // Type sto 4, then backspace
        calc.process_input("s").unwrap();
        calc.process_input("t").unwrap();
        calc.process_input("o").unwrap();
        calc.process_input("4").unwrap();
        assert_eq!(calc.command_buffer, "sto 4");
        
        calc.process_input("\u{8}").unwrap();  // Backspace
        assert_eq!(calc.command_buffer, "sto ");
        
        // Now type 7 and it should execute sto 07
        calc.process_input("7").unwrap();
        assert_eq!(calc.command_buffer, "sto 7");
        
        let result = calc.process_input("2").unwrap();
        assert_eq!(result, Some("STO 72".to_string()));
    }

    #[test]
    fn test_stack_operations() {
        let (calc, _) = process_keys(&["5", "enter", "3", "+"]); 
        assert_eq!(calc.stack[0], 8.0);
        
        let (calc, _) = process_keys(&["5", "enter", "3", "-"]);
        assert_eq!(calc.stack[0], 2.0);
        
        let (calc, _) = process_keys(&["5", "enter", "3", "*"]);
        assert_eq!(calc.stack[0], 15.0);
        
        let (calc, _) = process_keys(&["6", "enter", "3", "/"]);
        assert_eq!(calc.stack[0], 2.0);
    }

    #[test]
    fn test_stack_lift_behavior() {
        let mut calc = HP41CCalculator::new();
        
        // Enter 1, 2, 3, 4 onto stack
        calc.process_input("1").unwrap();
        calc.process_input("enter").unwrap();
        calc.process_input("2").unwrap();
        calc.process_input("enter").unwrap();
        calc.process_input("3").unwrap();
        calc.process_input("enter").unwrap();
        calc.process_input("4").unwrap();
        
        // Stack should be T:1, Z:2, Y:3, X:4
        assert_eq!(calc.stack[3], 1.0, "T register should be 1.0");  
        assert_eq!(calc.stack[2], 2.0, "Z register should be 2.0");  
        assert_eq!(calc.stack[1], 3.0, "Y register should be 3.0");  
        assert_eq!(calc.stack[0], 4.0, "X register should be 4.0");  
    }
}