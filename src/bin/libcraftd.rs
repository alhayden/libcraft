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
        let mut line: [u8; 50] = [0; 50];
        let mut i = 0;
        loop {
            let mut buf: [u8; 1] = [0];
            let n = stream.read(&mut buf).unwrap();
            print!("{}", n);
            if n == 0 {
                break;
            }
            print!("{}", from_utf8(&buf).unwrap());
            line[i] = buf[0];
            i += 1;
            if i >= line.len() - 1 || buf[0] == 0x0A { // EOL
                let actual_line  = from_utf8(&line[0..i]).unwrap();
                let pieces: Vec<&str> = actual_line.split(":").collect();
                let name = pieces[0];
                let data = &line[name.len()+1..i];
                i = 0;
                stream.write(name.as_bytes()).unwrap();
                stream.write(b" yoinks ").unwrap();
                stream.write(data).unwrap();
            }
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
