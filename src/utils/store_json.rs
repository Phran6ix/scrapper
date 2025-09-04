use std::{
    fmt::format,
    fs,
    io::{self, Write},
    path::Path,
    time::{self, UNIX_EPOCH},
};

use serde::ser;
use serde_json::json;

pub fn store_data_to_file(
    url: &str,
    title: &str,
    child_links: &Vec<String>,
) -> Result<(), io::Error> {
    let now = time::SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let file_name = format!("result/file_{}.json", now);
    println!("{file_name}");
    let file_path = Path::new(&file_name);
    let file = fs::File::create_new(file_path).expect("File could not be created");

    let file_content = json!({
        "url" : url,
        "html":title   ,
        "child_links":child_links
    });

    serde_json::to_writer_pretty(&file, &file_content).unwrap();

    println!("file = {:?}", file);
    Ok(())
}
