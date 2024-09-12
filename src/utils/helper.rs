use std::{
    sync::{mpsc, Arc},
    thread::{self, available_parallelism},
};

use colored::*; // Make sure to add `colored` crate to Cargo.toml

pub enum MessageType {
    Error,
    Success,
    Neutral,
}

impl MessageType {
    fn colorize(&self, message: &str) -> String {
        match self {
            MessageType::Error => message.red().to_string(),
            MessageType::Success => message.green().to_string(),
            MessageType::Neutral => message.to_string(),
        }
    }
}

pub fn prntln(message: &str, message_type: MessageType) {
    println!("{}", message_type.colorize(message))
}

pub fn run_in_threads_default<F, T, R>(items: Vec<T>, task: F) -> Vec<R>
where
    F: Fn(usize, &T) -> R + Send + Sync + 'static,
    T: Send + Sync + 'static,
    R: Send + 'static,
{
    let num_threads = available_parallelism().unwrap().into();
    run_in_threads(num_threads, items, task)
}

pub fn run_in_threads<F, T, R>(num_threads: usize, items: Vec<T>, task: F) -> Vec<R>
where
    F: Fn(usize, &T) -> R + Send + Sync + 'static,
    T: Send + Sync + 'static,
    R: Send + 'static,
{
    let task: Arc<F> = Arc::new(task);
    let items: Arc<Vec<T>> = Arc::new(items);
    let (tx_result, rx_result) = mpsc::channel();
    //If threads are not set by default the available threads are taken from std::thread
    let mut handles: Vec<thread::JoinHandle<()>> = vec![];

    for thread_id in 0..num_threads {
        let task: Arc<F> = Arc::clone(&task);
        let items: Arc<Vec<T>> = Arc::clone(&items);
        let tx_result: mpsc::Sender<R> = tx_result.clone();
        let handle: thread::JoinHandle<()> = thread::spawn(move || {
            for i in (thread_id..items.len()).step_by(num_threads) {
                let item: &T = &items[i];
                let result: R = task(thread_id, item);
                tx_result.send(result).expect("Failed to send result");
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    drop(tx_result); // Close the channel
    let mut results: Vec<R> = Vec::new();
    for result in rx_result {
        results.push(result);
    }

    results
}
