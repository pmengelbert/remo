use std::io::prelude::*;
use std::os::unix::net::{UnixListener, UnixStream};

fn main() -> std::io::Result<()> {
    let l = UnixListener::bind("/var/run/rtorrent/rpc.socket").expect("couldn't create socket");

    match l.accept() {
        Ok((mut socket, addr)) => {
            let mut b = vec![0_u8; 30000];

            socket.read(&mut b)?;

            std::fs::write("./req.xml", b)?;
        }
        Err(e) => println!("err!"),
    }
    Ok(())
}
