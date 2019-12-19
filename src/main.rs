extern crate clap;
extern crate serde;
extern crate serde_json;

use clap::{App, Arg};
use std::process;

use medusa::config;

fn main() {
    let matches = App::new("Medusa")
        .version("0.1.0")
        .author("Brad Sherman & Carter Green")
        .about("API load testing tool")
        .arg(
            Arg::with_name("config")
                .short("-c")
                .long("config")
                .value_name("FILE")
                .help("JSON file containing the configuration to run")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("url")
                .short("-u")
                .long("url")
                .value_name("URL")
                .help("Sets the url to be tested")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("threads")
                .short("-t")
                .long("threads")
                .value_name("THREADS")
                .help("Sets the number of threads to be used")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("max_concurrent_requests")
                .short("-m")
                .long("max-concurrent-reqs")
                .value_name("REQUEST COUNT")
                .help("Sets a limit for the number of concurrent requests")
                .takes_value(true),
        )
        .get_matches();

    let config = config::Config::try_parse(matches).unwrap_or_else(|e| {
        eprintln!("Error parsing config: {}", e);
        process::exit(1)
    });

    medusa::run(config);
}
