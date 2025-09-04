use core::fmt;
use http::StatusCode;
use reqwest::{blocking::Client, header::ACCEPT};
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use std::{collections::HashMap, io, sync::mpsc};

#[derive(Debug)]
pub struct ResponseData {
    pub url: String,
    pub status_code: u16,
    pub html: Option<String>,
    pub headers: HashMap<String, String>,
}

impl ResponseData {
    fn new(
        url: String,
        status_code: u16,
        html: Option<String>,
        headers: HashMap<String, String>,
    ) -> Self {
        Self {
            url,
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

fn send_request(client: Client, url: &str) -> Result<ResponseData, io::Error> {
    let response = match client
        .get(url)
        .header(ACCEPT, "text/html")
        .header("User-Agent", "Crawler@olafrancis780@gmail.com")
        .send()
    {
        Ok(res) => res,
        // Err(e) => panic!("An error occured while sending request {e}"),
        Err(e) => {
            let status_code: u16 = match e.status() {
                Some(status) => status.as_u16(),
                None => StatusCode::BAD_GATEWAY.as_u16(),
            };
            eprintln!("STATUS CODE = $ {}", status_code);
            eprintln!("An error occured while sending request {e}");

            return Ok(ResponseData::new(
                url.to_string(),
                status_code,
                None,
                HashMap::new(),
            ));
        }
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

    let response_data = ResponseData::new(
        url.to_string(),
        status_code.as_u16(),
        Some(html.to_string()),
        headers,
    );

    // parser::parse_html::parse_html(&response_data)?;
    Ok(response_data)
}

pub fn process_queue_request(
    queue_rx: mpsc::Receiver<Vec<String>>,
    queue_handles: Vec<JoinHandle<()>>,
) -> Result<(Receiver<ResponseData>, Vec<JoinHandle<()>>), io::Error> {
    let max: usize = 4;
    let url_semaphores: Arc<Mutex<HashMap<String, usize>>> = Arc::new(Mutex::new(HashMap::new()));

    let mut handles: Vec<JoinHandle<()>> = Vec::new();

    let client: Arc<Mutex<Client>> = Arc::new(Mutex::new(Client::new()));

    let retry_queue: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let bound: usize = 50;
    let (result_tx, result_rx) = mpsc::sync_channel::<ResponseData>(bound);

    while let Ok(urls) = queue_rx.recv() {
        let client = Arc::clone(&client);
        let semaphore = Arc::clone(&url_semaphores);
        let retry_queue = Arc::clone(&retry_queue);

        let result_tx = result_tx.clone();
        let handle = thread::spawn(move || {
            for url in urls {
                let lock_acqure = try_acquire_semaphore_lock(&semaphore, url.as_str(), max);

                if !lock_acqure {
                    thread::sleep(Duration::from_millis(500));
                    retry_queue.lock().unwrap().push(url);
                    continue;
                }
                // Let us not hold on to a mutex lock during an IO session
                let http_client = {
                    let guard = client.lock().unwrap();
                    guard.clone()
                };

                let response = send_request(http_client, &url);

                match response {
                    Ok(res) => {
                        println!("Success - On to the request queue  \n \n ");
                        match result_tx.send(res) {
                            Ok(_) => {
                                println!("DONE SENDING TO RESULT POOL");
                                continue;
                            }
                            Err(e) => {
                                eprintln!("An error occured when sending to result pool {e}");
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("REQUEST FAILED FOR URL = {} with err {}", url, e);
                    }
                }

                release_semaphore_lock(&semaphore, url.as_str());
            }

            thread::sleep(Duration::from_millis(5000));
        });
        handles.push(handle);
    }

    for h in queue_handles {
        println!("Handling the queue threads");
        h.join().unwrap();
    }

    println!("Okay we are done");
    Ok((result_rx, handles))
}

fn try_acquire_semaphore_lock(
    semaphore_map: &Arc<Mutex<HashMap<String, usize>>>,
    url: &str,
    max: usize,
) -> bool {
    let mut map = semaphore_map.lock().unwrap();
    let current_count = map.entry(url.to_string()).or_insert(0);

    if *current_count >= max {
        false
    } else {
        *current_count += 1;
        true
    }
}

fn release_semaphore_lock(semaphore_map: &Arc<Mutex<HashMap<String, usize>>>, url: &str) {
    let mut map = semaphore_map.lock().unwrap();
    if let Some(c) = map.get_mut(url) {
        *c -= 1;
        if *c == 0 {
            map.remove(url);
        }
    }
}
