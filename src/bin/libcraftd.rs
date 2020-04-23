#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;

use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::process::Child;
use std::{io, process};
use std::fs::{File, read_dir};
use yaml_rust::{YamlLoader, Yaml};
use std::io::{Error, ErrorKind, Read};
use std::path::Path;
use std::ffi::OsStr;

#[get("/")]
fn index() -> &'static str {
    "Working minecraft server manager - Implementation is left as an exercise to the reader."
}

fn main() {
    let server_map: Arc<Mutex<HashMap<String, Arc<Server>>>> = Arc::new(Mutex::new(HashMap::new()));
    load(server_map.clone());
    rocket::ignite().mount("/", routes![index, list, create]).launch();
}

#[get("/server")]
fn list() -> &'static str {
    "lsit of servers"
}

#[post("/server/<id>")]
fn create(id: i32) -> &'static str {
    "u just made a server congrats"
}

fn start(args: Vec<String>) {
}

fn stop(args: Vec<String>) {
}

fn force_stop(args: Vec<String>) {
}

fn restart(args: Vec<String>) {
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
        let conf: &Yaml = &confs[0];

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

    fn start(&mut self)  -> String {
        dbg!(&self.child);
        match &self.child {
            None => return String::from("Could not start process: server is already running"),
            Some(c) => {},
        };
        let mut proc = process::Command::new("java");
        for arg in self.jvm_args.split(" ") {
            proc.arg(arg);
        }
        proc.arg("-jar");
        proc.arg(&self.jarfile);
        proc.current_dir(&self.pwd);
        match proc.spawn() {
            Ok(childProc) => self.child = Some(childProc),
            Err(e) => {return String::from("Failed to start process: error in spawn");},
        };
        let s: String = "aaa".parse().unwrap();
        println!("{}", s);
        dbg!(&self.child);
        String::from("Successfully started child process...")
    }
}

fn load(server_map: Arc<Mutex<HashMap<String, Arc<Server>>>>) {
    println!("Attempting to load server yaml files...");
    for entry in read_dir(Path::new(".")).unwrap() {//TODO THIS IS NOT THE FINAL PATH
            let entry = entry.unwrap();
            let path = entry.path();
            if !path.is_dir() && path.extension().unwrap_or(OsStr::new("urbad")) == OsStr::new("yaml") {
                load_server(server_map.clone(), String::from(path.to_str().unwrap()));
            }
        }
    println!("Done!  {} servers loaded", server_map.lock().unwrap().len());
}

fn load_server(server_map: Arc<Mutex<HashMap<String, Arc<Server>>>>, filename: String) {
    println!("Attempting to load server from {} ...", filename);
    match Server::new(filename) {
        Ok(srv) => {
            server_map.lock().unwrap().insert(String::from(&(srv.name)), Arc::new(srv));
            println!("Successfully loaded server!");
        },
        Err(e) => {println!("Failed to load server: {}",e)}
    }
}

fn check_yaml_correct(yaml: &Yaml) -> bool {
    for s in ["name", "pwd", "jarfile", "jvm-args", "server-args", "properties"].iter() {
        if yaml[*s].is_badvalue() { return false; }
    }
    return true;
}