extern crate reqwest;

use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;
use std::time::SystemTime;

pub mod config;
pub mod stats;

pub fn run(config: config::Config) {
    let max_info = match config.max_concurrent_requests {
        Some(max_conc) => format!(" (maximum of {} concurrently)", max_conc),
        None => String::new(),
    };
    println!(
        "Load testing '{}' with {} concurrent requests{}",
        config.url, config.num_threads, max_info
    );

    // Atomic shared pointer to url
    let url = Arc::new(config.url);
    let client = Arc::new(reqwest::Client::new());
    let max_concurrent_requests = Arc::new(config.max_concurrent_requests);
    let current_request_count = Arc::new(Mutex::new(0u32));

    let all_threads: Vec<thread::JoinHandle<Result<u128, String>>> = (0..config.num_threads)
        .map(|_| {
            let url = Arc::clone(&url);
            let client = Arc::clone(&client);
            let max_concurrent_requests = Arc::clone(&max_concurrent_requests);
            let current_request_count = Arc::clone(&current_request_count);

            thread::spawn(move || {
                let ten_ms = time::Duration::from_millis(10);
                if let Some(max_concurrent_requests) = *max_concurrent_requests {
                    while *current_request_count.lock().unwrap() >= max_concurrent_requests {
                        thread::sleep(ten_ms);
                    }
                    let mut current_request_count = current_request_count.lock().unwrap();
                    *current_request_count += 1;
                }

                let now = SystemTime::now();
                let result = client.get(&*url).send();

                if (*max_concurrent_requests).is_some() {
                    let mut current_request_count = current_request_count.lock().unwrap();
                    *current_request_count -= 1;
                }
                match (result, now.elapsed()) {
                    (Ok(_), Ok(elapsed)) => Ok(elapsed.as_millis()),
                    (Err(e), _) => Err(String::from(e.description())),
                    (_, Err(e)) => Err(String::from(e.description())),
                }
            })
        })
        .collect();
    print!("{}", stats::calc_stats(all_threads));
}
