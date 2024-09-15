use std::io;

// Declare the modules for all simulations.
pub mod sim_cafe;
pub mod sim_factory;
pub mod sim_home;
pub mod sim_nuclear;
pub mod sim_weather; // Add the Weather Machine simulation module

#[tokio::main]
async fn main() {
    loop {
        println!("\n=== Simulation Menu ===");
        println!("1. Cafe Simulation");
        println!("2. Factory Simulation");
        println!("3. Home Automation Simulation");
        println!("4. Nuclear Reactor Simulation");
        println!("5. Weather Machine Simulation");
        println!("0. Exit");

        // Get user input
        let mut choice = String::new();
        io::stdin()
            .read_line(&mut choice)
            .expect("Failed to read input");

        match choice.trim() {
            "1" => {
                println!("Running Cafe Simulation...");
                sim_cafe::run().expect("Cafe simulation failed"); // Run the Cafe simulation
            }
            "2" => {
                println!("Running Factory Simulation...");
                sim_factory::run(); // Now running the Factory simulation
            }
            "3" => {
                println!("Running Home Automation Simulation...");
                sim_home::run().expect("Home automation simulation failed"); // Run the Home Automation simulation
            }
            "4" => {
                println!("Running Nuclear Reactor Simulation...");
                sim_nuclear::run().await; // Run the async Nuclear simulation
            }
            "5" => {
                println!("Running Weather Machine Simulation...");
                sim_weather::run().await; // Run the async Weather Machine simulation
            }
            "0" => {
                println!("Exiting...");
                break; // Exit the loop to stop the program
            }
            _ => {
                println!("Invalid choice! Please select a valid number.");
            }
        }
    }
}
