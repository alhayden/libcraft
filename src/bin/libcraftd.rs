use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::process::Child;
use std::{io, process};
use std::fs::{File, read_dir};
use std::path::Path;
use warp::Filter;
use serde::{Serialize, Deserialize};
use std::collections::hash_map::RandomState;
use libcraft::Error;
use regex::Regex;


#[tokio::main]
async fn main() {
    let not_found = warp::any().map(|| "404.");
    let server_list = warp::path!("server").map(list);
    let server_get = warp::path!("server" / String).map(get_server);
    let server_create = warp::path!("server").map(create);
    let server_edit = warp::path!("server" / String / "edit").map(|frederick| "edit");
    let server_delete = warp::path!("server" / String / "delete").map(|bill| "ded");
    let server_start = warp::path!("server" / String / "start").map(start);
    let server_stop = warp::path!("server" / String / "stop").map(stop);


    let get_methods = warp::get().and(server_list.or(server_get));
    let post_methods = warp::post().and(server_create.or(server_edit).or(server_delete).or(server_start).or(server_stop));
    let routes = get_methods.or(post_methods).or(not_found);

    let servers_path = "./servers";
    let server_map = load_servers(servers_path);

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}


fn list() -> &'static str {
    "lsit of servers"
}

fn get_server(id: String) -> &'static str { "t" }

fn create() -> &'static str {
    "u just made a server congrats"
}

fn start(id: String) -> &'static str {
    "go"
}

fn stop(id: String) -> &'static str {
    "stop"
}


#[derive(Serialize, Deserialize)]
struct Server {
    id: String,
    name: String,
    jarfile: String,
    pwd: String,
    #[serde(default)]
    jvm_args: String,
    #[serde(skip)]
    yaml_path: String,
    #[serde(skip)]
    process: Option<Child>,
}

impl Server {
    fn verify(&self) -> Result<(), Error> {
        // Verify server path and jarfile existence
        if !Path::new(self.pwd.as_str()).exists() {
            return Err(Error::VerificationError("Server directory does not exist."));
        }
        if !Path::new(self.pwd.as_str()).join(self.jarfile.as_str()).exists() {
            return Err(Error::VerificationError("Provided jarfile does not exist."));
        }
        // Double-check that yaml_path exists
        if !Path::new(self.yaml_path.as_str()).exists() {
            return Err(Error::VerificationError("Backing YAML file does not exist.")); // TODO maybe remove?
        }

        // Verify that id is only using allowed characters
        // let m: Vec<&str> = self.id.matches("^[0-9A-Za-z\\-]$").collect();
        let re = Regex::new("[0-9A-Za-z\\-_]+").unwrap();
        if !re.is_match(&self.id) {
            return Err(Error::VerificationError("ID uses invalid characters."));
        }

        Ok(())
    }

    fn start(&mut self) -> String {
        dbg!(&self.process);
        match &self.process {
            None => return String::from("Could not start process: server is already running"),
            Some(_c) => {}
        };
        let mut proc = process::Command::new("java");
        for arg in self.jvm_args.split(" ") {
            proc.arg(arg);
        }
        proc.arg("-jar");
        proc.arg(&self.jarfile);
        proc.current_dir(&self.pwd);
        match proc.spawn() {
            Ok(child_process) => self.process = Some(child_process),
            Err(_e) => { return String::from("Failed to start process: error in spawn"); }
        };
        let s: String = "aaa".parse().unwrap();
        println!("{}", s);
        dbg!(&self.process);
        String::from("Successfully started child process...")
    }
}

fn load_servers(path: &str) -> HashMap<String, Server> {
    println!("Attempting to load server yaml files...");
    let mut servers: HashMap<String, Server> = HashMap::new();
    for entry in read_dir(Path::new(path)).unwrap() { // TODO give a correct error message and crash gracefully when path is not found
        let entry = entry.unwrap();
        let path = entry.path();
        if !path.is_dir() && path.extension().unwrap_or("".as_ref()) == "yaml" {
            // read file
            println!("Attempting to load server from {} ...", path.to_str().unwrap());
            match load_server(path.to_str().unwrap()) {
                Ok(srv) => {
                    servers.insert(srv.id.clone(), srv);
                    ()
                }
                Err(e) => eprintln!("Error while loading server from {}: {}", path.to_str().unwrap(), e)
            }
        }
    }
    println!("Done!  {} servers loaded", servers.len());
    return servers;
}

fn load_server(filename: &str) -> Result<Server, Error> {
    let file = File::open(filename)?;
    let mut server: Server = serde_yaml::from_reader(file)?;
    server.yaml_path = filename.to_string();
    server.verify()?;
    return Ok(server);
}
