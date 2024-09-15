extern crate anyhow;
extern crate chrono;
extern crate rand;
extern crate scheduled_thread_pool;

use anyhow::Result;
use rand::Rng;
use scheduled_thread_pool::ScheduledThreadPool;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub enum Task {
    LightsOn,
    LightsOff,
    CoffeeMorning,
    CoffeeNoon,
    CoffeeEvening,
    ThermostatCoolMorning,
    ThermostatModerateNoon,
    ThermostatWarmEvening,
    ThermostatCoolNight,
    SprinklersMorning,
    SecurityArmedNight,
    SecurityDisarmedMorning,
    AIGoodMorning,
    AIGoodNight,
    PlayLofiMusic,
    PlayAmbientMusic,
    IntrusionDetected,
    DeadlyWeatherWarning,
}

impl Task {
    pub fn schedule(&self) -> (f64, f64) {
        match *self {
            Task::LightsOn => (18.0 * 3600.0, 24.0 * 3600.0),
            Task::LightsOff => (6.0 * 3600.0, 24.0 * 3600.0),
            Task::CoffeeMorning => (7.0 * 3600.0, 24.0 * 3600.0),
            Task::CoffeeNoon => (12.0 * 3600.0, 24.0 * 3600.0),
            Task::CoffeeEvening => (18.0 * 3600.0, 24.0 * 3600.0),
            Task::ThermostatCoolMorning => (6.0 * 3600.0, 24.0 * 3600.0),
            Task::ThermostatModerateNoon => (12.0 * 3600.0, 24.0 * 3600.0),
            Task::ThermostatWarmEvening => (18.0 * 3600.0, 24.0 * 3600.0),
            Task::ThermostatCoolNight => (22.0 * 3600.0, 24.0 * 3600.0),
            Task::SprinklersMorning => (5.0 * 3600.0, 24.0 * 3600.0),
            Task::SecurityArmedNight => (22.0 * 3600.0, 24.0 * 3600.0),
            Task::SecurityDisarmedMorning => (6.0 * 3600.0, 24.0 * 3600.0),
            Task::AIGoodMorning => (7.0 * 3600.0, 24.0 * 3600.0),
            Task::AIGoodNight => (22.0 * 3600.0, 24.0 * 3600.0),
            Task::PlayLofiMusic => (9.0 * 3600.0, 24.0 * 3600.0),
            Task::PlayAmbientMusic => (21.0 * 3600.0, 24.0 * 3600.0),
            Task::IntrusionDetected => (0.0, 0.0), // Placeholder values; not used in periodic scheduling
            Task::DeadlyWeatherWarning => (0.0, 0.0), // Placeholder values; not used in periodic scheduling
        }
    }

    pub fn description(&self) -> &'static str {
        match *self {
            Task::LightsOn => "6 PM: Lights turned ON",
            Task::LightsOff => "6 AM: Lights turned OFF",
            Task::CoffeeMorning => "7 AM: Coffee machine making coffee",
            Task::CoffeeNoon => "12 PM: Coffee machine making coffee",
            Task::CoffeeEvening => "6 PM: Coffee machine making coffee",
            Task::ThermostatCoolMorning => "6 AM: Thermostat set to cool (65°F)",
            Task::ThermostatModerateNoon => "12 PM: Thermostat set to moderate (72°F)",
            Task::ThermostatWarmEvening => "6 PM: Thermostat set to warm (75°F)",
            Task::ThermostatCoolNight => "10 PM: Thermostat set to cool (68°F)",
            Task::SprinklersMorning => "5 AM: Sprinklers watering the garden",
            Task::SecurityArmedNight => "10 PM: Security system armed, night cameras ON",
            Task::SecurityDisarmedMorning => "6 AM: Security system disarmed, morning cameras ON",
            Task::AIGoodMorning => "7 AM: AI Assistant: Good morning!",
            Task::AIGoodNight => "10 PM: AI Assistant: Good night!",
            Task::PlayLofiMusic => "9 AM: Playing Lofi music for the day",
            Task::PlayAmbientMusic => "9 PM: Playing Ambient music for the night",
            Task::IntrusionDetected => "---------------------------------\nIntruder detected!\nInitiating security protocol.",
            Task::DeadlyWeatherWarning => {
                "---------------------------------\nDeadly weather warning issued!\nInitiating emergency protocol."
            }
        }
    }
}

pub struct LastEvent {
    pub time: Instant,
    pub cooldown_duration: Duration,
}

impl LastEvent {
    pub fn new(cooldown: u64) -> Self {
        LastEvent {
            time: Instant::now() - Duration::from_secs(cooldown), // Initialize to allow immediate event
            cooldown_duration: Duration::from_secs(cooldown),
        }
    }

    pub fn trigger(&mut self) -> bool {
        if self.time.elapsed() >= self.cooldown_duration {
            self.time = Instant::now();
            true
        } else {
            false
        }
    }
}

pub fn schedule_task(
    sched: &ScheduledThreadPool,
    task: Task,
    scale_factor: f64,
    last_task_time: Arc<Mutex<(Instant, bool)>>,
) -> Result<()> {
    let (start_time, interval) = task.schedule();
    let task_description = task.description();

    sched.execute_at_fixed_rate(
        Duration::from_secs_f64(start_time / scale_factor),
        Duration::from_secs_f64(interval / scale_factor),
        move || {
            let mut last_time = last_task_time.lock().unwrap();
            last_time.0 = Instant::now();
            last_time.1 = false; // Reset idle state
            println!("{}", task_description);
        },
    );

    Ok(())
}

pub fn run() -> Result<()> {
    let sched = ScheduledThreadPool::new(8);
    let scale_factor = 2880.0; // Scale factor to simulate a day in 30 seconds
    let last_task_time = Arc::new(Mutex::new((Instant::now(), false)));

    let tasks = [
        Task::LightsOn,
        Task::LightsOff,
        Task::CoffeeMorning,
        Task::CoffeeNoon,
        Task::CoffeeEvening,
        Task::ThermostatCoolMorning,
        Task::ThermostatModerateNoon,
        Task::ThermostatWarmEvening,
        Task::ThermostatCoolNight,
        Task::SprinklersMorning,
        Task::SecurityArmedNight,
        Task::SecurityDisarmedMorning,
        Task::AIGoodMorning,
        Task::AIGoodNight,
        Task::PlayLofiMusic,
        Task::PlayAmbientMusic,
    ];

    // Schedule periodic tasks
    for task in tasks.iter() {
        schedule_task(
            &sched,
            task.clone(),
            scale_factor,
            Arc::clone(&last_task_time),
        )?;
    }

    // Separate cooldown mechanisms for aperiodic and sporadic events
    let last_intrusion = Arc::new(Mutex::new(LastEvent::new(10))); // Cooldown of 10 seconds for intrusion
    let last_weather_warning = Arc::new(Mutex::new(LastEvent::new(20))); // Cooldown of 20 seconds for weather warning

    let sporadic_events = move || {
        let mut rng = rand::thread_rng();
        let mut intrusion_lock = last_intrusion.lock().unwrap();
        let mut weather_lock = last_weather_warning.lock().unwrap();

        if rng.gen_bool(0.2) && intrusion_lock.trigger() {
            let task = Task::IntrusionDetected;
            println!("{}", task.description());
            println!("---------------------------------");
            println!("Doors locked");
            println!("Cameras recording for authorities");
            println!("Message to intruder: You are trespassing!");
            println!("Porch floodlights turned ON");
            println!("---------------------------------");
        } else if rng.gen_bool(0.05) && weather_lock.trigger() {
            let task = Task::DeadlyWeatherWarning;
            println!("{}", task.description());
            println!("---------------------------------");
            println!("Basement bunker lights ON");
            println!("Switching electricity to emergency only");
            println!("Message to all family members: Head to the bunker!");
            println!("---------------------------------");
        }
    };

    sched.execute_at_fixed_rate(
        Duration::from_secs(1),
        Duration::from_secs(1),
        sporadic_events,
    );

    // Task to print "IDLE" if no task has run for the past second
    sched.execute_at_fixed_rate(Duration::from_secs(1), Duration::from_secs(1), move || {
        let mut last_time = last_task_time.lock().unwrap();
        if last_time.0.elapsed() >= Duration::from_secs(1) && !last_time.1 {
            println!("IDLE");
            last_time.1 = true; // Set idle state to true
        }
    });

    println!("Smart home simulation started.");

    let start_time = Instant::now();
    let duration = Duration::from_secs(30); // 1 day * 30 seconds/day = 30 seconds

    // Main loop for running the simulation for 1 day (30 seconds)
    while Instant::now().duration_since(start_time) < duration {
        std::thread::sleep(Duration::from_secs(1));
    }

    println!("Smart home simulation ended.");

    Ok(())
}
