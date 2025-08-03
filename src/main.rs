use std::io;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
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
    println!("HP-41C Calculator Emulator v0.5.0 (Rust) - With Debug Logging\r");
    println!("================================================================\r");
    println!("Enter ':' to toggle programming mode\r");
    println!("Enter 'q' to quit, 'F' to toggle flags, 'L' for logging\r");
    println!("Logging shortcuts:\r");
    println!("  Ctrl+L (toggle), Ctrl+A (all), Ctrl+M (minimal), Ctrl+O (off)\r");
    println!("  Ctrl+F (enable file logging), Ctrl+D (disable file logging)\r");
    println!("\r");

    loop {
        // Clear screen and show display
        print!("\x1B[2J\x1B[H"); // Clear screen and move cursor to top-left
        println!("HP-41C Calculator Emulator v0.5.0 (Rust) - With Debug Logging\r");
        println!("================================================================\r");
        println!("Enter ':' to toggle programming mode\r");
        println!("Enter 'q' to quit, 'F' to toggle flags, 'L' for logging\r");
        println!("Logging shortcuts:\r");
        println!("  Ctrl+L (toggle), Ctrl+A (all), Ctrl+M (minimal), Ctrl+O (off)\r");
        println!("  Ctrl+F (enable file logging), Ctrl+D (disable file logging)\r");
        
        // Show current log file if active
        if let Some(path) = calc.get_log_file_path() {
            println!("  📄 Logging to: {}\r", path.display());
        }
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
                KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => break,
                KeyCode::Char('q') => break,
                KeyCode::Esc => break,
                
                // Logging control shortcuts
                KeyCode::Char('l') if modifiers.contains(KeyModifiers::CONTROL) => {
                    if let Some(msg) = calc.toggle_logging() {
                        println!("\r>>> {}\r", msg);
                        std::thread::sleep(std::time::Duration::from_millis(1000));
                    }
                }
                KeyCode::Char('a') if modifiers.contains(KeyModifiers::CONTROL) => {
                    if let Some(msg) = calc.configure_logger("all") {
                        println!("\r>>> {}\r", msg);
                        std::thread::sleep(std::time::Duration::from_millis(1000));
                    }
                }
                KeyCode::Char('m') if modifiers.contains(KeyModifiers::CONTROL) => {
                    if let Some(msg) = calc.configure_logger("minimal") {
                        println!("\r>>> {}\r", msg);
                        std::thread::sleep(std::time::Duration::from_millis(1000));
                    }
                }
                KeyCode::Char('o') if modifiers.contains(KeyModifiers::CONTROL) => {
                    if let Some(msg) = calc.configure_logger("off") {
                        println!("\r>>> {}\r", msg);
                        std::thread::sleep(std::time::Duration::from_millis(1000));
                    }
                }
                
                // NEW: File logging controls
                KeyCode::Char('f') if modifiers.contains(KeyModifiers::CONTROL) => {
                    let default_path = "hp41c_debug.log";
                    match calc.enable_file_logging(default_path) {
                        Ok(Some(msg)) => {
                            println!("\r>>> {}\r", msg);
                            println!("\r>>> You can now run: tail -f {} (in another terminal)\r", default_path);
                            std::thread::sleep(std::time::Duration::from_millis(2000));
                        }
                        Ok(None) => {
                            println!("\r>>> File logging enabled\r");
                            std::thread::sleep(std::time::Duration::from_millis(1000));
                        }
                        Err(e) => {
                            println!("\r>>> ERROR: {}\r", e);
                            std::thread::sleep(std::time::Duration::from_millis(1000));
                        }
                    }
                }
                KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) => {
                    match calc.disable_file_logging() {
                        Ok(Some(msg)) => {
                            println!("\r>>> {}\r", msg);
                            std::thread::sleep(std::time::Duration::from_millis(1000));
                        }
                        Ok(None) => {
                            println!("\r>>> File logging disabled\r");
                            std::thread::sleep(std::time::Duration::from_millis(1000));
                        }
                        Err(e) => {
                            println!("\r>>> ERROR: {}\r", e);
                            std::thread::sleep(std::time::Duration::from_millis(1000));
                        }
                    }
                }
                
                KeyCode::Enter => {
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
                KeyCode::Char(' ') => {
                    match calc.process_input(" ") {
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
                KeyCode::Char('L') => {
                    // 'L' key for logging toggle (non-Ctrl)
                    if let Some(msg) = calc.toggle_logging() {
                        println!("\r>>> {}\r", msg);
                        std::thread::sleep(std::time::Duration::from_millis(1000));
                    }
                }
                KeyCode::Char(c) => {
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