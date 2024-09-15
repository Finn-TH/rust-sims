use rand::Rng;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

pub struct Reactor {
    pub temperature: Arc<Mutex<i32>>,     // Shared state for temperature
    pub power_output: Arc<Mutex<i32>>,    // Shared state for power output
    pub radiation_level: Arc<Mutex<i32>>, // Shared state for radiation level (Bq)
    pub shutdown: Arc<Mutex<bool>>,       // Shutdown flag
    pub allow_radiation_leak: Arc<Mutex<bool>>, // Flag to allow radiation leak
}

impl Reactor {
    pub fn new() -> Self {
        Reactor {
            temperature: Arc::new(Mutex::new(100)), // Initial temperature: 100°C
            power_output: Arc::new(Mutex::new(100)), // Initial power output: 100 watts
            radiation_level: Arc::new(Mutex::new(0)), // Initial radiation level: 0 Bq
            shutdown: Arc::new(Mutex::new(false)),  // Initial shutdown state: false
            allow_radiation_leak: Arc::new(Mutex::new(false)), // Radiation leak not allowed initially
        }
    }

    pub async fn startup(&self) {
        println!("========================= Reactor Initialization =========================");
        println!("Reactor: Startup Success");
        println!(
            "Reactor: Initial Temperature: {}°C",
            *self.temperature.lock().unwrap()
        );
        println!(
            "Reactor: Initial Power Output: {}w",
            *self.power_output.lock().unwrap()
        );
        println!(
            "Reactor: Initial Radiation Reading: {}Bq",
            *self.radiation_level.lock().unwrap()
        );

        println!("\n========================= Efficiency Targets =========================");
        println!("Target Levels: 200w | 200°C | 0 Bq");
        println!("-----------------------------------------------------------------------");
        println!("Temperature (T) | Power (P) | Radiation (R)\n");

        // Enable radiation leak possibility after 15 seconds
        let allow_leak = Arc::clone(&self.allow_radiation_leak);
        tokio::spawn(async move {
            sleep(Duration::from_secs(10)).await;
            let mut allow_leak = allow_leak.lock().unwrap();
            *allow_leak = true;
        });
    }

    pub async fn fluctuate_temperature(&self) {
        if *self.shutdown.lock().unwrap() {
            return; // Do nothing if the reactor is shutting down
        }

        let mut rng = rand::thread_rng();
        let mut fluctuation = rng.gen_range(-30..=30); // Make fluctuation mutable

        // Introduce a random coolant pump failure (15% chance)
        if rng.gen_bool(0.95) {
            println!("\nReactor: Coolant pump failure detected! Temperature spike occurring.\n");
            fluctuation += rng.gen_range(50..=100); // Spike between 50 and 100°C
        }

        let mut temp = self.temperature.lock().unwrap();
        *temp = (*temp + fluctuation).clamp(100, 300); // Clamping between 100°C and 300°C
    }

    pub async fn fluctuate_power_output(&self) {
        if *self.shutdown.lock().unwrap() {
            return; // Do nothing if the reactor is shutting down
        }

        let mut rng = rand::thread_rng();
        let mut fluctuation = rng.gen_range(-30..=30); // Make fluctuation mutable

        // Introduce a random power surge (15% chance)
        if rng.gen_bool(0.15) {
            println!("\nReactor: Electrical malfunction detected! Power surge occurring.\n");
            fluctuation += rng.gen_range(500..=1000); // Surge between 50 and 100 watts
        }

        let mut power = self.power_output.lock().unwrap();
        *power = (*power + fluctuation).clamp(00, 300); // Clamping between 100 watts and 300 watts
    }

    pub async fn maybe_cause_radiation_leak(&self) {
        if *self.shutdown.lock().unwrap() || !*self.allow_radiation_leak.lock().unwrap() {
            return; // Do nothing if the reactor is shutting down or radiation leaks are not allowed yet
        }

        let mut rng = rand::thread_rng();
        if rng.gen_bool(0.10) {
            // 10% chance of causing a radiation leak
            println!(
                "\nReactor: Severe mechanical failure detected! Major radiation leak occurring.\n"
            );
            let mut radiation = self.radiation_level.lock().unwrap();
            *radiation = 10000; // Arbitrary high radiation level to simulate a major leak
        }
    }

    pub fn get_temperature(&self) -> i32 {
        *self.temperature.lock().unwrap()
    }

    pub fn get_power_output(&self) -> i32 {
        *self.power_output.lock().unwrap()
    }

    pub fn get_radiation_level(&self) -> i32 {
        *self.radiation_level.lock().unwrap()
    }

    pub fn decrease_temperature(&self, amount: i32) {
        let mut temp = self.temperature.lock().unwrap();
        *temp = (*temp - amount).max(100); // Ensure it doesn't go below 100°C
    }

    pub fn decrease_power_output(&self, amount: i32) {
        let mut power = self.power_output.lock().unwrap();
        *power = (*power - amount).max(100); // Ensure it doesn't go below 100 watts
    }

    pub fn increase_temperature(&self, amount: i32) {
        let mut temp = self.temperature.lock().unwrap();
        *temp = (*temp + amount).min(300); // Ensure it doesn't go above 300°C
    }

    pub fn increase_power_output(&self, amount: i32) {
        let mut power = self.power_output.lock().unwrap();
        *power = (*power + amount).min(300); // Ensure it doesn't go above 300 watts
    }

    pub async fn run(&self, control_tx: mpsc::Sender<()>) {
        loop {
            if *self.shutdown.lock().unwrap() {
                break; // Exit the loop if the reactor is shutting down
            }

            self.fluctuate_temperature().await;
            self.fluctuate_power_output().await;
            self.maybe_cause_radiation_leak().await;

            if let Err(e) = control_tx.send(()).await {
                eprintln!("\nFailed to send message to control room: {}\n", e);
                break;
            }

            sleep(Duration::from_secs(2)).await;
        }
    }

    pub fn initiate_shutdown(&self) {
        let mut shutdown = self.shutdown.lock().unwrap();
        *shutdown = true; // Set the shutdown flag
    }
}
