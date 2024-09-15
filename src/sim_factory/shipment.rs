use amiquip::{
    Channel, Connection, ConsumerMessage, ConsumerOptions, Exchange, Publish, QueueDeclareOptions,
    Result,
};

//Struct for Shipment Function
pub struct Shipment;

impl Shipment {
    pub fn new() -> Self {
        Shipment {}
    }
    // Start Listening for Shipping Request
    pub fn start(&self) -> Result<()> {
        let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672")?;
        let channel = connection.open_channel(None)?;
        let queue = channel.queue_declare("shipment_requests", QueueDeclareOptions::default())?;
        let consumer = queue.consume(ConsumerOptions::default())?;
        println!("Shipment: Waiting for shipment requests...");

        //Logic for Keyword
        for message in consumer.receiver().iter() {
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    let body = String::from_utf8_lossy(&delivery.body);
                    if body.starts_with("RequestShipment:") {
                        let amount: i32 = body["RequestShipment: ".len()..body.len() - 6]
                            .parse()
                            .unwrap();
                        println!("Shipment: Received Notification [Shipping]");
                        println!("Shipment: Processing [Shipping] of {} grams...", amount);
                        self.send_shipment_confirmation(&channel)?;
                        println!("Shipment: Completed [Shipping]");
                        break; // Exit after processing the shipment
                    }

                    consumer.ack(delivery)?;
                }
                _ => {
                    println!("Shipment: Consumer ended.");
                    break;
                }
            }
        }

        connection.close()
    }
    // Shipping Confirmation Msg
    fn send_shipment_confirmation(&self, channel: &Channel) -> Result<()> {
        let exchange = Exchange::direct(channel);
        exchange.publish(Publish::new("ShipmentConfirmed".as_bytes(), "factory_ai"))?;
        Ok(())
    }
}
