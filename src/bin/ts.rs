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

struct Options {
    season: String,
    episode: String,
    download_first: bool,
    print_all: bool,
    help: bool,
    print_num: usize,
    loose: Vec<String>,
}

impl std::default::Default for Options {
    fn default() -> Self {
        Self {
            season: String::default(),
            episode: String::default(),
            loose: Vec::default(),
            download_first: false,
            print_all: false,
            help: false,
            print_num: 5,
        }
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 1 {
        print_help();
        return Ok(());
    }

    let api_key = std::env::var("JACKETT_API_KEY").unwrap_or_else(|_| "password".to_owned());
    let Options {
        help,
        print_all,
        download_first,
        season,
        episode,
        mut print_num,
        loose,
    } = parse_options(&args, "ha1s:e:n:")?;

    if help {
        print_help();
        return Ok(());
    }

    let search_term: String = loose.join("+");
    let base_url = format!(
        "http://10.0.0.3:30333/api/v2.0/indexers/all/results/torznab?apikey={}",
        api_key
    );
    let url = match &(season.len(), episode.len()) {
        (0, 0) => format!("{}&t=search&q={}", base_url, search_term),
        (1.., 0) => format!(
            "{}&t=tvsearch&q={}&season={}",
            base_url, search_term, season
        ),
        (1.., 1..) => format!(
            "{}&t=tvsearch&q={}&season={}&ep={}",
            base_url, search_term, season, episode
        ),
        (0, 1..) => {
            return Err(anyhow::format_err!(
                "if -e is provided, -s must also be provided"
            ))
        }
        (_, _) => unreachable!(),
    };

    let c = reqwest::blocking::Client::new();
    let doc = c.get(&url).send()?.text()?;
    let doc: Channel = serde_xml_rs::from_str(&doc).unwrap_or_default();

    if doc.channel.len() == 0 {
        let s = format!("no torrents found for search term {:?}", &loose);
        if download_first {
            return Err(anyhow::format_err!("{}", s));
        }

        return Ok(());
    }

    let channel = &doc.channel[0];

    if download_first {
        let item = &channel.item[0];
        call_rpc(&["load.start", "", &item.link])?;
        println!("{}", notify(&c, &item.title)?);
        return Ok(());
    }

    print_num = if print_all {
        channel.item.len()
    } else {
        std::cmp::min(channel.item.len(), print_num)
    };

    let mut count = 0;

    for t in &channel.item[..print_num] {
        let size = t.size.parse::<u64>()?;
        let size = parse_size(size)?;

        let seeders = t
            .attr
            .iter()
            .find(|a| a.name == "seeders")
            .map(|a| a.value.as_str())
            .unwrap_or_default();

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

    let item = &channel.item[i];
    call_rpc(&["load.start", "", &item.link])?;
    println!("{}", notify(&c, &item.title)?);

    Ok(())
}

fn notify(c: &reqwest::blocking::Client, msg: &str) -> Result<String> {
    let resp = c
        .request(reqwest::Method::POST, "http://10.0.0.3:30777/")
        .body(msg.to_owned())
        .send()?;

    Ok(resp.text()?)
}

fn parse_options(args: &[String], getopt: &'static str) -> Result<Options> {
    let mut opt = getopt::Parser::new(&args, getopt);
    let mut opts = Options::default();

    while let Some(o) = opt.next().transpose()? {
        match o {
            getopt::Opt('h', None) => {
                opts.help = true;
                return Ok(opts);
            }
            getopt::Opt('s', Some(s)) => opts.season = s.clone(),
            getopt::Opt('e', Some(e)) => opts.episode = e.clone(),
            getopt::Opt('n', Some(n)) => opts.print_num = n.parse::<usize>()?,
            getopt::Opt('1', None) => opts.download_first = true,
            getopt::Opt('a', None) => opts.print_all = true,
            getopt::Opt('*', _) => (),
            _ => unreachable!(),
        }
    }

    opts.loose = opt.loose;
    Ok(opts)
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
