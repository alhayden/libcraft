use std::sync::{Mutex, MutexGuard};
use std::collections::HashMap;
use std::os::unix::io::{FromRawFd, AsRawFd};
use std::process::{Child, Stdio};
use std::process;
use std::fs::{File, read_dir};
use std::path::Path;
use warp::Filter;
use serde::{Serialize, Deserialize};
use libcraft::Error;
use regex::Regex;
use once_cell::sync::OnceCell;
use std::io::Read;
use std::os::raw::c_int;
use std::sync::Arc;
use futures::{StreamExt, FutureExt};

static GLOBAL_SERVER_MAP: OnceCell<Mutex<HashMap<String, Server>>> = OnceCell::new();

/// Get a locked, mutable reference to the global Server HashMap
fn get_server_map() -> MutexGuard<'static, HashMap<String, Server>> {
    // Unsure if this lifetime should be static. Am I saying that the HashMap is static, or that the MutexGuard is static?
    // Survey says that it's specifying a static HashMap, but I'm still unsure.
    GLOBAL_SERVER_MAP.get().expect("Server map unintalized.").lock().expect("Could not lock server list.")
}

#[tokio::main]
async fn main() {
    let not_found = warp::any().map(|| warp::http::Response::builder().status(404).body("Path not found"));
    let server_list = warp::path!("server").map(list);
    let server_get = warp::path!("server" / String).map(get_server);
    let server_create = warp::path!("server").map(create);
    let server_edit = warp::path!("server" / String / "edit").map(|_| "edit");
    let server_delete = warp::path!("server" / String / "delete").map(|_| "ded");
    let server_start = warp::path!("server" / String / "start").map(start);
    let server_stop = warp::path!("server" / String / "stop").map(stop);
    let ws_console = warp::path!("server" / String / "console").and(warp::ws()).map(console);

    let get_methods = warp::get().and(server_list.or(server_get));
    let post_methods = warp::post().and(server_create.or(server_edit).or(server_delete).or(server_start).or(server_stop));
    let routes = get_methods.or(post_methods).or(ws_console).or(not_found);

    let servers_path = "./servers";
    let local_server_map = load_servers(servers_path);
    let tmp = Mutex::new(local_server_map);
    match GLOBAL_SERVER_MAP.set(tmp) {
        Ok(_) => (),
        Err(_) => panic!("Attempted to set an already-set global variable.  Something is very very wrong.")
    };

    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}

fn list() -> String {
    let map = get_server_map();
    let list: Vec<&Server> = map.values().collect();
    serde_json::to_string(&list).expect("Couldn't serialize server list")
}

fn get_server(id: String) -> String {
    let list = get_server_map();
    match list.get(id.as_str()) {
        Some(server) => serde_json::to_string(&server).expect("Error serializing server to JSON"),
        None => "test".to_string()
    }
}

fn create() -> &'static str {
    "u just made a server congrats"
}

fn start(id: String) -> String {
    let mut map = get_server_map();
    if !map.contains_key(&id) {
        return "Error: Couldn't find specified server.".to_string();
    }
    let srv = map.get_mut(&id).unwrap();
    match srv.start() {
        Ok(_) => "Server started successfully".to_string(),
        Err(e) => "Error: ".to_owned() + &e.to_string()
    }
}

fn stop(id: String) -> String {
    let mut map = get_server_map();
    if !map.contains_key(&id) {
        return "Error: Couldn't find specified server.".to_string();
    }
    let srv = map.get_mut(&id).unwrap();
    match srv.stop() {
        Ok(_) => "Server stopped successfully".to_string(),
        Err(e) => "Error: ".to_owned() + &e.to_string()
    }
}

fn console(id: String, ws: warp::ws::Ws) -> impl warp::Reply {
    ws.on_upgrade(|websocket| {
        let mut map = get_server_map();
        let srv = map.get_mut(&"server".to_string()).unwrap(); // todo bad
        let file = match &srv.child_pipe {
            Some(o) => o,
            None => panic!()
        }.as_ref();
        let copy;
        unsafe {
            let fd = file.as_raw_fd();
            copy = File::from_raw_fd(fd);
        }
        let tf = tokio::fs::File::from_std(copy);
        let (tx, rx) = websocket.split();

        rx.forward(tx).map(|_| {})
    })
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
    child_pipe: Option<Arc<File>>,
    #[serde(skip)]
    yaml_path: String,
    #[serde(skip)]
    process: Option<Child>,
}

impl Server {
    /// Verify that the parameters on this struct make sense.
    /// This checks:
    ///  - that the jarfile exists
    ///  - that the server pwd exists and is a directory
    ///  - that the backing YAML file exists
    ///  - that the server id is valid (only uses alphanumeric and -_ characters)
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

    /// Check if the server subprocess is still running.
    fn is_alive(&mut self) -> bool {
        if self.process.is_none() {
            return false;
        }
        match self.process.as_mut().unwrap().try_wait() { // TODO save exit status?
            Ok(Some(_x)) => false,
            Ok(None) => true,
            Err(_) => panic!("Error trying to wait for server process")
        }
    }

    /// Try to start the server.
    /// This method will attempt to run the server command, using the parameters specified
    /// as part of the Server struct.  It will return an error if there is already a server
    /// process running for this Server.
    /// This method also waits for 200ms after spawning the server, then checks if it is
    /// still alive before returning a success value.  This is to catch errors such as
    /// incorrect JVM arguments or an unset EULA that cause the process to immediately exit.
    fn start(&mut self) -> Result<(), Error> {
        // dbg!(&self.process);
        if self.is_alive() {
            return Err(Error::SubprocessError("Server is already running"));
        }
        let mut proc = process::Command::new("java");
        if self.jvm_args != "" {
            for arg in self.jvm_args.split(" ") {
                proc.arg(arg);
            }
        }
        proc.arg("-jar");
        proc.arg(&self.jarfile);
        dbg!(&proc);
        proc.current_dir(&self.pwd);
        unsafe {
            let mut fds: [c_int; 2] = [0, 0];
            let rv = libc::pipe(&mut fds[0] as *mut c_int);
            assert!(rv == 0); // This must succeed, otherwise urbad
            // So apparently when you create an Stdio, it takes exclusive ownership
            // of the named fd, and in particular is responsible for cleaning it up when
            // the Stdio goes out of scope.  This might cause some issues with the two
            // Stdio objects trying to close the same file, but who knows?
            proc.stdout(Stdio::from_raw_fd(fds[1]));
            proc.stderr(Stdio::from_raw_fd(fds[1]));
            // println!("File descriptors: {}, {}", fds[0], fds[1]);
            // Also lets cross out fingers that closing the input side of the pipe doesn't somehow invalidate the output side
            self.child_pipe = Some(Arc::new(File::from_raw_fd(fds[0])));
        }
        match proc.spawn() {
            Ok(child_process) => self.process = Some(child_process),
            Err(_e) => { return Err(Error::SubprocessError("Error spawning child process")); }
        };
        std::thread::sleep(std::time::Duration::from_millis(200));
        if !self.is_alive() {
            return Err(Error::SubprocessError("Server died prematurely: "));
        }
        Ok(())
    }

    /// Stop the server if it is currently running.  Currently just terminates the process.
    fn stop(&mut self) -> Result<(), Error> {
        if !self.is_alive() {
            return Err(Error::SubprocessError("Server is not currently running"));
        }
        self.process.as_mut().unwrap().kill().expect("Couldn't send signal");
        if self.is_alive() { // It could just be taking a little bit to die, give it some time
            std::thread::sleep(std::time::Duration::from_millis(1000)); // TODO may want to make this configurable?
            if self.is_alive() {
                return Err(Error::SubprocessError("Child is not responding to kill signal."));
            }
        }
        Ok(())
    }
}

/// Given the path of a folder which contains server YAML files,
/// parse and load all of the YAML files in that directory as servers.
/// They are placed into a HashMap, keyed by server ID.
fn load_servers(path: &str) -> HashMap<String, Server> {
    println!("Attempting to load server yaml files...");
    let mut server_map: HashMap<String, Server> = HashMap::new();
    for entry in read_dir(Path::new(path)).unwrap() { // TODO give a correct error message and crash gracefully when path is not found
        let entry = entry.unwrap();
        let path = entry.path();
        if !path.is_dir() && path.extension().unwrap_or("".as_ref()) == "yaml" {
            // read file
            println!("Attempting to load server from {} ...", path.to_str().unwrap());
            match load_server(path.to_str().unwrap()) {
                Ok(srv) => {
                    server_map.insert(srv.id.clone(), srv);
                    ()
                }
                Err(e) => eprintln!("Error while loading server from {}: {}", path.to_str().unwrap(), e)
            }
        }
    }
    println!("Done!  {} servers loaded", server_map.len());
    return server_map;
}

/// Load a single server in from a YAML file, returning an Error
/// if an error is encountered during load.
fn load_server(filename: &str) -> Result<Server, Error> {
    let file = File::open(filename)?;
    let mut server: Server = serde_yaml::from_reader(file)?;
    server.yaml_path = filename.to_string();
    server.verify()?;
    return Ok(server);
}
