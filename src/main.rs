use crate::synonym::providers::base::Provider::{MerriamWebster, Thesaurus, Thesaurus2};
use std::thread;
use std::sync::mpsc;
use std::collections::HashMap;
use crate::ResultBuilderMessage::NoMoreSynonyms;

mod synonym;

pub enum ResultBuilderMessage {
    NewSynonym { word: String, synonym: String },
    NoMoreSynonyms
}

fn main() {
    let word = "car";
    // let max_concurrent_requests = 4;
    // let min_time_between_requests = Duration::from_secs(1);

    let (sender, receiver) = mpsc::channel::<ResultBuilderMessage>();
    let result_builder = thread::spawn(move || {
        let mut result = HashMap::<String, HashMap<String, usize>>::new();
        loop {
            match receiver.recv() {
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

    let sender1 = mpsc::Sender::clone(&sender);
    let sender2 = mpsc::Sender::clone(&sender);
    let sender3 = mpsc::Sender::clone(&sender);
    let thesaurus2_thread = thread::spawn(move || {
        synonym::providers::base::synonyms(word, Thesaurus2, sender1);
    });
    let webster_thread = thread::spawn(move || {
        synonym::providers::base::synonyms(word, MerriamWebster, sender2);
    });
    let thesaurus_thread = thread::spawn(move || {
        synonym::providers::base::synonyms(word, Thesaurus, sender3);
    });

    match thesaurus2_thread.join()
        .and_then(|_| webster_thread.join())
        .and_then(|_| thesaurus_thread.join()) {
        Ok(_) => {}
        Err(error) => { println!("Error joining: {:?}", error) }
    };
    match sender.send(NoMoreSynonyms) {
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
}
