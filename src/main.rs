use crate::synonym::providers::base::Provider::{MerriamWebster, Thesaurus, Thesaurus2};
use crate::synonym::helpers::file_parser;
use crate::ResultBuilderMessage::NoMoreSynonyms;
use std::thread;
use std::sync::{mpsc, Arc, Mutex, Condvar};
use std::collections::HashMap;
use std_semaphore::Semaphore;
use std::time::Duration;
use std::thread::sleep;

mod synonym;

pub enum ResultBuilderMessage {
    NewSynonym { word: String, synonym: String },
    NoMoreSynonyms
}

pub enum SleeperMessage {
    RequestHasStarted
}

fn main() {
    let max_concurrent_requests = 2;
    let min_seconds_between_requests = 5;

    let max_concurrent_requests_semaphore = Arc::new(Semaphore::new(max_concurrent_requests));

    let (
        result_builder_sender,
        result_builder_receiver
    ) = mpsc::channel::<ResultBuilderMessage>();
    let result_builder = thread::spawn(move || {
        let mut result = HashMap::<String, HashMap<String, usize>>::new();
        loop {
            match result_builder_receiver.recv() {
                Ok(message) => {
                    match message {
                        ResultBuilderMessage::NewSynonym { word, synonym } => {
                            let word_result =
                                result
                                .entry(word)
                                .or_insert(HashMap::new());
                            let synonym_word_count =
                                word_result
                                .entry(synonym)
                                .or_insert(0);
                            *synonym_word_count += 1;
                        }
                        ResultBuilderMessage::NoMoreSynonyms => {
                            return Ok(result);
                        }
                    }
                }
                Err(error) => {
                    return Err(error);
                }
            }
        }
    });

    let mut synonym_fetcher_threads = Vec::new();

    for provider in [Thesaurus, Thesaurus2, MerriamWebster].iter() {
        let time_between_requests_has_elapsed_condvar = Arc::new((Mutex::new(true), Condvar::new()));
        let (sleeper_sender, sleeper_receiver) = mpsc::channel::<()>();
        let condvar1 = time_between_requests_has_elapsed_condvar.clone();
        thread::spawn(move || {
            loop {
                match sleeper_receiver.recv() {
                    Ok(_) => {}
                    Err(_) => {}
                };
                sleep(Duration::from_secs(min_seconds_between_requests));
                let (allow_request, condvar) = &*condvar1.clone();
                match allow_request.lock() {
                    Ok(mut allow_request) => { *allow_request = true; }
                    Err(error) => { panic!("{}", error.to_string()) }
                };
                condvar.notify_all();
            }
        });
        match file_parser::read_lines("./words.txt") {
            Ok(lines) => {
                for word in lines {
                    match word {
                        Ok(word) => {
                            let result_builder_sender = mpsc::Sender::clone(&result_builder_sender);
                            let sleeper_sender = mpsc::Sender::clone(&sleeper_sender);
                            let semaphore = max_concurrent_requests_semaphore.clone();
                            let condvar2 = time_between_requests_has_elapsed_condvar.clone();
                            synonym_fetcher_threads.push(thread::spawn(move || {
                                synonym::providers::base::synonyms(word.as_str(), provider, result_builder_sender, &semaphore, &*condvar2, sleeper_sender);
                            }));
                        }
                        Err(error) => { panic!("{}", error) }
                    };
                }
            }
            Err(error) => {panic!("Couldn't read words.txt: {}", error)}
        };
    }

    for thread in synonym_fetcher_threads {
        match thread.join() {
            Ok(_) => {}
            Err(_) => {panic!("Couldn't join a synonym fetcher")}
        }
    }
    match result_builder_sender.send(NoMoreSynonyms) {
        Ok(_) => {}
        Err(_) => {panic!("Lost connection with result_builder")}
    }
    match result_builder.join() {
        Ok(result) => {
            match result {
                Ok(result) => { println!("{:?}", result) }
                Err(result_builder_error) => { println!("{:?}", result_builder_error) }
            }
        }
        Err(join_error) => { println!("{:?}", join_error) }
    };

    // Obtener como param
    let max_parallel_requests = 2;
    let min_wait_millis: u64 = 1000;
    let filename = "./words.txt";
    synonym::actors::actor::start_actors(filename.to_string(), max_parallel_requests, min_wait_millis);
}
