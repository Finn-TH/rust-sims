use amiquip::{
    Connection, ConsumerMessage, ConsumerOptions, Exchange, Publish, QueueDeclareOptions, Result,
};
use std::thread;
use std::time::Duration;

// Factory Struct
pub struct Factory {
    inventory: i32,
    beans_per_batch: i32,
    total_produced: i32,
    production_threshold: i32,
    current_cycle: i32,
    max_cycles: i32,
}

//Factory Variables

impl Factory {
    pub fn new(max_cycles: i32) -> Self {
        Factory {
            inventory: 300,
            beans_per_batch: 100,
            total_produced: 0,
            production_threshold: 200,
            current_cycle: 0,
            max_cycles,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        while self.current_cycle < self.max_cycles {
            if self.inventory > 0 {
                self.perform_production_cycle()?;
            } else {
                self.wait_for_signal("RestockComplete")?;
                self.inventory = 300; // Simulate restock replenishment
                println!("Factory: Inventory replenished to 300 grams. Production resuming...");
            }

            if self.total_produced >= self.production_threshold {
                self.request_shipment()?; // FactoryAI will report the shipment request
                self.total_produced = 0; // Reset after shipment
            }

            self.current_cycle += 1;

            if self.current_cycle >= self.max_cycles {
                println!("Factory: Maximum production cycles reached. Ending simulation.");
                self.send_end_signal()?;
                break;
            }
        }
        Ok(())
    }

    fn perform_production_cycle(&mut self) -> Result<()> {
        // Requesting grinding task
        println!("Factory: Requesting Authorization to Start [Grinding]");
        self.request_task("RequestGrinding")?;
        self.wait_for_signal("StartGrinding")?;
        self.notify_task_complete("GrindingComplete")?;

        // Requesting brewing task
        println!("Factory: Requesting Authorization to Start [Brewing]");
        self.request_task("RequestBrewing")?;
        self.wait_for_signal("StartBrewing")?;
        self.notify_task_complete("BrewingComplete")?;

        // Requesting packaging task
        println!("Factory: Requesting Authorization to Start [Packaging]");
        self.request_task("RequestPackaging")?;
        self.wait_for_signal("StartPackaging")?;
        self.notify_task_complete("PackagingComplete")?;

        self.inventory -= self.beans_per_batch;
        self.total_produced += self.beans_per_batch;

        println!("Inventory after production: {} grams", self.inventory);
        self.report_inventory()?;

        // Halt production if inventory is 0
        if self.inventory <= 0 {
            println!("Factory: Inventory is 0, halting production...");
        }

        thread::sleep(Duration::from_secs(1));
        Ok(())
    }

    //Requesting Authorization from FactoryAI
    fn request_task(&self, task: &str) -> Result<()> {
        let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672")?;
        let channel = connection.open_channel(None)?;
        let exchange = Exchange::direct(&channel);
        exchange.publish(Publish::new(task.as_bytes(), "factory_ai"))?;
        connection.close()?;
        Ok(())
    }

    //Report Max Production, request shipment to retail
    fn request_shipment(&self) -> Result<()> {
        let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672")?;
        let channel = connection.open_channel(None)?;
        let exchange = Exchange::direct(&channel);
        exchange.publish(Publish::new(
            format!("RequestShipment: {} grams", self.total_produced).as_bytes(),
            "factory_ai",
        ))?;
        connection.close()?;
        Ok(())
    }
    // Waiting for Authorization
    fn wait_for_signal(&self, expected_signal: &str) -> Result<()> {
        let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672")?;
        let channel = connection.open_channel(None)?;
        let queue = channel.queue_declare("factory_status", QueueDeclareOptions::default())?;
        let consumer = queue.consume(ConsumerOptions::default())?;

        for message in consumer.receiver().iter() {
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    let body = String::from_utf8_lossy(&delivery.body);
                    if body == expected_signal {
                        println!("\nFactory: Authorized [{}], Proceeding...", body);
                        consumer.ack(delivery)?;
                        break;
                    }
                }
                _ => break,
            }
        }

        connection.close()?;
        Ok(())
    }
    // Notifcation Per Production Task
    fn notify_task_complete(&self, task_complete: &str) -> Result<()> {
        let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672")?;
        let channel = connection.open_channel(None)?;
        let exchange = Exchange::direct(&channel);
        exchange.publish(Publish::new(task_complete.as_bytes(), "factory_ai"))?;
        connection.close()?;
        Ok(())
    }
    // Report Inventory Levels
    fn report_inventory(&self) -> Result<()> {
        let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672")?;
        let channel = connection.open_channel(None)?;
        let exchange = Exchange::direct(&channel);
        exchange.publish(Publish::new(
            format!("Inventory: {} grams", self.inventory).as_bytes(),
            "factory_ai",
        ))?;
        connection.close()?;
        Ok(())
    }
    // Send end Signal to Threads
    fn send_end_signal(&self) -> Result<()> {
        let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672")?;
        let channel = connection.open_channel(None)?;
        let exchange = Exchange::direct(&channel);
        exchange.publish(Publish::new("EndSimulation".as_bytes(), "factory_ai"))?;
        connection.close()?;
        Ok(())
    }
}
