use anyhow::Result;
use remo::call_rpc;

fn main() -> Result<()> {
    let y: Vec<String> = std::env::args().skip(1).collect();
    let resp = call_rpc(&y)?;
    // let resp = unwrap_response(&resp);

    println!("{}", resp);

    Ok(())
}
