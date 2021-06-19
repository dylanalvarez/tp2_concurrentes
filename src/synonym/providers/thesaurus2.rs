use crate::synonym::providers::base::Provider::Thesaurus2;
use crate::synonym::providers::base;

pub fn synonyms(word: &str) -> Result<Vec<String>, String> {
    base::synonyms(word, Thesaurus2)
}

pub fn raw_response_to_synonyms(raw_response: String) -> Result<Vec<String>, String> {
    let synonyms: Vec<String> = raw_response
        .split(r#"<div data-testid="word-grid-container""#)
        .collect::<Vec<&str>>()[1]
        .split("</ul>")
        .collect::<Vec<&str>>()[0]
        .split("<!-- -->")
        .map(|x| x.rsplit(">").collect::<Vec<&str>>()[0].to_string())
        .collect();
    Ok(synonyms[0..synonyms.len() - 1].to_owned())
}
