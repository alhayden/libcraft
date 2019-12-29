use std::process;
use std::io::Read;
use std::str::from_utf8;
use std::thread;
use std::sync::{Arc, Mutex};

fn subthread_stream_to_vec<R>(mut stream: R) -> Arc<Mutex<Vec<u8>>>
where R: Read + Send + 'static,
{
    let out = Arc::new(Mutex::new(Vec::new()));
    let vec = out.clone();
    thread::Builder::new()
        .name("subthread_stream_to_vec".into())
        .spawn(move ||  loop {
            let mut buffer = [0];
            match stream.read(&mut buffer) {
                Err(err) => {
                    print!("Failed to read from buffer: {}", err);
                    break;
                }
                Ok(output) => {
                    if output == 0 {
                        break
                    } else if output == 1 {
                        vec.lock().expect("!lock").push(buffer[0]);
                    } else {
                        print!("unexpected number of bytes {}", output);
                        break;
                    }
                }
            }
        }).expect("!thread");
    out
}

fn main() {
    let mut server_process = match process::Command::new("bash")
        .arg("-c")
        .arg("for i in 1 2 3 4 5 6 7 8 9; do echo hello; sleep 1; done")
        .stdout(process::Stdio::piped())
        .stdin(process::Stdio::piped())
        .spawn() {
        Err(why) => panic!("Failed to start server: {}", why),
        Ok(proc) => proc,
    };

    let buff = subthread_stream_to_vec(server_process.stdout.take().expect("!stdout"));
    loop {
        let mut b = buff.lock().expect("!locked");
        if b.len() > 0  {
            print!("{}", from_utf8(b.as_slice()).unwrap());
            b.clear();
        }/*
        if buff.lock().expect("!lock").len() > 0 {
            print!("{}", from_utf8(buff.lock().expect("!locked").as_slice()).unwrap());
            buff.lock().expect("!locked").clear();
        }*/
    }
    /*let mut stdout = server_process.stdout.unwrap();
    loop {
        let mut buf: [u8; 10] = [0; 10];
        let n = stdout.read(&mut buf).unwrap();//TODO change location?
        if n == 0 {
            break;
        }
        print!("{}", from_utf8(&buf[0..n]).unwrap());
    }*/

}