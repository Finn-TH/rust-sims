use super::reactor::Reactor; // Change from `crate::reactor` to `super::reactor`
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

pub struct Control;

impl Control {
    pub fn new() -> Self {
        Control
    }

    pub async fn monitor_and_regulate(
        &self,
        reactor: &Reactor,
        shutdown_flag: Arc<AtomicBool>,
        mut control_rx: mpsc::Receiver<()>,
    ) {
        let mut previous_power = reactor.get_power_output();
        let mut previous_temperature = reactor.get_temperature();

        while (control_rx.recv().await).is_some() {
            let temp = reactor.get_temperature();
            let power = reactor.get_power_output();
            let radiation = reactor.get_radiation_level();

            println!(
                "CtrlRoom: T: {}°C | P: {}w | R: {} Bq",
                temp, power, radiation
            );

            // Detect power surge by comparing the current and previous power levels
            let power_change = power - previous_power;

            if power_change > 50 {
                println!(
                    "\n* CtrlRoom: Power Surge Detected! Increase of {}w",
                    power_change
                );

                let reduction_amount = power_change.min(60); // Cap the reduction at 60w
                reactor.decrease_power_output(reduction_amount);
                println!(
                    " - Action: Emergency Power Reduction | Decreasing by {}w\n",
                    reduction_amount
                );
            }

            // Detect temperature spike by comparing the current and previous temperature levels
            let temp_change = temp - previous_temperature;

            if temp_change > 50 {
                println!(
                    "\n* CtrlRoom: Temperature Spike Detected! Increase of {}°C",
                    temp_change
                );

                let cooling_amount = temp_change.min(60); // Cap the cooling at 60°C
                reactor.decrease_temperature(cooling_amount);
                println!(
                    " - Action: Emergency Cooling Activated | Reducing by {}°C\n",
                    cooling_amount
                );
            }

            // Detect radiation leak
            if radiation > 5000 {
                println!("\n==================== Critical Failure ====================\n");
                println!(
                    "CtrlRoom: Major Radiation Leak Detected! Initiating Full Shutdown and Evacuation!\n"
                );
                reactor.initiate_shutdown(); // Trigger the reactor shutdown
                shutdown_flag.store(true, Ordering::SeqCst);
                self.initiate_evacuation().await;
                break;
            }

            // Check for critical conditions first
            if power >= 300 {
                println!("\n==================== Critical Failure ====================\n");
                println!("CtrlRoom: Critical P Reached! Initiating Shutdown!\n");
                shutdown_flag.store(true, Ordering::SeqCst);
                reactor.initiate_shutdown(); // Trigger the reactor shutdown
                break;
            }

            // Normal regulation logic
            if temp < 150 {
                reactor.increase_temperature(20);
                println!(" - Action: Temp Low | Increasing");
            } else if temp > 250 {
                reactor.decrease_temperature(20);
                println!(" - Action: Temp High | Decreasing");
            } else if !(180..=220).contains(&temp) {
                reactor.increase_temperature(5);
                println!(" - Status: Keeping T Stable");
            }

            if power < 150 {
                reactor.increase_power_output(20);
                println!(" - Action: Power Low | Increasing");
            } else if power > 250 && power < 300 {
                reactor.decrease_power_output(20);
                println!(" - Action: Power High | Decreasing");
            } else if !(180..=220).contains(&power) {
                reactor.increase_power_output(5);
                println!(" - Status: Keeping P Stable");
            }

            previous_power = power;
            previous_temperature = temp;
        }
    }

    async fn initiate_evacuation(&self) {
        println!("\n==================== Evacuation ====================\n");
        for i in (1..=10).rev() {
            println!("Evacuation in progress... {} seconds remaining!", i);
            sleep(Duration::from_secs(1)).await;
        }
        println!("\nEvacuation complete. Shutting down the reactor.\n");
        std::process::exit(0); // This exits the program after the evacuation
    }
}
