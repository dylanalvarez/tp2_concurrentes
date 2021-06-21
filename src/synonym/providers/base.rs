use reqwest::header::USER_AGENT;
use crate::synonym::providers::{thesaurus, merriam_webster, thesaurus2};
use crate::ResultBuilderMessage;
use std::sync::mpsc::{Sender};
use crate::ResultBuilderMessage::NewSynonym;
use std_semaphore::Semaphore;
use std::sync::{Arc, Condvar, Mutex, PoisonError, MutexGuard};

pub enum Provider {
    Thesaurus,
    Thesaurus2,
    MerriamWebster
}

pub fn synonyms(word: &str, provider: &Provider, result_builder_sender: Sender<ResultBuilderMessage>, max_concurrent_requests_semaphore: &Arc<Semaphore>, time_between_requests_has_elapsed_condvar: &(Mutex<bool>, Condvar), sleeper_sender: Sender<()>) {
    let base_url = match provider {
        Provider::Thesaurus => "https://thesaurus.yourdictionary.com/",
        Provider::MerriamWebster => "https://www.merriam-webster.com/thesaurus/",
        Provider::Thesaurus2 => "https://www.thesaurus.com/browse/",
    };
    match fetch_synonyms_raw_response(word, base_url, max_concurrent_requests_semaphore, time_between_requests_has_elapsed_condvar, sleeper_sender) {
        Err(error) => { panic!("{}", error); }
        Ok(response_body) => {
            match match provider {
                Provider::Thesaurus => thesaurus::raw_response_to_synonyms(response_body),
                Provider::MerriamWebster => merriam_webster::raw_response_to_synonyms(response_body),
                Provider::Thesaurus2 => thesaurus2::raw_response_to_synonyms(response_body)
            } {
                Ok(synonyms) => {
                    for synonym in synonyms {
                        match result_builder_sender.send(NewSynonym { word: word.to_string(), synonym }) {
                            Ok(_) => {}
                            Err(error) => { panic!("{}", error); }
                        };
                    };
                }
                Err(error) => { panic!("{}", error); }
            }
        }
    };
}

fn fetch_synonyms_raw_response(word: &str, base_url: &str, max_concurrent_requests_semaphore: &Arc<Semaphore>, time_between_requests_has_elapsed_condvar: &(Mutex<bool>, Condvar), sleeper_sender: Sender<()>) -> Result<String, String> {
    let mut query_url: String = base_url.to_owned();
    query_url.push_str(word);

    let client = reqwest::blocking::Client::new();

    max_concurrent_requests_semaphore.acquire();
    let (allow_request, condvar) = &*time_between_requests_has_elapsed_condvar.clone();
    match condvar.wait_while(allow_request.lock().unwrap(), |allow_request| {
        !*allow_request
    }) {
        Ok(_) => {}
        Err(error) => { return Err(error.to_string()) }
    };
    match allow_request.lock() {
        Ok(mut allow_request) => { *allow_request = false; }
        Err(error) => { return Err(error.to_string()) }
    };
    sleeper_sender.send(());
    println!("Calling URL: {:?}", query_url);
    let result = match client
        .get(query_url.clone())
        .header(USER_AGENT, "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:89.0) Gecko/20100101 Firefox/89.0")
        .send() {
        Ok(response) => {
            match response.text() {
                Ok(text) => {Ok(text)}
                Err(error) => {Err(error.to_string())}
            }
        }
        Err(error) => {Err(error.to_string())}
    };
    println!("Finished calling URL: {:?}", query_url);
    max_concurrent_requests_semaphore.release();
    result
}
