use std::thread::JoinHandle;
use std::{fs, sync::mpsc};

mod coordinator;
mod http_client;
mod parser;
mod workers;

use coordinator::normalizer;
use coordinator::queue;

fn main() {
    let input_text = fs::read_to_string("inputs.txt").expect("Error occured when reading file");

    let url_vec: Vec<&str> = input_text.lines().collect();
    let total_urls = url_vec.len();

    if total_urls < 1 {
        panic!("Cannot perform operation with empty data set");
    }
    let buf_size: usize = 200;

    if total_urls >= buf_size {
        panic!("Length of seeds is too large");
    }

    let (coordinator, reciever) = mpsc::sync_channel(buf_size);

    let coord_buf_size: usize = normalizer::clean_seed_inputs(url_vec, coordinator).unwrap();

    println!("BEFORE  buf_size = {}", coord_buf_size);

    let (queue_receiver, handles): (mpsc::Receiver<Vec<String>>, Vec<JoinHandle<()>>) =
        queue::push_task_to_thread(reciever, &coord_buf_size).unwrap();

    http_client::connection::process_queue_request(queue_receiver, handles);

    println!("Hello, world!");
}
