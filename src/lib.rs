use anyhow::{Context, Result};
use serde_xmlrpc::{request_to_string, response_from_str, Value};
use std::borrow::Cow;
use std::io::prelude::*;
use std::os::unix::net::UnixStream;

pub fn unwrap_response(resp: &str) -> &str {
    let first = resp.find('<').unwrap_or(resp.len());
    &resp[first..]
}

pub fn call_rpc<T: AsRef<str>>(cmd_args: &[T]) -> Result<String> {
    let cmd = cmd_args.get(0).context("need at least one arg")?;

    let args = cmd_args
        .iter()
        .skip(1)
        .map(|s| Value::String(s.as_ref().to_owned()))
        .collect();

    let call = request_to_string(cmd.as_ref(), args)?;
    let msg = wrap_xml_request(call.as_bytes());

    let read_socket = std::env::var("REMO_RPC_SOCKET").map_or_else(
        |_| Cow::Borrowed("/var/run/rtorrent/rpc.socket"),
        Cow::Owned,
    );

    let mut stream = UnixStream::connect(read_socket.as_ref())?;
    let raw_response = make_rpc_request(&mut stream, msg)?;
    let resp = build_custom_response(&raw_response);

    Ok(resp.into_owned())
}

fn make_rpc_request(stream: &mut UnixStream, msg: Vec<u8>) -> Result<String> {
    stream.write_all(&msg)?;
    let mut raw_response = String::new();
    stream.read_to_string(&mut raw_response)?;
    Ok(raw_response)
}

fn build_custom_response(orig: &str) -> Cow<str> {
    let unwrapped = unwrap_response(orig);
    let len = unwrapped.len();

    if let Ok(v) = response_from_str::<Vec<String>>(unwrapped) {
        return Cow::Owned(prepare_string_response(v, len));
    }

    if let Ok(i) = response_from_str::<i64>(unwrapped) {
        return Cow::Owned(i.to_string());
    }

    if let Ok(s) = response_from_str::<String>(unwrapped) {
        return Cow::Owned(s);
    }

    if let Ok(matrix) = response_from_str::<Vec<Vec<String>>>(unwrapped) {
        return Cow::Owned(prepare_matrix_response(matrix, len));
    }

    Cow::Borrowed(unwrapped)
}

fn prepare_matrix_response(matrix: Vec<Vec<String>>, len: usize) -> String {
    let mut r = String::with_capacity(len);

    for row in matrix {
        r.push_str("[\n");
        for column in row {
            r.push('\t');
            r.push_str(&column);
            r.push('\n');
        }
        r.push_str("],\n");
    }

    r
}

fn prepare_string_response(v: Vec<String>, len: usize) -> String {
    let mut r = String::with_capacity(len);

    for s in v {
        r.push_str(&s);
        r.push('\n');
    }

    r
}

pub fn add_header(v: &mut Vec<u8>, key: &str, value: &str) {
    v.extend(key.as_bytes());
    v.push(0);

    v.extend(value.as_bytes());
    v.push(0);
}

pub fn wrap_xml_request(xml: &[u8]) -> Vec<u8> {
    let mut header = Vec::new();
    add_header(&mut header, "CONTENT_LENGTH", &xml.len().to_string());
    add_header(&mut header, "HTTP_ACCEPT", "*/*");
    add_header(&mut header, "HTTP_CONTENT_TYPE", "text/xml");
    add_header(&mut header, "HTTP_CONTENT_LENGTH", &xml.len().to_string());

    let mut msg: Vec<u8> = vec![];
    msg.extend(format!("{}:", header.len()).as_bytes());
    msg.extend(header);
    msg.push(b',');
    msg.extend(xml);

    msg
}
