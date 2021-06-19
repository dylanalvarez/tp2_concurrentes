use crate::synonym::providers::base::Provider::Thesaurus2;
use crate::synonym::providers::base;

pub fn synonyms(word: &str) -> Vec<String> {
    return base::synonyms(word, Thesaurus2);
}

pub fn raw_response_to_synonyms(_raw_response: String) -> Vec<String> {
    return Vec::new();
}
