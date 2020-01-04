use std::{thread, io};
use std::os::unix::net::{UnixStream, UnixListener};
use std::io::Read;
use std::io::Write;
use std::sync::{Arc, Mutex};

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
        let mut line: Vec<u8> = Vec::new();
//        let mut packet: HashMap<str, str> = HashMap::new();
        loop {
            let mut buf: [u8; 1] = [0];
            let n = self.input_stream.lock().unwrap().read(&mut buf).unwrap();
            if n == 0 {
                break;
            }
            io::stdout().write(&buf).unwrap();
            line.push(buf[0]);
            if buf[0] == 0x0A{ // EOL
                if !line.contains(&0x3Au8) {
                    dbg!("Ignoring line");
                    continue; // this line didn't have a colon separator, ignore it
                }
                let pieces: Vec<&[u8]> = line.split(|s| *s == 0x3A).collect(); // ':' character
                let name = match String::from_utf8(Vec::from(pieces[0])) {
                    Ok(t) => t,
                    Err(_e) => break // this is bad, we're done TODO handle this better
                };
                let data = String::from_utf8(Vec::from(&line[pieces[0].len() + 1..line.len()-1])).unwrap();
                line = Vec::new();
                let mut os = self.output_stream.lock().unwrap();
                os.write(name.as_bytes()).unwrap();
                os.write(b" yoinks ").unwrap();
                os.write(data.as_bytes()).unwrap();
                os.write(b"\n").unwrap();
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
