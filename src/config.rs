use clap::value_t;
use serde::{Deserialize, Serialize};
use std::error::Error;
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
            If a config file is passed in it trumps all other arguments,
            otherwise we will try to parse the given arguments individually
        */
        match args.value_of("config") {
            Some(config_file) => Config::try_parse_file(&config_file),
            None => Config::try_parse_args(args),
        }
    }

    fn try_parse_file(config_file: &str) -> Result<Config, String> {
        let mut config_str = String::new();
        File::open(config_file)
            .map(|mut f| f.read_to_string(&mut config_str))
            .map_err(|e| format!("Unable to open file: {}", e.description()))
            .and_then(|_| {
                serde_json::from_str(&config_str)
                    .map_err(|e| format!("Invalid json in configuration file {}", e.description()))
            })
    }

    fn try_parse_args(args: clap::ArgMatches) -> Result<Config, String> {
        match (args.value_of("url"), args.value_of("threads")) {
            (Some(url), Some(num_threads)) => match num_threads.parse::<u32>() {
                Ok(num_threads) if num_threads > 0 => {
                    let max_concurrent_requests =
                        value_t!(args.value_of("max_concurrent_requests"), u32).ok();
                    Ok(Config::new(url, num_threads, max_concurrent_requests))
                }
                _ => Err("`threads` should be a positive integer".to_owned()),
            },
            (None, Some(_)) => Err("Missing required argument `url`".to_owned()),
            (Some(_), None) => Err("Missing required argument `threads`".to_owned()),
            (None, None) => Err("Missing required arguments `url` and `threads`".to_owned()),
        }
    }
}
