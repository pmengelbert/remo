use std::io::Write;

use anyhow::Result;
use getopt_rs::opt;
use remo::call_rpc;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
struct Attr {
    name: String,
    value: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Torrent {
    title: String,
    link: String,
    size: String,
    attr: Vec<Attr>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Item {
    title: String,
    item: Vec<Torrent>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Channel {
    channel: Vec<Item>,
}

fn main() -> Result<()> {
    let mut args: Vec<String> = std::env::args().collect();

    let mut season = String::new();
    let mut episode = String::new();
    let mut download_first = false;
    let mut num = 5_usize;
    let mut interactive = false;

    while let Some(o) = getopt_rs::getopt(
        &mut args,
        "is:e:n:",
        &[opt!('s'), opt!('e'), opt!('n'), opt!('i')],
    ) {
        match o {
            ('s', s) => season = s.unwrap().clone(),
            ('e', e) => episode = e.unwrap().clone(),
            ('n', n) => num = n.unwrap().parse::<usize>()?,
            ('1', _) => download_first = true,
            ('i', _) => interactive = true,
            _ => unreachable!(),
        }
    }

    let new_args = args;

    dbg!(season, episode, num, download_first, interactive, &new_args);
    // let search_term: String = new_args.join("+");
    // let mut url = format!("http://10.0.0.3:30333/api/v2.0/indexers/all/results/torznab?apikey=sypom0i0ugs22oqoyvg9edyky5uasoy5&t=search&q={}", search_term);

    // let is_tv_show = !season.is_empty() && !episode.is_empty();
    // if is_tv_show {
    //     url = format!("http://10.0.0.3:30333/api/v2.0/indexers/all/results/torznab?apikey=sypom0i0ugs22oqoyvg9edyky5uasoy5&t=tvsearch&q={}&season={}&ep={}", search_term, season, episode);
    // }

    // let x = reqwest::blocking::get(&url)?.text()?;

    // let x: Channel = serde_xml_rs::from_str(&x).unwrap_or_default();
    // if x.channel.len() == 0 {
    //     println!("no torrents found for search term {:?}", &new_args);
    //     std::process::exit(0);
    // }

    // if download_first {
    //     call_rpc(&["load.start", "", &x.channel[0].item[0].link])?;
    //     std::process::exit(0);
    // }

    // if interactive {
    //     let mut count = 0;
    //     let mut seeders = String::new();

    //     for t in &x.channel[0].item[..num] {
    //         let size = t.size.parse::<usize>()?;
    //         let size = parse_size(size)?;

    //         for x in &t.attr {
    //             if x.name == "seeders" {
    //                 seeders = x.value.clone();
    //             }
    //         }

    //         println!(
    //             "{})\tname:\t\t{}\n\tseeders:\t{}\n\tsize:\t\t{}\n",
    //             count, t.title, seeders, size,
    //         );

    //         count += 1;
    //     }

    //     let mut s = String::new();
    //     print!("which one? ");
    //     std::io::stdout().flush()?;
    //     std::io::stdin().read_line(&mut s)?;
    //     let i: usize = s.trim().parse()?;

    //     call_rpc(&["load.start", "", &x.channel[0].item[i].link])?;
    //     std::process::exit(0);
    // }

    // // dbg!(&x.channel[0].item[..num]);
    Ok(())
}

fn parse_size(size: usize) -> Result<String> {
    let mut sz = size;

    let mut exp = 0;
    let mut last = sz as f64;
    while sz > 1024 {
        exp += 1;
        last = sz as f64;
        sz /= 1024;
    }
    last /= 1024.0;

    let suffix = match exp {
        0 => "b",
        1 => "k",
        2 => "M",
        3 => "G",
        _ => "b",
    };

    Ok(format!("{:.2}{}", last, suffix))
}
