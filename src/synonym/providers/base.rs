use reqwest::header::USER_AGENT;
use crate::synonym::providers::{thesaurus, merriam_webster, thesaurus2};

pub enum Provider {
    Thesaurus,
    Thesaurus2,
    MerriamWebster
}

pub fn synonyms(word: &str, provider: Provider) -> Result<Vec<String>, String> {
    let base_url = match provider {
        Provider::Thesaurus => "https://thesaurus.yourdictionary.com/",
        Provider::MerriamWebster => "https://www.merriam-webster.com/thesaurus/",
        Provider::Thesaurus2 => "https://www.thesaurus.com/browse/",
    };
    match fetch_synonyms_raw_response(word, base_url) {
        Err(error) => { Err(error) }
        Ok(response_body) => {
            match provider {
                Provider::Thesaurus => thesaurus::raw_response_to_synonyms(response_body),
                Provider::MerriamWebster => merriam_webster::raw_response_to_synonyms(response_body),
                Provider::Thesaurus2 => thesaurus2::raw_response_to_synonyms(response_body)
            }
        }
    }
}

fn fetch_synonyms_raw_response(word: &str, base_url: &str) -> Result<String, String> {
    let mut query_url: String = base_url.to_owned();
    query_url.push_str(word);

    let client = reqwest::blocking::Client::new();

    println!("Calling URL: {:?}", query_url);
    match client
        .get(query_url)
        .header(USER_AGENT, "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:89.0) Gecko/20100101 Firefox/89.0")
        .send() {
        Ok(response) => {
            match response.text() {
                Ok(text) => {Ok(text)}
                Err(error) => {Err(error.to_string())}
            }
        }
        Err(error) => {Err(error.to_string())}
    }
}
