use std::io::prelude::*;
use std::os::unix::net::UnixStream;

fn main() -> std::io::Result<()> {
    let b = include_bytes!("../req.xml");

    dbg!("1");
    let mut stream = UnixStream::connect("/var/run/rtorrent/rpc.socket")?;
    stream.write_all(b)?;

    dbg!("2");
    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    println!("{}", response);
    Ok(())
}
