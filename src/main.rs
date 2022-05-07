use std::io::prelude::*;
use std::io::Cursor;
use std::os::unix::net::UnixStream;

struct Sock(UnixStream);

impl xmlrpc::Transport for Sock {
    type Stream = Cursor<Vec<u8>>;
    fn transmit(
        self,
        request: &xmlrpc::Request<'_>,
    ) -> Result<Self::Stream, Box<dyn std::error::Error + Send + Sync>> {
        let msg = wrap_xml_request(request);
        let mut sock = self.0;

        sock.write_all(&msg)?;

        let mut resp = Vec::new();
        sock.read_to_end(&mut resp)?;

        Ok(Cursor::new(resp))
    }
}

fn main() -> std::io::Result<()> {
    let cmd = std::env::args().nth(1).expect("need an arg");
    let x = xmlrpc::Request::new(&cmd);

    let s = x
        .call(Sock(UnixStream::connect("/var/run/rtorrent/rpc.socket")?))
        .unwrap();

    let mut v = Vec::new();
    s.write_as_xml(&mut v)?;
    let s = String::from_utf8(v).unwrap();

    // for arg in std::env::args().skip(2) {
    //     x = x.arg(arg);
    // }

    // let msg = wrap_xml_request(&x);

    // dbg!("1");
    // let mut stream = UnixStream::connect("/var/run/rtorrent/rpc.socket")?;
    // stream.write_all(&msg)?;

    // dbg!("2");
    // let mut response = String::new();
    // stream.read_to_string(&mut response)?;

    println!("{}", s);
    Ok(())
}

fn add_header(v: &mut Vec<u8>, key: &str, value: &str) {
    v.extend(key.as_bytes());
    v.push(0);

    v.extend(value.as_bytes());
    v.push(0);
}

fn wrap_xml_request(req: &xmlrpc::Request) -> Vec<u8> {
    let mut xml = vec![];
    req.write_as_xml(&mut xml).unwrap();

    let mut header = Vec::new();
    add_header(&mut header, "CONTENT_LENGTH", &format!("{}", xml.len()));
    add_header(&mut header, "HTTP_ACCEPT", "*/*");
    add_header(&mut header, "HTTP_CONTENT_TYPE", "text/xml");
    add_header(
        &mut header,
        "HTTP_CONTENT_LENGTH",
        &format!("{}", xml.len()),
    );

    let mut msg: Vec<u8> = vec![];
    msg.extend(format!("{}:", header.len()).as_bytes());
    msg.extend(header);
    msg.push(b',');
    msg.extend(xml);

    msg
}
