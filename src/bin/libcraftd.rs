use std::{process, thread};
use std::str::from_utf8;
use std::sync::{Mutex, Arc};
use std::io::Read;
use libcraft::threadutil::subthread_stream_to_vec;


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
        if b.len() > 0 {
            print!("{}", from_utf8(b.as_slice()).unwrap());
            b.clear();
        }
    }
}