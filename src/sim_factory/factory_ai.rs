use amiquip::{
    Channel, Connection, ConsumerMessage, ConsumerOptions, Exchange, Publish, QueueDeclareOptions,
    Result,
};
//FactoryAI Struct
pub struct FactoryAI {
    resupply_pending: bool, // Tracks if a resupply is already in progress
}

impl FactoryAI {
    pub fn new() -> Self {
        FactoryAI {
            resupply_pending: false, // Initialize with no resupply pending
        }
    }
    // Simulation start, overseeing production
    pub fn start_simulation(&mut self) -> Result<()> {
        let mut connection = Connection::insecure_open("amqp://guest:guest@localhost:5672")?;
        let channel = connection.open_channel(None)?;
        let queue = channel.queue_declare("factory_ai", QueueDeclareOptions::default())?;
        let consumer = queue.consume(ConsumerOptions::default())?;
        println!("FactoryAI: Waiting for task requests. Press Ctrl-C to exit.");

        for message in consumer.receiver().iter() {
            match message {
                ConsumerMessage::Delivery(delivery) => {
                    let body = String::from_utf8_lossy(&delivery.body);

                    match body.as_ref() {
                        "RequestGrinding" => {
                            println!("FactoryAI: Request [Grinding]");
                            self.send_message("StartGrinding", &channel)?;
                            println!("FactoryAI: Authorized [Grinding]");
                        }
                        "RequestBrewing" => {
                            println!("FactoryAI: Request [Brewing]");
                            self.send_message("StartBrewing", &channel)?;
                            println!("FactoryAI: Authorized [Brewing]");
                        }
                        "RequestPackaging" => {
                            println!("FactoryAI: Request [Packaging]");
                            self.send_message("StartPackaging", &channel)?;
                            println!("FactoryAI: Authorized [Packaging]");
                        }
                        "GrindingComplete" => {
                            println!("FactoryAI: Update [Grinding Complete]");
                        }
                        "BrewingComplete" => {
                            println!("FactoryAI: Update [Brewing Complete]");
                        }
                        "PackagingComplete" => {
                            println!("FactoryAI: Update [Packaging Complete]");
                        }
                        _ if body.starts_with("Inventory:") => {
                            let inventory_value: i32 = body["Inventory: ".len()..body.len() - 6]
                                .parse()
                                .unwrap_or(-1);
                            if inventory_value >= 0 {
                                if inventory_value == 0 && !self.resupply_pending {
                                    // Factory has already halted production before this point
                                    println!(
                                        "FactoryAI: Received notification: Inventory depleted."
                                    );
                                    self.request_restock(&channel)?; // Trigger resupply request
                                    self.resupply_pending = true; // Set pending flag to true
                                }
                                println!(
                                    "FactoryAI: Update [Current Inventory: {} grams]",
                                    inventory_value
                                );
                            } else {
                                println!("FactoryAI: Invalid inventory message format.");
                            }
                        }
                        _ if body.starts_with("RequestShipment:") => {
                            let shipment_amount: i32 = body
                                ["RequestShipment: ".len()..body.len() - 6]
                                .parse()
                                .unwrap_or(-1);
                            if shipment_amount > 0 {
                                println!(
                                    "FactoryAI: Request Production Quota Reached: [Shipping to Retail]"
                                );
                                self.request_shipment(&channel, shipment_amount)?;
                            } else {
                                println!("FactoryAI: Invalid shipment request format.");
                            }
                        }
                        "RequestRestock" => {
                            // Ignore since FactoryAI already handles this case
                            println!("FactoryAI: Ignoring duplicate restock request.");
                        }
                        "SupplyConfirmed" => {
                            println!("FactoryAI: Update [Resupply Complete]");
                            self.send_message("RestockComplete", &channel)?;
                            println!("FactoryAI: Inventory replenished.");
                            self.resupply_pending = false; // Reset after successful resupply
                        }
                        "ShipmentConfirmed" => {
                            println!("FactoryAI: Update [Shipping Complete]");
                        }
                        "EndSimulation" => {
                            println!("FactoryAI: Simulation ending, shutting down...");
                            break;
                        }
                        _ => println!("FactoryAI: Unknown task request [{}]", body),
                    }

                    consumer.ack(delivery)?;
                }
                _ => {
                    println!("FactoryAI: Consumer ended.");
                    break;
                }
            }
        }

        connection.close()
    }
    // Requesting restock by contacting Supplier
    fn request_restock(&mut self, channel: &Channel) -> Result<()> {
        let exchange = Exchange::direct(channel);
        exchange.publish(Publish::new(
            "RequestRestock".as_bytes(),
            "supplier_requests",
        ))?;
        println!("FactoryAI: Notified Supplier [Resupply]");
        self.resupply_pending = true; // Set pending flag to true
        Ok(())
    }
    //Requesting shipment to retail by contacting Shipper
    fn request_shipment(&self, channel: &Channel, amount: i32) -> Result<()> {
        let exchange = Exchange::direct(channel);
        exchange.publish(Publish::new(
            format!("RequestShipment: {} grams", amount).as_bytes(),
            "shipment_requests",
        ))?;
        println!("FactoryAI: Notified Retailer [Shipping]");
        Ok(())
    }
    // Publishing Factory Status Messages
    fn send_message(&self, message: &str, channel: &Channel) -> Result<()> {
        let exchange = Exchange::direct(channel);
        exchange.publish(Publish::new(message.as_bytes(), "factory_status"))?;
        Ok(())
    }
}
