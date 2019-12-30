use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        //no args, print help
        1 => {
            help();
            std::process::exit(0);
        },
        //handle single-argument commands
        2 => {

        },
        //handle multi-argument commands
        _ => {

        }
    }

}

fn print_exit() {
    print!("Invalid arguments.  Try 'mirsh help' for a list of commands.");
    std::process::exit(1);
}

fn help() {
    println!("\nUsage:\nmirsh <command> [args...]");
    println!("\n  commands:");
    println!("   create           creates a new server");
    println!("   start            starts a server that is not running");
    println!("   stop             issues a stop command to a running server");
    println!("   force-stop       kills a server externally.  This could lead to data loss");
    println!("   restart          sends the stop command to a server and then starts it again");
}