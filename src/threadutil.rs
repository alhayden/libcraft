use std::sync::{Arc, Mutex};
use std::io::Read;
use std::thread;

pub fn subthread_stream_to_vec<R>(mut stream: R) -> Arc<Mutex<Vec<u8>>>
    where R: Read + Send + 'static,
{
    let out = Arc::new(Mutex::new(Vec::new()));
    let vec = out.clone();
    thread::Builder::new()
        .name("subthread_stream_to_vec".into())
        .spawn(move || loop {
            let mut buffer = [0];
            match stream.read(&mut buffer) {
                Err(err) => {
                    print!("Failed to read from buffer: {}", err);
                    break;
                }
                Ok(output) => {
                    if output == 0 {
                        break;
                    } else if output == 1 {
                        vec.lock().expect("!lock").push(buffer[0]);
                    } else {
                        print!("unexpected number of bytes {}", output);
                        break;
                    }
                }
            }
        }).expect("!thread");
    return out;
}
