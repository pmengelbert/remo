use anyhow::{anyhow, Result};
use serde_xmlrpc::{request_to_string, value_from_str, Value};
use std::io::prelude::*;
use std::os::unix::net::UnixStream;
pub fn unwrap_response(resp: &str) -> &str {
    let first = resp.find('<').unwrap_or(resp.len());
    &resp[first..]
}

pub fn call_rpc<T: AsRef<str>>(cmd_args: &[T]) -> Result<String> {
    let cmd = cmd_args
        .get(0)
        .ok_or_else(|| anyhow!("need at least one arg"))?;

    let args: Vec<Value> = cmd_args
        .into_iter()
        .skip(1)
        .map(|s| Value::String(s.as_ref().to_owned()))
        .collect();

    let call = request_to_string(cmd.as_ref(), args)?;
    let msg = wrap_xml_request(call.as_bytes());

    let mut stream = UnixStream::connect("/var/run/rtorrent/rpc.socket")?;
    stream.write_all(&msg)?;

    let mut response = String::new();
    stream.read_to_string(&mut response)?;

    let v = value_from_str(&response)?;
    let resp = match v {
        Value::Int(i) => format!("{}", i),
        Value::Bool(b) => format!("{}", b),
        Value::Int64(i) => format!("{}", i),
        Value::Array(v) => {
            let mut resp = String::new();
            for w in v {
                if let Value::String(s) = w {
                    resp.push_str(&s);
                }
            }
            resp
        }
        Value::String(s) => s,
        Value::Double(f) => format!("{}", f),
        Value::Base64(s) => String::from_utf8(s)?,
        Value::DateTime(d) => d.to_string(),
        _ => return Err(anyhow!("bad value returned")),
    };

    Ok(resp)
}

pub fn add_header(v: &mut Vec<u8>, key: &str, value: &str) {
    v.extend(key.as_bytes());
    v.push(0);

    v.extend(value.as_bytes());
    v.push(0);
}

pub fn wrap_xml_request(xml: &[u8]) -> Vec<u8> {
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
