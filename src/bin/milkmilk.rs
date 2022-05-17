use amiquip::{Connection, ConsumerMessage, ConsumerOptions, QueueDeclareOptions};
use anyhow::Result;
use remo::call_rpc;
use reqwest::blocking::Client;
use std::collections::HashMap;

fn main() -> Result<()> {
    let mut conn = Connection::insecure_open("amqp://guest:guest@10.0.0.3:30666")?;
    let chan = conn.open_channel(None)?;
    let q = chan.queue_declare("milkmilk", QueueDeclareOptions::default())?;
    let c = Client::new();

    let cnsm = q.consume(ConsumerOptions::default())?;
    println!("Waiting for messages. Press Ctrl-C to exit");

    for message in cnsm.receiver().iter() {
        match message {
            ConsumerMessage::Delivery(d) => {
                let body: HashMap<String, String> = serde_json::from_slice(&d.body)?;
                c.post("http://10.0.0.3:30777/")
                    .body(body.get("title").unwrap().to_owned())
                    .send()?;

                let z = call_rpc(&["load.start", "", body.get("download").unwrap()])?;
                println!("{}", z);
                cnsm.ack(d)?;
            }
            other => {
                println!("Consumer ended: {:?}", other);
                break;
            }
        }
    }

    Ok(())
}
