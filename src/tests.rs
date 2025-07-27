use crate::*;

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
    fn test_basic_arithmetic() {
        let (calc, _) = process_keys(&["5", "enter", "3", "+"]);
        assert_eq!(calc.test_get_stack()[0], 8.0);
        
        let (calc, _) = process_keys(&["5", "enter", "3", "-"]);
        assert_eq!(calc.test_get_stack()[0], 2.0);
        
        let (calc, _) = process_keys(&["5", "enter", "3", "*"]);
        assert_eq!(calc.test_get_stack()[0], 15.0);
        
        let (calc, _) = process_keys(&["6", "enter", "3", "/"]);
        assert_eq!(calc.test_get_stack()[0], 2.0);
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
        
        let stack = calc.test_get_stack();
        // Stack should be T:1, Z:2, Y:3, X:4
        assert_eq!(stack[3], 1.0, "T register should be 1.0");  
        assert_eq!(stack[2], 2.0, "Z register should be 2.0");  
        assert_eq!(stack[1], 3.0, "Y register should be 3.0");  
        assert_eq!(stack[0], 4.0, "X register should be 4.0");  
    }

    #[test]
    fn test_sto_rcl_execution() {
        let mut calc = HP41CCalculator::new();
    
        // Put 42.0 in X register
        calc.process_input("4").unwrap();
        calc.process_input("2").unwrap();
    
        // Store in register 5
        calc.process_input("s").unwrap();
        calc.process_input("t").unwrap();
        calc.process_input("o").unwrap();
        calc.process_input(" ").unwrap();
        calc.process_input("5").unwrap();
    
        // Check that it was stored
        assert_eq!(calc.test_get_storage(5), Some(42.0));
    
        // Clear X register by entering 0
        calc.process_input("0").unwrap();
        assert_eq!(calc.test_get_stack()[0], 0.0);
    
        // Recall from register 5 (same calculator!)
        calc.process_input("r").unwrap();
        calc.process_input("c").unwrap();
        calc.process_input("l").unwrap();
        calc.process_input(" ").unwrap();
        calc.process_input("5").unwrap();
    
        // Check that it was recalled
        assert_eq!(calc.test_get_stack()[0], 42.0);
    }

    #[test]
    fn test_display_modes() {
        // Test FIX mode
    	let (_calc, messages) = process_keys(&["f", "i", "x", " ", "6"]);
    	assert!(messages.iter().any(|msg| msg.contains("FIX 6")));
    
        // Test SCI mode  
    	let (_calc, messages) = process_keys(&["s", "c", "i", " ", "3"]);
	    assert!(messages.iter().any(|msg| msg.contains("SCI 3")));
    
        // Test ENG mode
	let (_calc, messages) = process_keys(&["e", "n", "g", " ", "2"]);
	assert!(messages.iter().any(|msg| msg.contains("ENG 2")));
    }

    #[test]
    fn test_math_functions() {
        let mut calc = HP41CCalculator::new();
        
        // Test sin(pi/2) = 1
        calc.process_input("p").unwrap();
        calc.process_input("i").unwrap(); // This should complete "pi"
        calc.process_input("2").unwrap();
        calc.process_input("/").unwrap();
        calc.process_input("s").unwrap();
        calc.process_input("i").unwrap();
        calc.process_input("n").unwrap();
        
        let result = calc.test_get_stack()[0];
        assert!((result - 1.0).abs() < 1e-10, "sin(pi/2) should be 1.0, got {}", result);
    }

    #[test]
    fn test_programming_mode_toggle() {
        let mut calc = HP41CCalculator::new();
        
        let result = calc.process_input(":").unwrap();
        assert!(result.is_some());
        assert!(result.unwrap().contains("Programming mode ON"));
        
        let result = calc.process_input(":").unwrap();
        assert!(result.is_some());
        assert!(result.unwrap().contains("Programming mode OFF"));
    }

    #[test]
    fn test_number_entry_with_decimal() {
        let mut calc = HP41CCalculator::new();
        
        calc.process_input("1").unwrap();
        calc.process_input("2").unwrap();
        calc.process_input("3").unwrap();
        calc.process_input(".").unwrap();
        calc.process_input("4").unwrap();
        calc.process_input("5").unwrap();
        
        let stack = calc.test_get_stack();
        assert_eq!(stack[0], 123.45);
    }

    #[test]
    fn test_backspace_behavior() {
        let mut calc = HP41CCalculator::new();
        
        // Enter a number
        calc.process_input("1").unwrap();
        calc.process_input("2").unwrap();
        calc.process_input("3").unwrap();
        
        // Backspace should remove last digit
        calc.process_input("\u{8}").unwrap();  // Backspace
        
        let stack = calc.test_get_stack();
        assert_eq!(stack[0], 12.0);
    }

    #[test]
    fn test_clear_operations() {
        let mut calc = HP41CCalculator::new();
        
        // Put some values on stack
        calc.process_input("1").unwrap();
        calc.process_input("enter").unwrap();
        calc.process_input("2").unwrap();
        calc.process_input("enter").unwrap();
        calc.process_input("3").unwrap();
        
        // Test CLX (clear X)
        calc.process_input("c").unwrap();
        calc.process_input("l").unwrap();
        calc.process_input("x").unwrap();
        
        let stack = calc.test_get_stack();
        assert_eq!(stack[0], 0.0);  // X should be 0
        assert_eq!(stack[1], 2.0);  // Y should still be 2
        
        // Test CLR (clear all)
        calc.process_input("c").unwrap();
        calc.process_input("l").unwrap();
        calc.process_input("r").unwrap();
        
        let stack = calc.test_get_stack();
        assert_eq!(stack[0], 0.0);
        assert_eq!(stack[1], 0.0);
        assert_eq!(stack[2], 0.0);
        assert_eq!(stack[3], 0.0);
    }

    #[test]
    fn test_swap_operation() {
        let mut calc = HP41CCalculator::new();
        
        // Put 5 in X and 3 in Y
        calc.process_input("5").unwrap();
        calc.process_input("enter").unwrap();
        calc.process_input("3").unwrap();
        
        // Swap X and Y
        calc.process_input("s").unwrap();
        calc.process_input("w").unwrap();
        calc.process_input("a").unwrap();
        calc.process_input("p").unwrap();
        
        let stack = calc.test_get_stack();
        assert_eq!(stack[0], 5.0);  // X should now be 5
        assert_eq!(stack[1], 3.0);  // Y should now be 3
    }

    #[test]
    fn test_change_sign() {
        let mut calc = HP41CCalculator::new();
        
        calc.process_input("5").unwrap();
        calc.process_input("c").unwrap();
        calc.process_input("h").unwrap();
        calc.process_input("s").unwrap();
        
        let stack = calc.test_get_stack();
        assert_eq!(stack[0], -5.0);
    }

    #[test]
    fn test_constants() {
        let mut calc = HP41CCalculator::new();
        
        // Test pi constant
        calc.process_input("p").unwrap();
        calc.process_input("i").unwrap();
        
        let stack = calc.test_get_stack();
        assert!((stack[0] - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn test_display_output() {
        let calc = HP41CCalculator::new();
        let display = calc.get_display();
        
        // Display should contain stack registers
        assert!(display.contains("T:"));
        assert!(display.contains("Z:"));
        assert!(display.contains("Y:"));
        assert!(display.contains("X:"));
        
        // Display should contain command reference
        assert!(display.contains("sin cos tan"));
    }
}
#[test]
fn debug_display_command() {
    let mut calc = HP41CCalculator::new();
    
    println!("=== Debug Display Command ===");
    
    let result1 = calc.process_input("f");
    println!("After 'f': {:?}, buffer: '{}'", result1, calc.test_get_command_buffer());
    
    let result2 = calc.process_input("i");
    println!("After 'i': {:?}, buffer: '{}'", result2, calc.test_get_command_buffer());
    
    let result3 = calc.process_input("x");
    println!("After 'x': {:?}, buffer: '{}'", result3, calc.test_get_command_buffer());
    
    let result4 = calc.process_input(" ");
    println!("After ' ': {:?}, buffer: '{}'", result4, calc.test_get_command_buffer());
    
    let result5 = calc.process_input("6");
    println!("After '6': {:?}, buffer: '{}'", result5, calc.test_get_command_buffer());
    
    println!("Display mode: {:?}, digits: {}", calc.test_get_display_mode(), calc.test_get_display_digits());
}

#[test]
fn debug_storage_command() {
    let mut calc = HP41CCalculator::new();
    
    println!("=== Debug Storage Command ===");
    
    // Put 42 in X
    calc.process_input("4").unwrap();
    calc.process_input("2").unwrap();
    println!("After entering 42: X = {}", calc.test_get_stack()[0]);
    
    // Try STO 5 step by step
    let result1 = calc.process_input("s");
    println!("After 's': {:?}, buffer: '{}'", result1, calc.test_get_command_buffer());
    
    let result2 = calc.process_input("t");
    println!("After 't': {:?}, buffer: '{}'", result2, calc.test_get_command_buffer());
    
    let result3 = calc.process_input("o");
    println!("After 'o': {:?}, buffer: '{}'", result3, calc.test_get_command_buffer());
    
    let result4 = calc.process_input(" ");
    println!("After ' ': {:?}, buffer: '{}'", result4, calc.test_get_command_buffer());
    
    let result5 = calc.process_input("5");
    println!("After '5': {:?}, buffer: '{}'", result5, calc.test_get_command_buffer());
    
    println!("Storage register 5: {:?}", calc.test_get_storage(5));
    println!("X register: {}", calc.test_get_stack()[0]);
}

#[test]
fn debug_recall_command() {
    let mut calc = HP41CCalculator::new();
    
    println!("=== Debug Recall Command ===");
    
    // First, manually store 42 in register 5
    calc.test_set_x_register(42.0);
    let store_result = calc.execute_command("sto", Some(vec!["5".to_string()]));
    println!("Manual STO result: {:?}", store_result);
    println!("Storage register 5 after manual store: {:?}", calc.test_get_storage(5));
    
    // Clear X register
    calc.test_set_x_register(0.0);
    println!("X register cleared to: {}", calc.test_get_stack()[0]);
    
    // Now try RCL step by step
    let result1 = calc.process_input("r");
    println!("After 'r': {:?}, buffer: '{}'", result1, calc.test_get_command_buffer());
    
    let result2 = calc.process_input("c");
    println!("After 'c': {:?}, buffer: '{}'", result2, calc.test_get_command_buffer());
    
    let result3 = calc.process_input("l");
    println!("After 'l': {:?}, buffer: '{}'", result3, calc.test_get_command_buffer());
    
    let result4 = calc.process_input(" ");
    println!("After ' ': {:?}, buffer: '{}'", result4, calc.test_get_command_buffer());
    
    let result5 = calc.process_input("5");
    println!("After '5': {:?}, buffer: '{}'", result5, calc.test_get_command_buffer());
    
    println!("Final X register: {}", calc.test_get_stack()[0]);
}
