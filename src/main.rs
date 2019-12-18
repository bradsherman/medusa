extern crate clap;
extern crate serde;
extern crate serde_json;

use clap::{App, Arg, value_t};
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

    let url = matches.value_of("url").unwrap();
    let threads = value_t!(matches.value_of("threads"), u32).unwrap_or_else(|e| e.exit());
    let max_concurrent_requests = value_t!(matches.value_of("max_concurrent_requests"), u32).ok();
    let config = config::Config::new(url, threads, max_concurrent_requests);

    if let Err(e) = medusa::run(config) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}
