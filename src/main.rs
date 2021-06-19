mod synonym;

fn main() {
    let word = "car";

    println!("Synonyms of car: {:?}", synonym::providers::merriam_webster::synonyms(word));
    println!("Synonyms of car: {:?}", synonym::providers::thesaurus::synonyms(word));
    println!("Synonyms of car: {:?}", synonym::providers::thesaurus2::synonyms(word));
    // TODO: usar barrier(3) para joinear los resultados
}