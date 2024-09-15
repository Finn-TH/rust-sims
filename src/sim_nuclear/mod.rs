mod control;
mod reactor;

use control::Control;
use reactor::Reactor;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::sync::mpsc;

// The main simulation entry point
pub async fn run() {
    let reactor = Arc::new(Reactor::new());
    let control = Control::new();

    // Shared flag to signal shutdown
    let shutdown_flag = Arc::new(AtomicBool::new(false));

    // Create a channel for communication between reactor and control room
    let (control_tx, control_rx) = mpsc::channel(32);

    // Start the reactor
    reactor.startup().await;

    // Clone the Arc to share the reactor with the spawned task
    let reactor_for_task = Arc::clone(&reactor);

    // Run the reactor in a separate task
    let reactor_task = tokio::spawn(async move {
        reactor_for_task.run(control_tx).await;
    });

    // Run the control room in the main task
    control
        .monitor_and_regulate(&reactor, Arc::clone(&shutdown_flag), control_rx)
        .await;

    // Wait for the reactor task to finish (if ever)
    reactor_task.await.unwrap();

    println!("Simulation ended due to critical failure.");
}
