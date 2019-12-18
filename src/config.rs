use clap::value_t;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::error::Error;

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
