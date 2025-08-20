use core::fmt;
use reqwest::{blocking::Client, header::ACCEPT};
use std::clone;
use std::sync::mpsc::RecvError;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::{collections::HashMap, io, sync::mpsc};

use crate::coordinator::queue;
use crate::http_client;

struct ResponseData {
    status_code: u16,
    html: String,
    headers: HashMap<String, String>,
}

impl ResponseData {
    fn new(status_code: u16, html: String, headers: HashMap<String, String>) -> Self {
        Self {
            status_code,
            html,
            headers,
        }
    }
}

impl fmt::Display for ResponseData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "status code of => {}, with headers {:?}",
            self.status_code, self.headers
        )
    }
}

fn send_request(client: MutexGuard<'_, Client>, url: &str) -> Result<ResponseData, io::Error> {
    let response = match client
        .get(url)
        .header(ACCEPT, "text/html")
        .header("User-Agent", "Crawler@olafrancis780@gmail.com")
        .send()
    {
        Ok(res) => res,
        Err(e) => panic!("An error occured while sending request {e}"),
    };

    let status_code = response.status();
    let res_headers = response.headers();

    let mut headers = HashMap::new();

    for header in res_headers.iter() {
        if let Ok(value) = header.1.to_str() {
            headers.insert(header.0.to_string(), value.to_string());
        }
    }

    let html = response.text().expect("Error occured while parsing text");

    let response_data = ResponseData::new(status_code.as_u16(), html.to_string(), headers);

    Ok(response_data)
}

pub fn process_queue_request(
    queue_rx: mpsc::Receiver<Vec<String>>,
    queue_handles: Vec<JoinHandle<()>>,
) {
    let url_semaphores: Arc<Mutex<HashMap<&str, usize>>> = Arc::new(Mutex::new(HashMap::new()));

    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    let client: Arc<Mutex<Client>> = Arc::new(Mutex::new(Client::new()));
    while let Ok(urls) = queue_rx.recv() {
        // match Ok(task) {
        let client = Arc::clone(&client);

        let handle = thread::spawn(move || {
            for url in urls {
                let http_client = client.lock().unwrap();
                let response = send_request(http_client, &url).unwrap();

                println!("This is the response => {:#} \n \n \n ", response);
            }

            thread::sleep(Duration::from_millis(5000));

            // println!(
            //     "We are currently processing thread number and task is {:?}",
            //     urls
            // );
        });
        handles.push(handle);
        // }
        // Err(e) => match e {
        //     RecvError => println!("We are done sending to the http client"),
        // },
    }

    for h in queue_handles {
        println!("Handling the queue threads");
        h.join().unwrap();
    }

    for handle in handles {
        handle.join().unwrap();
    }
    println!("Okay we are done");
}
