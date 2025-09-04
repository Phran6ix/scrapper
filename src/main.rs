use std::{fs, sync::mpsc};
use std::time::Instant;

mod coordinator;
mod http_client;
mod parser;
mod utils;
mod workers;

use coordinator::normalizer;
use coordinator::queue;

fn main() {
    let now = Instant::now();

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

    let (queue_receiver, coord_handles) =
        queue::push_task_to_thread(reciever, &coord_buf_size).unwrap();

    let (result_rx, queue_handles) =
        http_client::connection::process_queue_request(queue_receiver, coord_handles).unwrap();

    println!("Hello, world!");

    // PROCESS THE RESULTS FROM THE CONNECTION NEXT
    workers::result_pool::process_response_data(result_rx, queue_handles).unwrap();

    let elapsed = now.elapsed();

    println!("TIME TAKEN TO COMPLETE  ==> {:.2?}",elapsed);
}
