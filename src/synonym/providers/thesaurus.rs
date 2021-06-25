pub fn raw_response_to_synonyms(raw_response: String) -> Result<Vec<String>, String> {

    if raw_response.contains("No results found") {
        return Ok(vec![]);
    }

    let tmp_synonyms_vec = raw_response.split("<div class=\"single-synonym-wrapper\"").collect::<Vec<&str>>()[1];
    let tmp_synonyms_column = tmp_synonyms_vec.split("<").collect::<Vec<&str>>()[1..].join("<");
    let tmp_single_synonyms: Vec<String> = tmp_synonyms_column
        .split("<div class=\"single-synonym\"")
        .map(|line| {
            SingleSynonym {raw_html: line.to_string()}.get_synonym()
        }).collect();
    
    Ok(tmp_single_synonyms)
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

    #[test]
    fn test_empty_synonyms_result() {
        let path = Path::new("src/synonym/providers/tests/empty_search.html");
        match read_to_string(path) {
            Err(e) => {
                print!("{:?}", e);
                panic!("Cant open file");
            }

            Ok(raw_html) => {
                let result = raw_response_to_synonyms(raw_html);
                assert_eq!(result, Ok(vec![]))
            }
        };
    }
}
