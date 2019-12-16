extern crate reqwest;

use std::error::Error;
use std::fmt;
use std::ops::Div;
use std::sync::{Arc, Mutex};
use std::time;
use std::thread;
use std::time::SystemTime;

pub struct Config {
    pub num_threads: u32,
    pub url: String,
    pub max_concurrent_requests: Option<u32>,
}

impl Config {
    pub fn new(url: &str, num_threads: u32, max_concurrent_requests: Option<u32>) -> Config {
        Config {
            url: url.to_string(),
            num_threads,
            max_concurrent_requests,
        }
    }
}

struct Stats {
    avg_time: u128,
    median_time: u128,
    min_time: u128,
    max_time: u128,
    success_count: usize,
}

impl fmt::Display for Stats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        writeln!(f, "Successfully completed {} requests", self.success_count)?;
        writeln!(f, "Avg response time: {}ms", self.avg_time)?;
        writeln!(f, "Median response time: {}ms", self.median_time)?;
        writeln!(f, "Min response time: {}ms", self.min_time)?;
        writeln!(f, "Max response time: {}ms", self.max_time)
    }
}

fn median<T>(mut times: Vec<T>) -> T
where
    T: Div + Ord + Copy,
{
    times.sort();
    let mid_idx = times.len() / 2;
    times[mid_idx]
}

fn calc_stats(threads: Vec<thread::JoinHandle<Result<u128, String>>>) -> Stats {
    let mut sum = 0;
    let mut idx: usize = 0;
    let mut max_time = 0;
    let mut min_time = u128::max_value();
    let mut times = Vec::new();
    for thread in threads {
        let t = thread.join().unwrap();
        match t {
            Ok(time) => {
                idx += 1;
                sum += time;
                if time < min_time {
                    min_time = time;
                }
                if time > max_time {
                    max_time = time;
                }
                times.push(time);
            }
            Err(e) => {
                println!(
                    "Not adding request stats due to error during request: {}",
                    e
                );
            }
        }
    }
    let avg = sum / idx as u128;
    let median_time = median(times);
    Stats {
        avg_time: avg,
        median_time,
        min_time,
        max_time,
        success_count: idx,
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let mut all_threads = Vec::with_capacity(config.num_threads as usize);
    println!(
        "Load testing '{}' with {} concurrent requests",
        config.url, config.num_threads
    );

    // Atomic shared pointer to url
    let url = Arc::new(config.url);
    let client = Arc::new(reqwest::Client::new());
    let max_concurrent_requests = Arc::new(config.max_concurrent_requests);
    let current_request_count = Arc::new(Mutex::new(0u32));

    for _i in 0..config.num_threads {
        // Rebind `url` to a copy of the smart pointer so it can be moved into
        // the thread
        let url = Arc::clone(&url);
        let client = Arc::clone(&client);
        let max_concurrent_requests = Arc::clone(&max_concurrent_requests);
        let current_request_count = Arc::clone(&current_request_count);

        let thread = thread::spawn(move || {
            let ten_ms = time::Duration::from_millis(10);
            if let Some(max_concurrent_requests) = *max_concurrent_requests {
                while *current_request_count.lock().unwrap() >= max_concurrent_requests {
                    thread::sleep(ten_ms);
                }
                {
                    let mut current_request_count = current_request_count.lock().unwrap();
                    *current_request_count += 1;
                }
            }
            let now = SystemTime::now();
            let result = client.get(&*url).send();
            {
                let mut current_request_count = current_request_count.lock().unwrap();
                *current_request_count -= 1;
            }
            match result {
                Ok(_) => match now.elapsed() {
                    Ok(elapsed) => {
                        Ok(elapsed.as_millis())
                    }
                    Err(e) => {
                        Err(String::from(e.description()))
                    }
                },
                Err(e) => {
                    Err(String::from(e.description()))
                }
            }
        });
        all_threads.push(thread);
    }
    print!("{}", calc_stats(all_threads));
    Ok(())
}
