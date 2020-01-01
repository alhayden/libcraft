use std::{thread, io};
use std::os::unix::net::{UnixStream, UnixListener};
use std::io::Read;
use std::io::Write;
use std::str::from_utf8;

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
            if n == 0 {
                break;
            }
            io::stdout().write(&buf).unwrap();
            line[i] = buf[0];
            i += 1;
            if i >= line.len() - 1 || buf[0] == 0x0A { // EOL
                if !line[0..i].contains(&0x3Au8) {
                    i = 0;
                    continue; // this line didn't have a colon separator, ignore it
                }
                let pieces: Vec<&[u8]> = line[0..i].split(|s| *s == 0x3A).collect(); // ':' character
                let name = pieces[0];
                let data = &line[name.len() + 1..i];
                i = 0;
                stream.write(name).unwrap();
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
