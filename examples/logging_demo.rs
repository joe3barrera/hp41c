/// Quick demonstration of the HP-41C logging system
/// Add this as examples/logging_demo.rs to see it in action

use hp41c::{HP41CCalculator, Logger};

fn main() {
    println!("🎯 HP-41C Calculator Logging Demo 🎯\n");
    
    // Create calculator and enable minimal logging
    let mut calc = HP41CCalculator::new();
    calc.logger_mut().log_flags = true;
    calc.logger_mut().log_stack = true;
    println!("✅ Created calculator with minimal logging: {}\n", calc.logger().get_config_string());
    
    // Demonstrate flag changes
    println!("🚩 Testing flag changes:");
    calc.process_input("F").unwrap(); // Toggle flags display - should log flag change
    calc.process_input("F").unwrap(); // Toggle again
    println!();
    
    // Demonstrate stack operations  
    println!("📚 Testing stack operations:");
    calc.process_input("5").unwrap();    // Enter 5
    calc.process_input("enter").unwrap(); // ENTER (should log stack operation)
    calc.process_input("3").unwrap();    // Enter 3  
    calc.process_input("+").unwrap();    // Add (should log stack operation)
    println!();
    
    // Enable full logging for more complex operations
    println!("🔍 Enabling full debug logging:");
    *calc.logger_mut() = Logger::debug_all();
    println!("Logger config: {}\n", calc.logger().get_config_string());
    
    // Demonstrate storage operations with full logging
    println!("💾 Testing storage operations with full logging:");
    calc.process_input("s").unwrap();  // Start STO command
    calc.process_input("t").unwrap();  // Continue building
    calc.process_input("o").unwrap();  // Complete STO
    calc.process_input("0").unwrap();  // First digit of register
    calc.process_input("5").unwrap();  // Second digit - should execute and log
    println!();
    
    // Clear X and recall to see more logging
    calc.process_input("c").unwrap();  // Start CLX
    calc.process_input("l").unwrap();  // Continue
    calc.process_input("x").unwrap();  // Complete CLX
    
    calc.process_input("r").unwrap();  // Start RCL
    calc.process_input("c").unwrap();  // Continue
    calc.process_input("l").unwrap();  // Complete RCL  
    calc.process_input("0").unwrap();  // First digit
    calc.process_input("5").unwrap();  // Second digit - should execute and log
    println!();
    
    // Test different logging configurations
    println!("⚙️  Testing custom logging configurations:");
    
    // Input + commands only
    calc.logger_mut().reset();
    calc.logger_mut().log_input = true;
    calc.logger_mut().log_commands = true;
    println!("Custom config: {}", calc.logger().get_config_string());
    
    calc.process_input("2").unwrap();
    calc.process_input("*").unwrap();
    println!();
    
    // Programming mode with logging
    calc.logger_mut().log_programming = true;
    println!("📝 Testing programming mode with logging:");
    calc.process_input(":").unwrap(); // Enter programming mode
    calc.process_input("1").unwrap(); // Add instruction
    calc.process_input("2").unwrap(); // Add instruction
    calc.process_input(":").unwrap(); // Exit programming mode
    println!();
    
    println!("🎉 Logging demo complete!");
    println!("\n💡 In the actual calculator app, use these controls:");
    println!("   L        - Toggle logging on/off");  
    println!("   Ctrl+A   - Enable ALL logging");
    println!("   Ctrl+M   - Minimal logging (flags + stack)");
    println!("   Ctrl+O   - Turn OFF all logging");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_logging_doesnt_break_calculation() {
        // Verify that logging doesn't interfere with calculations
        let mut calc_no_log = HP41CCalculator::new();
        let mut calc_with_log = HP41CCalculator::new();
        calc_with_log.logger_mut().log_stack = true;
        calc_with_log.logger_mut().log_flags = true;
        calc_with_log.logger_mut().log_input = true;
        
        // Perform the same operations on both
        let ops = ["5", "enter", "3", "+", "2", "*"];
        
        for op in &ops {
            calc_no_log.process_input(op).unwrap();
            calc_with_log.process_input(op).unwrap();
        }
        
        // Results should be identical
        assert_eq!(calc_no_log.test_get_stack(), calc_with_log.test_get_stack());
        assert_eq!(calc_no_log.test_get_stack()[0], 16.0); // (5+3)*2 = 16
    }
}