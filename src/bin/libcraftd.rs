use std::{thread, io};
use std::os::unix::net::{UnixStream, UnixListener};
use std::io::Read;
use std::io::Write;
use std::sync::{Arc, Mutex};
use libcraft::net::get_packet;
use std::ops::DerefMut;

//static mut CLIENTS: Mutex<Vec<Arc<Client>>> = Mutex::new(Vec::new());

fn main() {
    open_listener();
}

struct Client {
    input_stream: Mutex<UnixStream>,
    output_stream: Mutex<UnixStream>,
    client_list: Arc<Mutex<Vec<Arc<Client>>>>,
}

impl Client {
    fn new(stream: UnixStream, client_list: Arc<Mutex<Vec<Arc<Client>>>>) -> Client {
        let is = stream.try_clone().unwrap();
        Client { input_stream: Mutex::new(stream), output_stream: Mutex::new(is), client_list }
    }

    fn handle_incoming(&self) {
        loop {
            let mut istream = self.input_stream.lock().unwrap();
            let packet = match get_packet(istream.deref_mut()) {
                Ok(p) => p,
                Err(e) => break
            };
            let mut ostream = self.output_stream.lock().unwrap();
            dbg!(&packet);
            for entry in packet {
                ostream.write(entry.0.as_bytes());
                ostream.write(b" yoinks ");
                ostream.write(entry.1.as_bytes());
                ostream.write(b"\n");
            }
        }
        dbg!("Client Disconnecting...");
        let mut clients = self.client_list.lock().unwrap();
        let pos = clients.iter().position(|arc| (arc.as_ref() as *const Client) == (self as *const Client)).unwrap();
        clients.remove(pos);
    }
}

fn open_listener() {
    // remove the sock if it exists so that we don't error when opening the server socket
    match std::fs::remove_file("libcraftd.sock") {
        Ok(_) => {}
        Err(_) => {}
    };
    let listener = UnixListener::bind("libcraftd.sock").expect("Couldn't open server socket!");

    let client_list: Arc<Mutex<Vec<Arc<Client>>>> = Arc::new(Mutex::new(Vec::new()));

    // accept connections and process them, spawning a new thread for each one
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                /* connection succeeded */
                let client = Arc::new(Client::new(stream, client_list.clone()));
                let client2 = client.clone();
                client_list.lock().unwrap().push(client);
                thread::spawn(move || client2.handle_incoming());
                println!("connect work");
            }
            Err(_) => {
                /* connection failed */
                println!("that was not very cash money of the unix socket api");
                break;
            }
        }
    }
}
