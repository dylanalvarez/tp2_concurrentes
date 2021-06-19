use crate::synonym::providers::base::Provider::Thesaurus;
use crate::synonym::providers::base;

pub fn synonyms(word: &str) -> Vec<String> {
    return base::synonyms(word, Thesaurus);
}

pub fn raw_response_to_synonyms(raw_response: String) -> Vec<String> {
    let tmp_synonyms_vec = raw_response.split("<div class=\"single-synonym-wrapper\"").collect::<Vec<&str>>()[1];
    
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
