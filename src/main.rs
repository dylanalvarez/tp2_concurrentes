mod synonym;

fn main() {
    let word = "car";

    println!("Synonyms of car: {:?}", synonym::providers::merriam_webster::synonyms(word));
    // TODO: consultar los demas providers y usar barrier(3) para joinear los resultados
}