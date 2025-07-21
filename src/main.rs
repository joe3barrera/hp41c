use std::io;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};

use hp41c::HP41CCalculator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut calc = HP41CCalculator::new();

    // Enable raw mode
    terminal::enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;

    // Ensure we clean up on exit
    let result = run_calculator(&mut calc);

    // Cleanup
    terminal::disable_raw_mode()?;
    io::stdout().execute(LeaveAlternateScreen)?;

    result
}

fn run_calculator(calc: &mut HP41CCalculator) -> Result<(), Box<dyn std::error::Error>> {
    println!("HP-41C Calculator Emulator v0.5.0 (Rust)\r");
    println!("==========================================\r");
    println!("Enter ':' to toggle programming mode\r");
    println!("Enter 'q' to quit, 'F' to toggle flags\r");
    println!("\r");

    loop {
        // Clear screen and show display
        print!("\x1B[2J\x1B[H"); // Clear screen and move cursor to top-left
        println!("HP-41C Calculator Emulator v0.5.0 (Rust)\r");
        println!("==========================================\r");
        println!("Enter ':' to toggle programming mode\r");
        println!("Enter 'q' to quit, 'F' to toggle flags\r");
        println!("\r");
        
        // Display calculator state
        let display = calc.get_display();
        for line in display.lines() {
            println!("{}\r", line);
        }
        println!("\r");

        // Read a single key
        if let Event::Key(KeyEvent { code, modifiers, kind, .. }) = event::read()? {
            // Only process key press events, ignore key release events
            if kind != KeyEventKind::Press {
                continue;
            }
            
            match code {
                KeyCode::Char('c') if modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => break,
                KeyCode::Char('F') => {
                    // Toggle flags display
                    calc.show_flags = !calc.show_flags;
                }
                KeyCode::Char('q') => break,
                KeyCode::Esc => break,
                KeyCode::Enter => {
                    // Enter key behavior depends on context
                    if !calc.command_buffer.is_empty() {
                        // If we have any command buffer content, process it as a command
                        match calc.process_command() {
                            Ok(Some(msg)) => {
                                println!("\r>>> {}\r", msg);
                                std::thread::sleep(std::time::Duration::from_millis(500));
                            }
                            Err(msg) => {
                                println!("\r>>> ERROR: {}\r", msg);
                                std::thread::sleep(std::time::Duration::from_millis(500));
                            }
                            Ok(None) => {}
                        }
                    } else {
                        // Execute ENTER command (stack lift) - this handles number entry completion
                        match calc.process_input("enter") {
                            Ok(Some(msg)) => {
                                println!("\r>>> {}\r", msg);
                                std::thread::sleep(std::time::Duration::from_millis(500));
                            }
                            Err(msg) => {
                                println!("\r>>> ERROR: {}\r", msg);
                                std::thread::sleep(std::time::Duration::from_millis(500));
                            }
                            Ok(None) => {}
                        }
                    }
                }
                KeyCode::Char(' ') => {
                    // Space adds to command buffer or completes commands
                    if !calc.command_buffer.is_empty() {
                        // Add space to command buffer to separate command from arguments
                        calc.command_buffer.push(' ');
                    }
                }
                KeyCode::Char(c) => {
                    // Handle digits and letters differently
                    if c.is_ascii_digit() || c == '.' {
                        // Process digits immediately as number input
                        match calc.process_input(&c.to_string()) {
                            Ok(Some(msg)) => {
                                println!("\r>>> {}\r", msg);
                                std::thread::sleep(std::time::Duration::from_millis(500));
                            }
                            Err(msg) => {
                                println!("\r>>> ERROR: {}\r", msg);
                                std::thread::sleep(std::time::Duration::from_millis(500));
                            }
                            Ok(None) => {}
                        }
                    } else if c.is_ascii_alphabetic() {
                        // Process letters as command building
                        match calc.process_input(&c.to_string()) {
                            Ok(Some(msg)) => {
                                println!("\r>>> {}\r", msg);
                                std::thread::sleep(std::time::Duration::from_millis(500));
                            }
                            Err(msg) => {
                                println!("\r>>> ERROR: {}\r", msg);
                                std::thread::sleep(std::time::Duration::from_millis(500));
                            }
                            Ok(None) => {}
                        }
                    } else {
                        // Handle other characters (operators, etc.)
                        match calc.process_input(&c.to_string()) {
                            Ok(Some(msg)) => {
                                println!("\r>>> {}\r", msg);
                                std::thread::sleep(std::time::Duration::from_millis(500));
                            }
                            Err(msg) => {
                                println!("\r>>> ERROR: {}\r", msg);
                                std::thread::sleep(std::time::Duration::from_millis(500));
                            }
                            Ok(None) => {}
                        }
                    }
                }
                KeyCode::Backspace => {
                    match calc.process_input("\u{8}") {
                        Ok(Some(msg)) => {
                            println!("\r>>> {}\r", msg);
                            std::thread::sleep(std::time::Duration::from_millis(500));
                        }
                        Err(msg) => {
                            println!("\r>>> ERROR: {}\r", msg);
                            std::thread::sleep(std::time::Duration::from_millis(500));
                        }
                        Ok(None) => {}
                    }
                }
                KeyCode::Delete => {
                    match calc.process_input("\u{7f}") {
                        Ok(Some(msg)) => {
                            println!("\r>>> {}\r", msg);
                            std::thread::sleep(std::time::Duration::from_millis(500));
                        }
                        Err(msg) => {
                            println!("\r>>> ERROR: {}\r", msg);
                            std::thread::sleep(std::time::Duration::from_millis(500));
                        }
                        Ok(None) => {}
                    }
                }
                _ => continue, // Ignore other keys
            }
        }
    }

    Ok(())
}