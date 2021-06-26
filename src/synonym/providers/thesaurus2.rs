pub fn raw_response_to_synonyms(raw_response: String)
 -> Result<Vec<String>, String> {
    if raw_response.contains("0 results for") {
        return Ok(Vec::new());
    }

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

#[cfg(test)]
mod tests {
    use std::{fs::{read_to_string}, path::Path};

    use super::*;
    #[test]
    fn test_empty_synonyms_result_thesaurus2() {
        let path = Path::new("src/synonym/providers/tests/empty_search_thesaurus2.html");
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
