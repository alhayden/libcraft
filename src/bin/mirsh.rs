use std::env;
use std::os::unix::net::UnixStream;
use std::io::{Write, Read};
use libcraft::net::{get_packet, send_packet};
use std::collections::HashMap;
use std::time::Duration;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        //no args, print help
        1 => {
            print_cmd_list();
            std::process::exit(0);
        },
        //handle multi-argument commands
        _ => {
            match &args[1][..] {
                "debug" => debug(args),
                "help" => help(args),
                "list" => list(args),
                "create" => create(args),
                "start" => start(args),
                "stop" => stop(args),
                "force-stop" => force_stop(args),
                "restart" => restart(args),
                _ => {
                    println!("{}", &args[0][..]);
                    println!("The command you entered is not recognized.  \nUse 'mirsh help' for information about valid commands.");
                    print_cmd_list();
                    std::process::exit(1);
                }
            }
        }
    }

}

fn debug(args: Vec<String>) {
    let mut stream = UnixStream::connect("libcraftd.sock").unwrap();
    if args.len() <= 2 {
        std::process::exit(1);
    }
    let mut i = 2;
    loop {
        stream.write(args[i].as_bytes()).unwrap();
        stream.write("\n".as_bytes()).unwrap();
        i += 1;
        if i == args.len() { break;}
    }
    println!("Message sent, awaiting reply:");
    loop {
        let mut buf: [u8; 1] = [0];
        stream.read(&mut buf).unwrap();
        std::io::stdout().write(&buf).unwrap();
    }
}


fn print_cmd_list() {
    println!("\nUsage:\nmirsh <command> [args...]");
    println!("\n  commands:");
    println!("   list             lists the defined servers");
    println!("   create           creates a new server");
    println!("   start            starts a server that is not running");
    println!("   stop             issues a stop command to a running server");
    println!("   force-stop       kills a server externally.  This could lead to data loss");
    println!("   restart          sends the stop command to a server and then starts it again");
    println!("   clone            duplicate a server to another location");
    println!("   backup           create a backup of a server");
    println!("   edit             open a server's config file for editing");
    println!("   console          connect to a server's console to issue commands");

    println!();
}

fn help(args: Vec<String>) {

}

fn list(args: Vec<String>) {

}

fn create(args: Vec<String>) {

}

fn start(args: Vec<String>) {
    send_arg_print_result(args, String::from("start"));
}

fn stop(args: Vec<String>) {
    send_arg_print_result(args, String::from("stop"));
}

fn force_stop(args: Vec<String>) {
    send_arg_print_result(args, String::from("force-stop"));
}

fn restart(args: Vec<String>) {
    send_arg_print_result(args, String::from("restart"));
}

fn send_arg_print_result(args: Vec<String>, action: String) {
    if args.len() < 3 || args.len() > 3 {
        println!("Error: Invalid arguments");
        println!("Usage: mirsh {} <server>", action);
        std::process::exit(1);
    }
    let mut out_pack: HashMap<String, String> = HashMap::new();
    out_pack.insert(String::from("action"), action);
    out_pack.insert("name".to_string(), String::from(&args[2]));

    let mut stream = UnixStream::connect("libcraftd.sock").unwrap();
    stream.set_read_timeout(Some(Duration::new(10, 0)));

    send_packet(&mut stream, out_pack);

    let packet = match get_packet(&mut stream) {
        Ok(p) => p,
        Err(e) => {
            println!("{}", e);
            std::process::exit(3);
        }
    };
    if packet.contains_key("result") {
        println!("{}", packet.get("result").unwrap());
    }
}
