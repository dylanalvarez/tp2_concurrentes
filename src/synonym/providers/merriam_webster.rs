// Mejorar esta bosta
pub fn raw_response_to_synonyms(body_to_scrap: String) -> Result<Vec<String>, String> {
    let mut tmp_synonyms_vec: Vec<&str> = body_to_scrap.rsplit("mw-list").collect();
    let results = tmp_synonyms_vec[tmp_synonyms_vec.len() - 2].to_string();
    tmp_synonyms_vec = results.split("/thesaurus/").collect();
    tmp_synonyms_vec = Vec::from(&tmp_synonyms_vec[1..]);

    let mut synonyms = Vec::new();
    for i in &tmp_synonyms_vec {
        let synonym: Vec<&str> = i.split("\"").collect();
        // TODO: user % ASCII decoding sin crate externo
        synonyms.push(synonym[0].replace("%20", " "));
    }

    Ok(synonyms)
}
