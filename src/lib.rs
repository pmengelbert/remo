use anyhow::{anyhow, Result};
use serde_xmlrpc::{request_to_string, response_from_str, Value};
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

    let unwrapped = unwrap_response(&response).to_owned();
    let mut resp = String::with_capacity(unwrapped.len());

    if let Ok(v) = response_from_str::<Vec<String>>(&unwrapped) {
        for s in v {
            resp.push_str(&s);
            resp.push('\n');
        }
    } else if let Ok(i) = response_from_str::<i64>(&unwrapped) {
        resp.push_str(&i.to_string());
    } else if let Ok(s) = response_from_str::<String>(&unwrapped) {
        resp = s;
    } else if let Ok(v) = response_from_str::<Vec<Vec<String>>>(&unwrapped) {
        for vv in v {
            resp.push_str("[\n");
            for vvv in vv {
                resp.push('\t');
                resp.push_str(&vvv);
                resp.push('\n');
            }
            resp.push_str("],\n");
        }
    }

    if resp.is_empty() {
        return Ok(unwrapped);
    }

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
