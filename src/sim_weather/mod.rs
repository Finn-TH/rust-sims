mod control;
mod weather_machine;

use control::Control;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::mpsc;
use weather_machine::WeatherMachine;

pub async fn run() {
    let weather_machine = Arc::new(WeatherMachine::new()); // Use Arc to allow sharing
    let control = Control::new();

    // Shared flag to signal shutdown
    let shutdown_flag = Arc::new(AtomicBool::new(false));

    // Create a channel for communication between weather machine and control system
    let (control_tx, control_rx) = mpsc::channel(32);

    // Start the weather machine
    weather_machine.startup().await;

    // Clone the Arc to share the weather machine with the spawned task
    let weather_machine_for_task = Arc::clone(&weather_machine);

    // Run the weather machine in a separate task
    let weather_machine_task = tokio::spawn(async move {
        weather_machine_for_task.run(control_tx).await;
    });

    // Run the control system in the main task
    control
        .monitor_and_regulate(&weather_machine, Arc::clone(&shutdown_flag), control_rx)
        .await;

    // Wait for the weather machine task to finish (if ever)
    weather_machine_task.await.unwrap();

    println!("Weather machine simulation ended.");
}
