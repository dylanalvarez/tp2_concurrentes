use crate::synonym::logger::file_logger;
use std::env;
mod synonym;

pub enum ResultBuilderMessage {
    NewSynonym { word: String, synonym: String },
    NoMoreSynonyms
}

pub enum SleeperMessage {
    RequestHasStarted
}

fn main() {
    file_logger::log("Starting...");
    let args: Vec<String> = env::args().collect();

    if let (
        Some(option),
        Some(max_concurrent_requests_string),
        Some(min_seconds_between_requests_string),
        Some(filename)
    ) = (
        args.get(1),
        args.get(2),
        args.get(3),
        args.get(4)
    ) {
        if let (
            Ok(max_concurrent_requests),
            Ok(min_seconds_between_requests)
        ) = (
            max_concurrent_requests_string.parse::<usize>(),
            min_seconds_between_requests_string.parse::<u64>()
        ) {
            match option.as_str() {
                "actors" => {
    synonym::actors::actor::start_actors(
                        filename,
                        max_concurrent_requests,
                        min_seconds_between_requests * 1000
    );
                },
                "without_actors" => {
                    synonym::without_actors::without_actors::without_actors(
                        filename,
                        max_concurrent_requests,
                        min_seconds_between_requests
                    );
                },
                _ => {
                    panic!("Invalid option. It must be 'actors' or 'without_actors'");
                }
            };
        }
    } else {
        panic!("Required args: option max_concurrent_requests min_seconds_between_requests filename");
    }
}
