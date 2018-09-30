use std::fs::File;
use std::io::prelude::*;

fn main() {
    let mut f = File::open("file.ts").expect("file not found");
    let mut buf = vec![];
    let _ = f.read_to_end(&mut buf);
    println!("{}", buf.len());
    let mut packets_count = 0;
    for chunk in buf.chunks(188) {
        packets_count += 1;
        println!("is it sync byte {}", chunk[0] == 71);
    }
    println!("{}", packets_count);
}
