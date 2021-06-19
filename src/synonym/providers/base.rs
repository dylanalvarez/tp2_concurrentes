use reqwest::header::USER_AGENT;
use crate::synonym::providers::{thesaurus, merriam_webster};

pub enum Provider {
    Thesaurus,
    MerriamWebster
}

pub fn synonyms(word: &str, provider: Provider) -> Vec<String> {
    let base_url = match provider {
        Provider::Thesaurus => "https://thesaurus.yourdictionary.com/",
        Provider::MerriamWebster => "https://www.merriam-webster.com/thesaurus/"
    };
    let synonyms = match fetch_synonyms_raw_response(word, base_url) {
        Err(e) => {
            println!("[thesaurus] Error on query_http: {:?}", e);

            Vec::new()
        }
        Ok(response_body) => {
            match provider {
                Provider::Thesaurus => thesaurus::raw_response_to_synonyms(response_body),
                Provider::MerriamWebster => merriam_webster::raw_response_to_synonyms(response_body)
            }
        }
    };

    synonyms
}

fn fetch_synonyms_raw_response(word: &str, base_url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut query_url: String = base_url.to_owned();
    query_url.push_str(word);

    let client = reqwest::blocking::Client::new();

    println!("[thesaurus] Calling URL: {:?}", query_url);
    let response_body = client
        .get(query_url).header(USER_AGENT, "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:89.0) Gecko/20100101 Firefox/89.0")
        .send().unwrap();

    Ok(response_body.text()?)
}
