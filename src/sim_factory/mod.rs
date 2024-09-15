mod factory;
mod factory_ai;
mod shipment;
mod supplier;

use factory::Factory;
use factory_ai::FactoryAI;
use shipment::Shipment;
use supplier::Supplier;

pub fn run() {
    // Clear relevant queues before starting
    clear_queue("factory_status").unwrap();
    clear_queue("supplier_requests").unwrap();
    clear_queue("factory_ai").unwrap();
    clear_queue("shipment_requests").unwrap();

    // Start the Supplier in a separate thread
    std::thread::spawn(move || {
        let supplier = Supplier::new();
        supplier.start().unwrap();
    });

    // Start the Shipment in a separate thread
    std::thread::spawn(move || {
        let shipment = Shipment::new();
        shipment.start().unwrap();
    });

    // Start the FactoryAI in a separate thread
    std::thread::spawn(move || {
        let mut factory_ai = FactoryAI::new();
        factory_ai.start_simulation().unwrap();
    });

    // Start the Factory process
    let mut factory = Factory::new(5); // Set the number of cycles
    factory.run().unwrap();

    // Ensure that the threads are joined before the program exits
    std::thread::sleep(std::time::Duration::from_secs(2)); // Allow time for other threads to finish
    println!("Factory simulation terminated.");
}

// Function to clear RabbitMQ queues
fn clear_queue(queue_name: &str) -> amiquip::Result<()> {
    let mut connection = amiquip::Connection::insecure_open("amqp://guest:guest@localhost:5672")?;
    let channel = connection.open_channel(None)?;
    let queue = channel.queue_declare(queue_name, amiquip::QueueDeclareOptions::default())?;
    queue.purge()?;
    connection.close()?;
    Ok(())
}
