extern crate reqwest;

use clap::value_t;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::ops::Div;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;
use std::time::SystemTime;

use std::fs::File;
use std::io::Read;

#[derive(Serialize, Deserialize)]
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

    pub fn try_parse(args: clap::ArgMatches) -> Result<Config, String> {
        /*
            If a config file is passed in it trumps all
            Otherwise we will try to parse the given arguments individually
        */
        match args.value_of("config") {
            Some(config_file) => {
                let mut config_str = String::new();
                if File::open(config_file)
                    .map(|mut f| f.read_to_string(&mut config_str))
                    .is_err()
                {
                    Err(format!("Invalid config file path {}", config_file))
                } else {
                    serde_json::from_str(&config_str).map_err(|e| {
                        format!("Invalid json in configuration file {}", e.description())
                    })
                }
            }
            None => {
                let url_opt = args.value_of("url");
                if url_opt.is_none() {
                    return Err(
                        "No config file found and url not passed in as an argument".to_string()
                    );
                }
                let url = url_opt.unwrap();

                let thread_opt = args.value_of("threads");
                if thread_opt.is_none() {
                    return Err(
                        "No config file found and threads not passed in as an argument".to_string(),
                    );
                }
                let parse_opt = thread_opt.unwrap().parse::<u32>();
                if parse_opt.is_err() {
                    return Err(format!(
                        "Invalid argument passed in for threads: {}",
                        thread_opt.unwrap()
                    ));
                }
                let threads = parse_opt.unwrap();

                let max_concurrent_requests =
                    value_t!(args.value_of("max_concurrent_requests"), u32).ok();

                Ok(Config::new(url, threads, max_concurrent_requests))
            }
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
                let mut current_request_count = current_request_count.lock().unwrap();
                *current_request_count += 1;
            }

            let now = SystemTime::now();
            let result = client.get(&*url).send();

            if (*max_concurrent_requests).is_some() {
                let mut current_request_count = current_request_count.lock().unwrap();
                *current_request_count -= 1;
            }
            match result {
                Ok(_) => match now.elapsed() {
                    Ok(elapsed) => Ok(elapsed.as_millis()),
                    Err(e) => Err(String::from(e.description())),
                },
                Err(e) => Err(String::from(e.description())),
            }
        });
        all_threads.push(thread);
    }
    print!("{}", calc_stats(all_threads));
    Ok(())
}
