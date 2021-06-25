use std::fs::OpenOptions;
use std::io::Write;

const LOG_FILE_NAME: &str = "/output.log";
const LINE_BREAK: &str = "\n";

pub fn log(content: &str) {
    let absolute_path_to_file = env!("CARGO_MANIFEST_DIR").to_string() + LOG_FILE_NAME;
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(absolute_path_to_file)
        .expect("Unable to open log file");
    let line = content.to_string() + LINE_BREAK;
    println!("{:?}", content);
    file.write_all(line.as_bytes()).expect("Unable to write data");
}
