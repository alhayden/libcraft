use std::{thread, io, process};
use std::os::unix::net::{UnixStream, UnixListener};
use std::io::Read;
use std::io::Write;
use std::sync::{Arc, Mutex};
use libcraft::net::get_packet;
use std::ops::DerefMut;
use std::process::Child;
use yaml_rust::{YamlEmitter,YamlLoader};
use std::fs::File;

fn main() {
    open_listener();
}

struct Server {
    yaml_path: String,
    commit: bool,
    name: String,
    jarfile: String,
    pwd: String,
    jvm_args: String,
    child: Option<Child>
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
            if packet.contains_key("action") {
                match &packet.get("action").unwrap()[..] {
                    "start" => {
                        println!("recieved start.  not doing anything");
                        let mut b = Server::new(String::from("server.yaml"));
                    }
                    _ => {}
                }
            }
        }
        println!("Client Disconnecting...");
        let mut clients = self.client_list.lock().unwrap();
        let pos = clients.iter().position(|arc| (arc.as_ref() as *const Client) == (self as *const Client)).unwrap();
        clients.remove(pos);
    }
}

impl Server {
    fn new(yaml_path: String) -> Server {
        let mut file = File::open(&yaml_path).expect("Unable to open file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Unable to read file");

        let mut conf = YamlLoader::load_from_str(&mut contents);
        dbg!(conf).unwrap();
        Server {
            yaml_path: "".parse().unwrap(),
            commit: false,
            name: "server".parse().unwrap(),
            jarfile: "test.jar".parse().unwrap(),
            pwd: "server".parse().unwrap(),
            jvm_args: "".parse().unwrap(),
            child: None
        }
    }

    fn start(&mut self)  {
        let mut proc = process::Command::new("java");
        for arg in self.jvm_args.split(" ") {
            proc.arg(arg);
        }
        proc.arg("-jar");
        proc.arg(&self.jarfile);
        proc.current_dir(&self.pwd);
        let mut child = proc.spawn().unwrap();
        let s: String = "aaa".parse().unwrap();
        println!("{}", s);
//        self.child = Some();
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
    println!("TODO make sure that we murdered all of our children")
}

fn check_yaml_correct(yaml: HashMap<String, String>) -> boolean {

}