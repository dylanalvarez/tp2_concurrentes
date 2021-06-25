pub fn raw_response_to_synonyms(body_to_scrap: String) -> Result<Vec<String>, String> {
    if body_to_scrap.contains("missing-query") {
        return Ok(Vec::new());
    }

    let synonyms: Vec<String> = body_to_scrap
        .split("mw-list")
        .collect::<Vec<&str>>()[1]
        .split("/thesaurus/")
        .collect::<Vec<&str>>()
        .iter()
        .filter(|x| x.contains("</a>"))
        .map(|x| x.replace("%20", " "))
        .map(|x| x.split("\"").collect::<Vec<&str>>()[0].to_string())
        .collect();

    Ok(synonyms)
}
