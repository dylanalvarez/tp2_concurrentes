use crate::synonym::providers::base::Provider::{MerriamWebster, Thesaurus, Thesaurus2};
use std::thread;

mod synonym;

fn main() {
    let word = "car";

    let thesaurus2_thread = thread::spawn(move || {
        println!("Synonyms of car: {:?}", synonym::providers::base::synonyms(word, Thesaurus2));
    });
    let webster_thread = thread::spawn(move || {
        println!("Synonyms of car: {:?}", synonym::providers::base::synonyms(word, MerriamWebster));
    });
    let thesaurus_thread = thread::spawn(move || {
        println!("Synonyms of car: {:?}", synonym::providers::base::synonyms(word, Thesaurus));
    });

    match thesaurus2_thread.join()
        .and_then(|_| webster_thread.join())
        .and_then(|_| thesaurus_thread.join()) {
        Ok(_) => {}
        Err(error) => { println!("Error joining: {:?}", error) }
    };
}
