use std::io::{Write, Read, BufReader};
use std::process::{Command, Stdio};


fn main() {
    let server_process = match Command::new("python")
        .arg("do.py")
        .stdout(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn() {
            Err(why) => panic!("Failed to start server: {}", why),
            Ok(process) => process,
        };
//    let output = server_process.wait_with_output();
    let reader = BufReader::new(server_process.stdout.as_mut().unwrap());
    loop {
        let mut line = String::new();
        /*let len = */reader.read_line(&mut line).unwrap();
        println("{}", line);
//        reader.lines().for_each(|line| println!("{}", line));
    }

}
