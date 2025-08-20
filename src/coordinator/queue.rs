use std::{
    collections::HashSet,
    io,
    sync::{Arc, Mutex, MutexGuard, mpsc},
    thread::{self, JoinHandle},
    time::Duration,
};

// use super::normalizer;

pub fn push_task_to_thread(
    recieved_task_queue: mpsc::Receiver<Vec<String>>,
    buf_size: &usize,
) -> Result<(mpsc::Receiver<Vec<String>>, Vec<JoinHandle<()>>), io::Error> {
    let k: usize = 10;
    let (tx, queue_rx) = mpsc::sync_channel::<Vec<String>>(k);
    // let (tx, queue_rx) = mpsc::sync_channel::<Vec<String>>(k);

    // let recieved_task_queue = Arc::clone(&cordinator_rx);

    let mut handles = vec![];

    let queue: Arc<Mutex<HashSet<Arc<str>>>> = Arc::new(Mutex::new(HashSet::new()));

    let tx = Arc::new(tx);

    for i in 0..*buf_size {
        // let pool_tx = Arc::clone(&tx);
        let pool_tx = tx.clone();
        let queue: Arc<Mutex<HashSet<Arc<str>>>> = Arc::clone(&queue);

        match recieved_task_queue.try_recv() {
            Ok(items) => {
                println!("DONE WITH URL SET CHECK");
                let handle = thread::spawn(move || {
                    let mut check_url: MutexGuard<'_, HashSet<Arc<str>>> = queue.lock().unwrap();

                    let urls_to_processed: Vec<String> = {
                        let mut urls: Vec<String> = Vec::new();

                        for url in items {
                            // let url_string = Arc::new(url.to_string());
                            let arc_url: Arc<str> = url.into();

                            let is_already_in_set = check_url.contains(&arc_url);

                            if !is_already_in_set {
                                urls.push(arc_url.to_string());
                                check_url.insert(arc_url.clone());
                            }
                        }

                        urls
                    };

                    for url in urls_to_processed.chunks(4) {
                        if let Err(e) = pool_tx.send(url.to_vec()) {
                            println!("Failed to send cos => {e}")
                        };
                    }
                    println!("TASKS SENT TO POOL TX");
                    // thread::sleep(Duration::from_millis(1000));
                    // urls_to_processed
                });

                handles.push(handle);
            }

            Err(e) => match e {
                mpsc::TryRecvError::Empty => {
                    println!("Channel buffer is empyt, we wait for 5ms for one more message");
                    thread::sleep(Duration::from_millis(500));
                }
                mpsc::TryRecvError::Disconnected => {
                    println!("send channel channel has been disconnected");
                    break;
                }
            },
        }
        println!("Current i => {i}");
        // break;
    }
    // returning the handles too so that the receiver starts receiving and this is due to the
    // buffer size on the sync_channel, the thread will be blocked cos reciever cant receive

    Ok((queue_rx, handles))
}
