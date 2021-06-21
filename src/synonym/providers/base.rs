use crate::synonym::providers::{thesaurus, merriam_webster, thesaurus2, http_requester};
use crate::ResultBuilderMessage;
use std::sync::mpsc::{Sender};
use crate::ResultBuilderMessage::NewSynonym;
use std_semaphore::Semaphore;
use std::sync::Arc;

pub enum Provider {
    Thesaurus,
    Thesaurus2,
    MerriamWebster
}

pub fn synonyms(word: &str, provider: Provider, sender: Sender<ResultBuilderMessage>, max_concurrent_requests_semaphore: &Arc<Semaphore>) {
    let base_url = match provider {
        Provider::Thesaurus => "https://thesaurus.yourdictionary.com/",
        Provider::MerriamWebster => "https://www.merriam-webster.com/thesaurus/",
        Provider::Thesaurus2 => "https://www.thesaurus.com/browse/",
    };
    match http_requester::fetch_synonyms_raw_response(word.to_string(), base_url.to_string()) {
        Err(error) => { panic!("{}", error); }
        Ok(response_body) => {
            match match provider {
                Provider::Thesaurus => thesaurus::raw_response_to_synonyms(response_body),
                Provider::MerriamWebster => merriam_webster::raw_response_to_synonyms(response_body),
                Provider::Thesaurus2 => thesaurus2::raw_response_to_synonyms(response_body)
            } {
                Ok(synonyms) => {
                    for synonym in synonyms {
                        match sender.send(NewSynonym { word: word.to_string(), synonym }) {
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
