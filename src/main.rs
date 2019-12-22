use std::process;
use std::io::Read;
use std::str::from_utf8;

fn main() {
    let server_process = match process::Command::new("bash")
        .arg("-c")
        .arg("for i in 1 2 3 4 5 6 7 8 9; do echo hello; sleep 1; done")
        .stdout(process::Stdio::piped())
        .stdin(process::Stdio::piped())
        .spawn() {
        Err(why) => panic!("Failed to start server: {}", why),
        Ok(proc) => proc,
    };
    let mut stdout = server_process.stdout.unwrap();
    loop {
        let mut buf: [u8; 10] = [0; 10];
        let n = stdout.read(&mut buf).unwrap();
        if n == 0 {
            break;
        }
        print!("{}", from_utf8(&buf[0..n]).unwrap());
    }
}