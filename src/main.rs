use crate::synonym::providers::base::Provider::{MerriamWebster, Thesaurus, Thesaurus2};
use std::thread;
use std::sync::{mpsc, Arc};
use std::collections::HashMap;
use crate::ResultBuilderMessage::NoMoreSynonyms;
use std_semaphore::Semaphore;

mod synonym;

pub enum ResultBuilderMessage {
    NewSynonym { word: String, synonym: String },
    NoMoreSynonyms
}

fn main() {
    let max_concurrent_requests = 2;
    let max_concurrent_requests_semaphore = Arc::new(Semaphore::new(max_concurrent_requests));

    // let min_time_between_requests = Duration::from_secs(1);

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

    for word in ["car", "cat"].iter() {
        let sender1 = mpsc::Sender::clone(&result_builder_sender);
        let sender2 = mpsc::Sender::clone(&result_builder_sender);
        let sender3 = mpsc::Sender::clone(&result_builder_sender);
        let semaphore1 = max_concurrent_requests_semaphore.clone();
        let semaphore2 = max_concurrent_requests_semaphore.clone();
        let semaphore3 = max_concurrent_requests_semaphore.clone();
        synonym_fetcher_threads.push(thread::spawn(move || {
            synonym::providers::base::synonyms(word, Thesaurus2, sender1, &semaphore1);
        }));
        synonym_fetcher_threads.push(thread::spawn(move || {
            synonym::providers::base::synonyms(word, MerriamWebster, sender2, &semaphore2);
        }));
        synonym_fetcher_threads.push(thread::spawn(move || {
            synonym::providers::base::synonyms(word, Thesaurus, sender3, &semaphore3);
        }));
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

    // If actores
    synonym::actors::actor::main();
}
