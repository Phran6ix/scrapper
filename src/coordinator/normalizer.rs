use std::{io, sync::mpsc::SyncSender, thread, time::Duration};

use url::Url;

pub fn clean_seed_inputs(
    inputs: Vec<&str>,
    // tx: Arc<Mutex<SyncSender<Vec<String>>>>,
    tx: SyncSender<Vec<String>>,
) -> Result<usize, io::Error> {
    let mut cleaned_urls: Vec<String> = Vec::with_capacity(inputs.len());

    for input in inputs {
        match Url::parse(input) {
            Ok(url) => {
                cleaned_urls.push(url.to_string());
            }
            Err(_) => continue,
        }
    }

    println!("===========");
    println!("Current len of cleaned url => {:?}", cleaned_urls.len());

    let mut counter: usize = 0;

    for chunk in cleaned_urls.chunks(10) {
        // tx.lock()
        tx.send(chunk.to_vec()).expect("Unable to add to queue");

        println!("...... message sent");

        thread::sleep(Duration::from_millis(100));
        counter += 1;
    }

    println!("We are done sending the chunks");
    println!("=========== \n\n\n");

    Ok(counter)
}
