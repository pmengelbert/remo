use anyhow::{anyhow, Result};
use simple_xmlrpc::{parse_value, stringify_request, Value};
use std::io::prelude::*;
use std::os::unix::net::UnixStream;

fn main() -> Result<()> {
    let y: Vec<String> = std::env::args().skip(1).collect();
    let resp = call_rpc(&y)?;
    println!("{}", resp);
    Ok(())
}

fn call_rpc<T: AsRef<str>>(cmd_args: &[T]) -> Result<String> {
    let cmd = cmd_args
        .get(0)
        .ok_or_else(|| anyhow!("need at least one arg"))?;

    let args: Vec<Value> = cmd_args
        .iter()
        .skip(1)
        .map(|s| parse_value(s.as_ref()).unwrap())
        .collect();

    let call = stringify_request(cmd.as_ref(), &args)?;
    let msg = wrap_xml_request(call.as_bytes());
    let mut stream = UnixStream::connect("/var/run/rtorrent/rpc.socket")?;
    stream.write_all(&msg)?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    Ok(response)
}

fn add_header(v: &mut Vec<u8>, key: &str, value: &str) {
    v.extend(key.as_bytes());
    v.push(0);

    v.extend(value.as_bytes());
    v.push(0);
}

fn wrap_xml_request(xml: &[u8]) -> Vec<u8> {
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
