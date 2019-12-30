use std::{thread, time};
use std::os::unix::net::{UnixStream, UnixListener};
use std::thread::sleep;
use std::str::from_utf8;
use std::io::Read;
use std::io::Write;

fn main() {
    open_listener();
}

fn open_listener() {
    fn handle_client(mut stream: UnixStream) {
        loop {
            let mut buf: [u8; 1] = [0];
            let n = stream.read(&mut buf).unwrap();
            if n == 0 {
                break;
            }
            print!("{}", from_utf8(&buf).unwrap());
            stream.write(&buf);
        }
        println!("me done");
    }

    // remove the sock if it exists so that we don't error when opening the server socket
    match std::fs::remove_file("libcraftd.sock") {
        Ok(_) => {}
        Err(_) => {}
    };
    let listener = UnixListener::bind("libcraftd.sock").expect("Couldn't open server socket!");

    // accept connections and process them, spawning a new thread for each one
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                /* connection succeeded */
                println!("connect work");
                thread::spawn(|| handle_client(stream));
            }
            Err(_) => {
                /* connection failed */
                println!("did not compute");
                break;
            }
        }
    }
}
