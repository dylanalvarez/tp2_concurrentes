use reqwest::header::USER_AGENT;
pub fn fetch_synonyms_raw_response(word: String, base_url: String) -> Result<String, String> {
    let mut query_url: String = base_url.to_owned();
    query_url.push_str(&word);

    let client = reqwest::blocking::Client::new();

    println!("Calling URL: {:?}", query_url);
    let result = match client
        .get(query_url.clone())
        .header(USER_AGENT, "Mozilla/5.0 (X11; Ubuntu; Linux x86_64; rv:89.0) Gecko/20100101 Firefox/89.0")
        .send() {
        Ok(response) => {
            match response.text() {
                Ok(text) => {Ok(text)}
                Err(error) => {Err(error.to_string())}
            }
        }
        Err(error) => {Err(error.to_string())}
    };
    println!("Finished calling URL: {:?}", query_url);
    result
}
