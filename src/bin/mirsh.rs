use std::env;
use std::os::unix::net::UnixStream;
use std::io::{Write, Read};

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
    let mut stream = UnixStream::connect("../../libcraftd.sock").unwrap();
    if args.len() <= 2 {
        std::process::exit(1);
    }
    /*for arg in args {
        stream.write_all(arg.as_bytes());
    }*/
    stream.write(args[2].as_bytes());
    stream.write("\n".as_bytes());
    println!("Message sent");
    loop {
        let mut resp = String::new();
        stream.read_to_string(&mut resp).unwrap();
        println!("{}", resp);
    }
}

fn print_exit() {
    print!("Invalid arguments.  Try 'mirsh help' for a list of commands.");
    std::process::exit(1);
}

fn print_cmd_list() {
    println!("\nUsage:\nmirsh <command> [args...]");
    println!("\n  commands:");
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

fn create(args: Vec<String>) {

}

fn start(args: Vec<String>) {

}

fn stop(args: Vec<String>) {

}

fn force_stop(args: Vec<String>) {

}

fn restart(args: Vec<String>) {

}
