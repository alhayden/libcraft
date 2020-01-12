use std::{thread, io, process};
use std::os::unix::net::{UnixStream, UnixListener};
use std::io::{Read, Write, Error, ErrorKind};
use std::sync::{Arc, Mutex};
use libcraft::net::get_packet;
use std::ops::DerefMut;
use std::process::Child;
use yaml_rust::{YamlEmitter,YamlLoader, Yaml};
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
                        let mut b = Server::new(String::from("server.yaml")).unwrap();
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
    fn new(yaml_path_arg: String) -> Result<Server, io::Error> {
        let mut file = match File::open(&yaml_path_arg) {
            Ok(f) => f,
            Err(e) => return Err(e),
        };
        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Ok(_) => {},
            Err(e) => return Err(e),
        };

        let confs = match YamlLoader::load_from_str(&mut contents){
            Ok(cfg) => cfg,
            Err(e) => return Err(Error::new(ErrorKind::Other, e)),
        };
        let mut conf: &Yaml = &confs[0];

        if ! check_yaml_correct(conf) {return Err(Error::new(ErrorKind::Other, "Malformed YAML"))} //somehow error out here

        dbg!(conf);
        Ok(
        Server {
            yaml_path: yaml_path_arg,
            commit: false,
            name: conf["name"].as_str().unwrap().to_string(),
            jarfile: conf["jarfile"].as_str().unwrap().to_string(),
            pwd: conf["pwd"].as_str().unwrap().to_string(),
            jvm_args: conf["jvm-args"].as_str().unwrap().to_string(),
            child: None
        })
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

fn check_yaml_correct(yaml: &Yaml) -> bool {
    for s in ["name", "pwd", "jarfile", "jvm-args", "server-args", "properties"].iter() {
        if yaml[*s].is_badvalue() { println!("{}", s);return false; }
    }
    return true;
}