use anyhow::{Context, Result};
use chrono::{Duration, Local};
use crossbeam::channel;
use parking_lot::{Condvar, Mutex};
use rand::Rng;
use std::{
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc,
    },
    thread, time,
};

#[derive(Clone)]
#[allow(dead_code)]
struct Order {
    customer_id: usize,
    order_details: String,
    ticket_number: usize,
}

struct Customer {
    id: usize,
    order_sender: channel::Sender<Order>,
}

impl Customer {
    fn new(id: usize, order_sender: channel::Sender<Order>) -> Self {
        Customer { id, order_sender }
    }

    fn place_order(&self, ticket_counter: Arc<AtomicUsize>) -> Result<()> {
        let ticket_number = ticket_counter.fetch_add(1, Ordering::SeqCst);
        let order_details = format!("ORDER{}", self.id);
        let order = Order {
            customer_id: self.id,
            order_details: order_details.clone(),
            ticket_number,
        };
        println!("Customer {}: Orders coffee {}", self.id, order_details);
        self.order_sender
            .send(order)
            .context("Failed to send order to barista")?;
        Ok(())
    }
}

struct Barista {
    id: usize,
    order_queue: channel::Receiver<Order>,
    coffee_machine: Arc<Semaphore>,
    next_ticket: Arc<AtomicUsize>,
}

impl Barista {
    fn new(
        id: usize,
        order_queue: channel::Receiver<Order>,
        coffee_machine: Arc<Semaphore>,
        next_ticket: Arc<AtomicUsize>,
    ) -> Self {
        Barista {
            id,
            order_queue,
            coffee_machine,
            next_ticket,
        }
    }

    fn process_orders(&self) -> Result<()> {
        while let Ok(order) = self.order_queue.recv() {
            if !self.coffee_machine.try_acquire() {
                println!("Barista {}: Coffee Machine occupied, waiting.", self.id);
                self.coffee_machine.acquire();
            }

            println!("Barista {}: Brewing {}", self.id, order.order_details);
            thread::sleep(time::Duration::from_secs(2)); // Simulate brewing time
            println!("Barista {}: Brewed {}", self.id, order.order_details);

            // Wait until it's this order's turn to be served
            while self.next_ticket.load(Ordering::SeqCst) != order.ticket_number {
                thread::sleep(time::Duration::from_millis(10));
            }

            let prepared_order = format!("Prepared {}", order.order_details);
            println!("Barista {}: Serving {}", self.id, prepared_order);

            self.next_ticket.fetch_add(1, Ordering::SeqCst);
            self.coffee_machine.release();
        }
        Ok(())
    }
}

struct Semaphore {
    permits: Mutex<usize>,
    condvar: Condvar,
}

impl Semaphore {
    fn new(capacity: usize) -> Arc<Self> {
        Arc::new(Semaphore {
            permits: Mutex::new(capacity),
            condvar: Condvar::new(),
        })
    }

    fn try_acquire(&self) -> bool {
        let mut permits = self.permits.lock();
        if *permits > 0 {
            *permits -= 1;
            true
        } else {
            false
        }
    }

    fn acquire(&self) {
        let mut permits = self.permits.lock();
        while *permits == 0 {
            println!("Waiting for a free coffee machine slot...");
            self.condvar.wait(&mut permits);
        }
        *permits -= 1;
    }

    fn release(&self) {
        let mut permits = self.permits.lock();
        *permits += 1;
        self.condvar.notify_one();
    }
}

pub fn run() -> Result<()> {
    println!("Welcome to the Cafe! Your orders will be processed shortly.");
    let running = Arc::new(AtomicBool::new(true));
    let ticket_counter = Arc::new(AtomicUsize::new(1));
    let (order_sender, order_receiver) = channel::unbounded();
    let start_time = Local::now();
    let run_duration = Duration::seconds(10);

    // Start baristas
    let coffee_machine = Semaphore::new(3);
    let next_ticket = Arc::new(AtomicUsize::new(1));
    let baristas: Vec<_> = (1..=5)
        .map(|id| {
            let order_receiver = order_receiver.clone();
            let coffee_machine = coffee_machine.clone();
            let next_ticket = next_ticket.clone();
            thread::spawn(move || {
                let barista = Barista::new(id, order_receiver, coffee_machine, next_ticket);
                barista.process_orders().unwrap();
            })
        })
        .collect();

    // Generate customers
    let customers = {
        let order_sender = order_sender.clone();
        thread::spawn({
            let running = running.clone();
            move || {
                let mut id = 1;
                while Local::now() - start_time < run_duration {
                    let sender_clone = order_sender.clone();
                    let ticket_clone = ticket_counter.clone();
                    thread::spawn(move || {
                        let customer = Customer::new(id, sender_clone);
                        customer.place_order(ticket_clone).unwrap();
                    });
                    id += 1;
                    // Reduced delay between customers to 300-500 milliseconds for faster customer generation
                    thread::sleep(time::Duration::from_millis(
                        rand::thread_rng().gen_range(500..1000),
                    ));
                }
                running.store(false, Ordering::SeqCst);
            }
        })
    };

    // Wait for the running period to end
    customers.join().unwrap();
    println!("Cafe is closing, last orders!");
    drop(order_sender); // Close the channel to stop baristas after all orders are processed

    for barista in baristas {
        barista.join().unwrap();
    }

    println!("Cafe is now closed! Thanks for coming.");
    Ok(())
}
