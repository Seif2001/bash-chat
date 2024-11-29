use rustyline::Editor;

fn run() {
    // Create an instance of the Editor (a REPL-like interface)
    let mut rl = Editor::<()>::new().unwrap();

    println!("Welcome to the interactive CLI!");
    println!("Type 'exit' to quit.");

    loop {
        // Read user input
        let readline = rl.readline(">> ");
        
        match readline {
            Ok(input) => {
                // Trim the input and check if it's the "exit" command
                let trimmed = input.trim();
                if trimmed == "exit" {
                    println!("Goodbye!");
                    break;  // Exit the loop if the user types 'exit'
                } else {
                    // Otherwise, process the command and display a response
                    println!("You typed: {}", trimmed);
                }
            }
            Err(_) => {
                // If there's an error reading input, break the loop
                println!("Error reading input, exiting.");
                break;
            }
        }
    }
}
