use std::io::Write;

use anyhow::Result;
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
    jackettindexer: String,
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
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        print_help();
        return Ok(());
    }

    let mut opt = getopt::Parser::new(&args, "has:e:1n:d");
    let api_key = std::env::var("JACKETT_API_KEY").unwrap_or_else(|_| "password".to_owned());

    let mut season = String::new();
    let mut episode = String::new();
    let mut download_first = false;
    let mut num = 5_usize;
    let mut print_all = false;

    while let Some(o) = opt.next().transpose()? {
        match o {
            getopt::Opt('h', None) => {
                print_help();
                return Ok(());
            }
            getopt::Opt('s', Some(s)) => season = s.clone(),
            getopt::Opt('e', Some(e)) => episode = e.clone(),
            getopt::Opt('n', Some(n)) => num = n.parse::<usize>()?,
            getopt::Opt('1', None) => download_first = true,
            getopt::Opt('a', None) => print_all = true,
            getopt::Opt('*', _) => (),
            _ => unreachable!(),
        }
    }

    let loose = opt.loose;
    let search_term: String = loose.join("+");
    let mut url = format!(
        "http://10.0.0.3:30333/api/v2.0/indexers/all/results/torznab?apikey={}&t=search&q={}",
        api_key, search_term
    );

    let is_tv_show = !season.is_empty() || !episode.is_empty();
    if is_tv_show {
        if season.is_empty() {
            return Err(anyhow::format_err!(
                "if -e is provided, -s must also be provided"
            ));
        }

        url = format!("http://10.0.0.3:30333/api/v2.0/indexers/all/results/torznab?apikey={}&t=tvsearch&q={}&season={}", api_key, search_term, season);
        if !episode.is_empty() {
            url.push_str("&ep=");
            url.push_str(&episode);
        }
    }

    let doc = reqwest::blocking::get(&url)?.text()?;

    let doc: Channel = serde_xml_rs::from_str(&doc).unwrap_or_default();
    if doc.channel.len() == 0 {
        let s = format!("no torrents found for search term {:?}", &loose);
        return if download_first {
            Err(anyhow::format_err!("{}", s))
        } else {
            Ok(())
        };
    }

    if download_first {
        call_rpc(&["load.start", "", &doc.channel[0].item[0].link])?;
        return Ok(());
    }

    let mut count = 0;
    let mut seeders = String::new();

    num = if print_all {
        doc.channel[0].item.len()
    } else {
        std::cmp::min(doc.channel[0].item.len(), num)
    };

    for t in &doc.channel[0].item[..num] {
        let size = t.size.parse::<u64>()?;
        let size = parse_size(size)?;

        for attr in &t.attr {
            if attr.name == "seeders" {
                seeders = attr.value.clone();
            }
        }

        println!(
            "{})\tname:\t\t{}\n\tseeders:\t{}\n\tsize:\t\t{}\n\tindexer:\t{}\n",
            count, t.title, seeders, size, t.jackettindexer,
        );

        count += 1;
    }

    let mut s = String::new();
    print!("which one? ");
    std::io::stdout().flush()?;
    std::io::stdin().read_line(&mut s)?;
    let i: usize = s.trim().parse()?;

    call_rpc(&["load.start", "", &doc.channel[0].item[i].link])?;
    Ok(())
}

fn parse_size(size: u64) -> Result<String> {
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

fn print_help() {
    println!(
        r"Usage: ts [OPTIONS...] <SEARCH TERM...>

Options:
    -h          print this help
    -s NUM      specify season
    -e NUM      specify the episode, must be used in conjunction with -s
    -n NUM      print NUM results (default 5)
    -1          download the top search result
    -a          print all results
"
    )
}
