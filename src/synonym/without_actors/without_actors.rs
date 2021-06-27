use crate::synonym::helpers::file_parser;
use crate::synonym::logger::file_logger;
use crate::synonym::providers::base::Provider::{MerriamWebster, Thesaurus, Thesaurus2};
use crate::ResultBuilderMessage::NoMoreSynonyms;
use crate::{synonym, ResultBuilderMessage};
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use std_semaphore::Semaphore;

pub fn without_actors(
    filename: &String,
    max_concurrent_requests: usize,
    min_seconds_between_requests: u64,
) {
    let max_concurrent_requests_semaphore =
        Arc::new(Semaphore::new(max_concurrent_requests as isize));

    let (result_builder_sender, result_builder_receiver) = mpsc::channel::<ResultBuilderMessage>();

    let (log_sender, log_receiver) = mpsc::channel::<String>();
    let log_sender_result_builder = mpsc::Sender::clone(&log_sender);

    thread::spawn(move || loop {
        match log_receiver.recv() {
            Err(e) => {
                panic!("{}", e)
            }

            Ok(string_to_log) => {
                file_logger::log(&string_to_log);
            }
        }
    });

    let result_builder = thread::spawn(move || {
        let mut result = HashMap::<String, HashMap<String, usize>>::new();
        loop {
            match result_builder_receiver.recv() {
                Ok(message) => match message {
                    ResultBuilderMessage::NewSynonym { word, synonym } => {
                        let word_result = result.entry(word).or_insert(HashMap::new());
                        let synonym_word_count = word_result.entry(synonym).or_insert(0);
                        *synonym_word_count += 1;
                    }
                    ResultBuilderMessage::NoMoreSynonyms => {
                        handle_log(
                            &log_sender_result_builder,
                            "[Result Builder] Recieved NoMoreSynonyms message. Returning result..."
                                .to_string(),
                        );
                        return Ok(result);
                    }
                },
                Err(error) => {
                    handle_log(
                        &log_sender_result_builder,
                        format!("[Result Builder] Error receiving from channel: {:?}", error),
                    );
                    return Err(error);
                }
            }
        }
    });

    let mut synonym_fetcher_threads = Vec::new();

    let log_sender_provider = mpsc::Sender::clone(&log_sender);
    for provider in [Thesaurus, Thesaurus2, MerriamWebster].iter() {
        handle_log(
            &log_sender_provider,
            format!("[Provider {:?}] Starting...", provider),
        );
        let time_between_requests_has_elapsed_condvar =
            Arc::new((Mutex::new(true), Condvar::new()));
        let (sleeper_sender, sleeper_receiver) = mpsc::channel::<()>();
        let condvar1 = time_between_requests_has_elapsed_condvar.clone();
        thread::spawn(move || loop {
            match sleeper_receiver.recv() {
                Ok(_) => {}
                Err(_) => {}
            };
            sleep(Duration::from_secs(min_seconds_between_requests));
            let (allow_request, condvar) = &*condvar1.clone();
            match allow_request.lock() {
                Ok(mut allow_request) => {
                    *allow_request = true;
                }
                Err(error) => {
                    panic!("{}", error.to_string())
                }
            };
            condvar.notify_all();
        });
        match file_parser::read_lines(filename) {
            Ok(lines) => {
                for word in lines {
                    match word {
                        Ok(word) => {
                            handle_log(
                                &log_sender_provider,
                                format!(
                                    "[Provider {:?}] Fetching synonyms for word: {:?}",
                                    provider, word
                                ),
                            );
                            let result_builder_sender = mpsc::Sender::clone(&result_builder_sender);
                            let sleeper_sender = mpsc::Sender::clone(&sleeper_sender);
                            let semaphore = max_concurrent_requests_semaphore.clone();
                            let condvar2 = time_between_requests_has_elapsed_condvar.clone();
                            synonym_fetcher_threads.push(thread::spawn(move || {
                                synonym::providers::base::synonyms(
                                    word.as_str(),
                                    provider,
                                    result_builder_sender,
                                    &semaphore,
                                    &*condvar2,
                                    sleeper_sender,
                                );
                            }));
                        }
                        Err(error) => {
                            handle_log(
                                &log_sender_provider,
                                format!(
                                    "[Provider {:?}] Failed to read lines from file: {:?}",
                                    provider, error
                                ),
                            );
                            panic!("{}", error)
                        }
                    };
                }
            }
            Err(error) => {
                handle_log(
                    &log_sender_provider,
                    format!(
                        "[Provider {:?}] Couldn't read words.txt: {}",
                        provider, error
                    ),
                );
                panic!("Couldn't read words.txt: {}", error)
            }
        };
    }

    handle_log(
        &log_sender,
        format!("[Main] Starting to join synonym fetchers"),
    );
    for thread in synonym_fetcher_threads {
        match thread.join() {
            Ok(_) => {}
            Err(_) => {
                panic!("Couldn't join a synonym fetcher")
            }
        }
    }

    match result_builder_sender.send(NoMoreSynonyms) {
        Ok(_) => {
            handle_log(
                &log_sender,
                format!("[Main] No more fetchers working. NoMoreSynonyms sent"),
            );
        }
        Err(_) => {
            panic!("Lost connection with result_builder")
        }
    }

    match result_builder.join() {
        Ok(result) => match result {
            Ok(result) => {
                handle_log(&log_sender, format!("[Main] Final result {:?}", result));
                println!("{:?}", result)
            }
            Err(result_builder_error) => {
                println!("{:?}", result_builder_error)
            }
        },
        Err(join_error) => {
            println!("{:?}", join_error)
        }
    };
}

fn handle_log(log_sender: &Sender<String>, message: String) {
    match log_sender.send(message) {
        Ok(_) => {}
        Err(err) => {
            println!("Failed to log: {:?}", err)
        }
    };
}
