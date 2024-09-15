use rand::Rng;
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};

pub struct WeatherMachine {
    pub temperature: Arc<Mutex<i32>>, // Shared state for temperature
    pub wind_speed: Arc<Mutex<i32>>,  // Shared state for wind speed
    pub structural_health: Arc<Mutex<i32>>, // Shared state for structural health
    pub catastrophic_event: Arc<Mutex<Option<String>>>, // Flag for catastrophic event with event type
}

impl WeatherMachine {
    pub fn new() -> Self {
        WeatherMachine {
            temperature: Arc::new(Mutex::new(20)), // Initial temperature: 20°C
            wind_speed: Arc::new(Mutex::new(10)),  // Initial wind speed: 10 km/h
            structural_health: Arc::new(Mutex::new(100)), // Initial structural health: 100%
            catastrophic_event: Arc::new(Mutex::new(None)), // Initially, no catastrophic event
        }
    }

    pub async fn startup(&self) {
        println!("Weather Machine Initialized");
        println!("All Systems Ready\n");
    }

    pub async fn fluctuate_conditions(&self) {
        let mut rng = rand::thread_rng();

        // Simulate wind speed fluctuations with a wider range
        let wind_fluctuation = rng.gen_range(-10..=30);
        let mut wind_speed = self.wind_speed.lock().unwrap();
        *wind_speed = (*wind_speed + wind_fluctuation).clamp(0, 300); // Clamp wind speed between 0 and 300 km/h

        // Adjust temperature inversely proportional to wind speed
        let base_temp = 25; // Base temperature when wind speed is 0
        let temp_decrease = (*wind_speed / 4).clamp(0, 20); // Decrease temp as wind increases
        let mut temp = self.temperature.lock().unwrap();
        *temp = (base_temp - temp_decrease).clamp(0, 50); // Clamp temperature between 0°C and 50°C

        // Apply random damage if wind speed spike occurs
        if rng.gen_bool(0.2) {
            // 20% chance for a wind speed spike
            println!("Warning: Wind speed spike detected! Incurring Structural Damage.");
            let damage_amount = if rng.gen_bool(0.5) { 10 } else { 30 }; // Randomly choose 10% or 30% damage
            let mut health = self.structural_health.lock().unwrap();
            *health = (*health - damage_amount).max(0); // Apply damage, ensuring health doesn't drop below 0%
            println!("Structural health decreased by {}%", damage_amount);
        }

        // Check for a catastrophic event
        if rng.gen_bool(0.05) {
            // 5% chance for a catastrophic event
            let event_type = match rng.gen_range(0..3) {
                0 => "Volcano Eruption",
                1 => "Tornado",
                _ => "Earthquake",
            };

            let mut catastrophic_event = self.catastrophic_event.lock().unwrap();
            *catastrophic_event = Some(event_type.to_string());

            // Set extreme conditions based on the event type
            match event_type {
                "Volcano Eruption" => {
                    *temp = 100; // Extreme temperature due to volcanic heat
                    *wind_speed = 300; // Max wind speed
                }
                "Tornado" => {
                    *temp = 0; // Sudden drop in temperature
                    *wind_speed = 300; // Max wind speed
                }
                "Earthquake" => {
                    *temp = 50; // Moderate temperature increase due to fires
                    *wind_speed = 200; // High wind speed from shockwaves
                }
                _ => {}
            }
        }
    }

    pub fn get_temperature(&self) -> i32 {
        *self.temperature.lock().unwrap()
    }

    pub fn get_wind_speed(&self) -> i32 {
        *self.wind_speed.lock().unwrap()
    }

    pub fn get_structural_health(&self) -> i32 {
        *self.structural_health.lock().unwrap()
    }

    pub fn get_catastrophic_event(&self) -> Option<String> {
        self.catastrophic_event.lock().unwrap().clone()
    }

    pub fn repair_structural_health(&self, amount: i32) {
        let mut health = self.structural_health.lock().unwrap();
        *health = (*health + amount).min(100); // Repair up to 100% health
    }

    pub async fn run(&self, control_tx: tokio::sync::mpsc::Sender<()>) {
        loop {
            self.fluctuate_conditions().await;

            // Notify the control system about the current state, including potential catastrophic events
            if let Err(e) = control_tx.send(()).await {
                eprintln!("Failed to send message to control system: {}", e);
                break;
            }

            sleep(Duration::from_secs(2)).await;
        }
    }
}
