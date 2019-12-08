extern crate reqwest;

use std::error::Error;
use std::thread;
use std::time::SystemTime;

pub struct Config {
    pub num_threads: u32,
    pub url: String
}

impl Config {
    pub fn new(mut args: std::env::Args) -> Result<Config, &'static str> {
        args.next();
        let num_threads = match args.next() {
            Some(n) => {
                let arg: u32 = match n.trim().parse() {
                    Ok(num) => num,
                    Err(_) => return Err("Didn't get a valid number for number of threads"),
                };
                arg
            },
            None => return Err("Didn't get a number of threads")
        };
        let url = match args.next() {
            Some(url) => url,
            None => return Err("Didn't get a url")
        };
        Ok(Config { num_threads, url })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let mut all_threads = Vec::with_capacity(config.num_threads as usize);
    println!("Load testing '{}' with {} concurrent requests", config.url, config.num_threads);

    for _i in 0..config.num_threads {
        // not ideal, but need to read more about strings again to fix
        let new_url = config.url.clone();

        let thread = thread::spawn(move || {
            let now = SystemTime::now();
            match reqwest::get(&new_url) {
                Ok(_) => {
                    match now.elapsed() {
                        Ok(elapsed) => {
                            return Ok(elapsed.as_millis());
                        }
                        Err(e) => {
                            return Err(String::from(e.description()));
                        }
                    }
                }
                Err(e) => {
                    return Err(String::from(e.description()));
                }
            }
        });
        all_threads.push(thread);
    }
    let mut sum = 0;
    let mut idx = 0;
    let mut max_time = 0;
    let mut min_time = u128::max_value();
    for thread in all_threads {
        let t = thread.join().unwrap();
        match t {
            Ok(time) => {
                idx = idx + 1;
                sum = sum + time;
                if time < min_time {
                    min_time = time;
                }
                if time > max_time {
                    max_time = time;
                }
            }
            Err(e) => {
                println!("Not adding request stats due to error during execution: {}", e);
            }
        }
    }
    let avg = sum / idx;
    println!("Successfully completed {} requests", idx);
    println!("Avg response time: {}ms", avg);
    println!("Min response time: {}ms", min_time);
    println!("Max response time: {}ms", max_time);
    Ok(())
}
