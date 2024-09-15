use amiquip::{
    Connection, ConsumerMessage, ConsumerOptions, Exchange, Publish, QueueDeclareOptions, Result,
};

// Struct for Supplier
pub struct Supplier;

impl Supplier {
    pub fn new() -> Self {
        Supplier {}
    }

    // Listening for Supply Requests + Timeout safety function
    pub fn start(&self) -> Result<()> {
        loop {
            match self.listen_for_restock_request() {
                Ok(_) => break,
                Err(e) => {
                    println!("Supplier: Error encountered: {}. Retrying...", e);
                    std::thread::sleep(std::time::Duration::from_secs(2));
                }
            }
        }
        Ok(())
    }

    fn listen_for_restock_request(&self) -> Result<()> {
        let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672")?;
        let channel = connection.open_channel(None)?;
        let queue = channel.queue_declare("supplier_requests", QueueDeclareOptions::default())?;
        let consumer = queue.consume(ConsumerOptions::default())?;
        println!("Supplier: Waiting for restock requests...");

        for message in consumer.receiver().iter() {
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    let body = String::from_utf8_lossy(&delivery.body);
                    if body == "RequestRestock" {
                        println!("Supplier: Received Notification [Resupply]");
                        println!("Supplier: Processing [Resupply]...");
                        self.send_supply_confirmation(&channel)?;
                        println!("Supplier: Completed [Resupply]");
                        break; // Exit the loop after processing the restock
                    }

                    consumer.ack(delivery)?;
                }
                _ => {
                    println!("Supplier: Consumer ended.");
                    break;
                }
            }
        }

        connection.close()
    }
    // Sucessful Supply Confirmation msg
    fn send_supply_confirmation(&self, channel: &amiquip::Channel) -> Result<()> {
        let exchange = Exchange::direct(channel);
        exchange.publish(Publish::new("SupplyConfirmed".as_bytes(), "factory_ai"))?;
        Ok(())
    }
}
