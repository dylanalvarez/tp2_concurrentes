pub fn raw_response_to_synonyms(body_to_scrap: String) -> Result<Vec<String>, String> {
    if body_to_scrap.contains("missing-query") {
        return Ok(Vec::new());
    }

    let synonyms: Vec<String> = body_to_scrap.split("mw-list").collect::<Vec<&str>>()[1]
        .split("/thesaurus/")
        .collect::<Vec<&str>>()
        .iter()
        .filter(|x| x.contains("</a>"))
        .map(|x| x.replace("%20", " "))
        .map(|x| x.split("\"").collect::<Vec<&str>>()[0].to_string())
        .collect();

    Ok(synonyms)
}

#[cfg(test)]
mod tests {
    use std::{fs::read_to_string, path::Path};

    use super::*;
    #[test]
    fn test_empty_synonyms_result_thesaurus2() {
        let path = Path::new("src/synonym/providers/tests/empty_search_merriam_webster.html");
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
