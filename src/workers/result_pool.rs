use std::{
    io,
    sync::mpsc::Receiver,
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::{
    http_client::connection::ResponseData, parser::parse_html::parse_html, utils::store_json,
};

struct Results {
    title: String,
    child_links: Vec<String>,
}

impl Results {
    fn new(title: &str, child_links: Vec<String>) -> Results {
        Results {
            title: title.to_string(),
            child_links,
        }
    }
}

pub fn process_response_data(
    rx: Receiver<ResponseData>,
    queue_handles: Vec<JoinHandle<()>>,
) -> Result<(), io::Error> {
    loop {
        match rx.recv() {
            Ok(res) => {
                let (url, html_text, child_links) = match parse_html(res) {
                    Ok((u, h, c)) => (u, h, c),
                    Err(e) => {
                        println!("Could not parse html  =>{e} ");
                        continue;
                    }
                };

                store_json::store_data_to_file(url.as_str(), html_text.as_str(), &child_links)
                    .unwrap();
                thread::sleep(Duration::from_millis(500));
            }

            Err(e) => {
                println!("SEEMS THE CHANNEL HAS CLOSED INIT, {e}");
                break;
            }
        }
    }
    //3. get our topic and number of child_links
    //4. and send them to the csv

    // while let Ok(resp) = rx.try_recv() {
    //     println!("current res {:?}", resp);
    // }

    //
    for queue_handle in queue_handles {
        println!("in result pool aidy");
        queue_handle.join().unwrap();
    }

    Ok(())
}

fn upload_result(results: Results) {
    println!("Ok")
}
