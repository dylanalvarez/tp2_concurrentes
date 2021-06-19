pub mod providers {
    pub mod thesaurus;
    mod base;

    pub mod merriam_webster {
        const BASE_URL: &str = "https://www.merriam-webster.com/thesaurus/";

        pub fn synonyms(word: &str) -> Vec<String> {
            let synonyms = match query_http(word) {
                Err(e) => {
                    println!("[merriam-webster] Error on query_http: {:?}", e);

                    Vec::new()
                }
                Ok(response_body) => {
                    scrap_synonyms(response_body)
                }
            };

            synonyms
        }

        // TODO: agregar
        // 1. Semaforo para limitar requests en vuelo
        // 2. Condvar para restringir tiempo minimo entre requests consecutivos
        fn query_http(word: &str) -> Result<String, Box<dyn std::error::Error>> {
            let mut query_url: String = BASE_URL.to_owned();
            query_url.push_str(word);

            println!("[merriam-webster] Calling URL: {:?}", query_url);
            let response_body = reqwest::blocking::get(query_url)?.text()?;

            Ok(response_body)
        }

        // Mejorar esta bosta
        fn scrap_synonyms(body_to_scrap: String) -> Vec<String> {
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

            println!("[merriam-webster] Found synonyms: {:?}", synonyms);
            synonyms
        }
    }
}

