use super::weather_machine::WeatherMachine;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::mpsc;

pub struct Control;

impl Control {
    pub fn new() -> Self {
        Control
    }

    pub async fn monitor_and_regulate(
        &self,
        weather_machine: &WeatherMachine,
        shutdown_flag: Arc<AtomicBool>,
        mut control_rx: mpsc::Receiver<()>,
    ) {
        let mut is_first_run = true;

        while (control_rx.recv().await).is_some() {
            // Check if the system is in shutdown mode
            if shutdown_flag.load(Ordering::SeqCst) {
                break;
            }

            // Check for catastrophic event
            if let Some(event) = weather_machine.get_catastrophic_event() {
                let temp = weather_machine.get_temperature();
                let wind_speed = weather_machine.get_wind_speed();
                let damage_amount = 100; // Set an immense damage amount

                println!("Warning: Catastrophic event detected: {}!", event);
                println!(
                    "Last recorded data - Temp: {}°C, Wind Speed: {} km/h",
                    temp, wind_speed
                );
                println!(
                    "Incoming damage: {}%. Activating Bunker Mode for survival.",
                    damage_amount
                );

                std::process::exit(0); // Immediately exit the program
            }

            let temp = weather_machine.get_temperature();
            let wind_speed = weather_machine.get_wind_speed();
            let structural_health = weather_machine.get_structural_health();

            if is_first_run {
                println!("Monitoring Atmospheric Conditions\n");
                is_first_run = false;
            }

            println!(
                "Temp: {}°C\nWind Speed: {} km/h\nMachine Structural Health: {}%\n",
                temp, wind_speed, structural_health
            );

            // Trigger healing mode if structural health drops below 70%
            if structural_health < 70 {
                println!("Warning: Structural health critical! Initiating repair protocol.");
                weather_machine.repair_structural_health(20); // Repair by 20%
                println!("Repair complete. Structural health restored.\n");
            }

            // Simulate a shutdown condition if structural health drops to 0
            if structural_health == 0 {
                println!("Critical failure: Structural health at 0%! Shutting down.");
                std::process::exit(0); // Immediately exit the program
            }

            // Sleep for a brief moment to simulate real-time monitoring
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    }
}
