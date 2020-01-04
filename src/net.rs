use std::io::{Read, Error, ErrorKind};
use std::collections::HashMap;

pub fn get_packet(istream: &mut dyn Read) -> Result<HashMap<String, String>, Error> {
    let mut last = 0u8;
    let mut buf = [0u8];
    let mut packet: Vec<u8> = Vec::new();
    while last != 0x0A || buf[0] != 0x0A {
        last = buf[0];
        let n = istream.read(&mut buf)?;
        if n != 1 {
            return Err(Error::new(ErrorKind::BrokenPipe, "GARBAGE"));
        }
        packet.push(buf[0]);
    }

    return Ok(parse_packet(packet));
}

fn parse_packet(packet: Vec<u8>) -> HashMap<String, String> {
    let mut line: Vec<u8> = Vec::new();
    let mut map: HashMap<String, String> = HashMap::new();
    for element in packet {
        if element == 0x0A { // EOL
            if !line.contains(&0x3Au8) {
                dbg!("Ignoring line");
                line = Vec::new();
                continue; // this line didn't have a colon separator, ignore it
            }
            let pieces: Vec<&[u8]> = line.split(|s| *s == 0x3A).collect(); // ':' character
            let name = match String::from_utf8(Vec::from(pieces[0])) {
                Ok(t) => t,
                Err(_e) => break // this is bad, we're done TODO handle this better
            };
            let data = String::from_utf8(Vec::from(&line[pieces[0].len() + 1..line.len()])).unwrap();
            line = Vec::new();
            map.insert(name, data);
        } else {
            line.push(element);
        }
    }

    return map;
}