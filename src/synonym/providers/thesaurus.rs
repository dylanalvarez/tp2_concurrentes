const BASE_URL: &str = "https://thesaurus.yourdictionary.com/";
use reqwest::header::USER_AGENT;

pub fn synonyms(word: &str) -> Vec<String> {
    let synonyms = match http_get_thesaurus_synonyms(word) {
        Err(e) => {
            println!("[thesaurus] Error on query_http: {:?}", e);

            Vec::new()
        }
        Ok(response_body) => {
            scrap_synonyms(response_body)
        }
    };

    synonyms
}

fn http_get_thesaurus_synonyms(word: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut query_url: String = BASE_URL.to_owned();
    query_url.push_str(word);

    let client = reqwest::blocking::Client::new();
    
    println!("[thesaurus] Calling URL: {:?}", query_url);
    let response_body = client
        .get(query_url).header(USER_AGENT, "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:89.0) Gecko/20100101 Firefox/89.0")
        .send().unwrap();

    Ok(response_body.text()?)
}


fn scrap_synonyms(req_body: String) -> Vec<String> {
    let tmp_synonyms_vec = req_body.split("<div class=\"single-synonym-wrapper\"").collect::<Vec<&str>>()[1];
    
    let tmp_synonyms_column = tmp_synonyms_vec.split("<").collect::<Vec<&str>>()[1..].join("<");
    let tmp_single_synonyms: Vec<String> = tmp_synonyms_column
        .split("<div class=\"single-synonym\"")
        .map(|line| {
            SingleSynonym {raw_html: line.to_string()}.get_synonym()
        }).collect();
    
    tmp_single_synonyms
}


#[derive(Debug)]
struct SingleSynonym {
    raw_html: String
}

impl SingleSynonym {
    pub fn get_synonym(&mut self) -> String {
        let tmp_wrapper= self.raw_html.split("<div class=\"synonym-link-wrapper\"").collect::<Vec<&str>>()[1];
        let synonym_link_wrapper = tmp_wrapper.split("</div>").collect::<Vec<&str>>()[0];
        let tmp_synonym_link = synonym_link_wrapper.split("synonym-link").collect::<Vec<&str>>()[1];

        if tmp_synonym_link.contains("</a>") {
            tmp_synonym_link.split("</a>").collect::<Vec<&str>>()[0].split(">").collect::<Vec<&str>>()[1].to_string()
        } else {
            tmp_synonym_link.split("</span>").collect::<Vec<&str>>()[0].split(">").collect::<Vec<&str>>()[1].to_string()
        }
    }
}


#[cfg(test)]
mod tests {
    use std::{fs::{read_to_string}, path::Path};

    use super::*;
    #[test]
    fn test_single_synonym() {
        let path = Path::new("src/synonym/providers/tests/single_synonym.html");
        match read_to_string(path) {
            Err(e) => {
                print!("{:?}", e);
                panic!("Cant open file");
            }

            Ok(raw_html) => {
                let result = SingleSynonym {raw_html: raw_html}.get_synonym();
                assert_eq!(result, "motor-car")
            }
        };
    }
}