#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;

mod util;
mod diecast;
mod spodcast;

extern crate chrono;
extern crate mp3_duration;
extern crate rayon;
extern crate regex;
extern crate reqwest;
extern crate rss;
extern crate select;
extern crate url;

use util::*;

use failure::ResultExt;
use rayon::prelude::*;
use rss::Channel;
use rss::Item;
use select::document::Document;
use std::convert::From;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

fn get_urls(podcast: &str) -> Result<Vec<String>> {
    let urls = File::open(format!("{}.urls", podcast))
        .context(format_err!("Error opening {}.urls", podcast))?;
    let mut buf_reader = BufReader::new(urls);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)
        .context(format_err!("Error reading {}.urls", podcast))?;
    Ok(contents.lines().map(|x| x.to_owned()).collect())
}

fn get_rss(podcast: &str) -> Result<Channel> {
    let xml = File::open(format!("dist/{}.xml", podcast))
        .context(format_err!("Error opening {}.xml", podcast))?;
    Channel::read_from(BufReader::new(xml))
        .context(format_err!("Error opening {}.xml", podcast))
        .map_err(From::from)
}

fn process_document(url: &str, document: &Document) -> Result<Item> {
    match url {
        x if spodcast::matches(x) => spodcast::get_info(url, document),
        x if diecast::matches(x) => diecast::get_info(url, document),
        _ => Err(format_err!("Unknown podcast: {}", url)),
    }
}

fn get_item(url: &str) -> Result<Item> {
    // Get the html and build an Item.
    let mut response = reqwest::get(url)?;
    let body = response.text()?;
    let document = Document::from(body.as_str());

    process_document(url, &document)
}

fn handle(podcast: &str) {
    // Read podcast.urls and dist/podcast.xml
    let urls = match get_urls(podcast) {
        Err(ref e) => {
            print_error(e);
            return;
        },
        Ok(urls) => urls
    };

    let mut rss_data = match get_rss(podcast) {
        Err(ref e) => {
            print_error(e);
            return;
        },
        Ok(rss_data) => rss_data
    };

    println!("{}: {}/{}", podcast, rss_data.items().len(), urls.len());
    let items: Vec<_> = urls.par_iter()
        .map(|url| {
            if url.starts_with('#') {
                None
            } else if let Some(found) = rss_data
                .items()
                .iter()
                .find(|item| item.link() == Some(url))
            {
                Some(found.clone())
            } else {
                // Find any missing urls.
                // println!("Missing {}", url);
                let item = get_item(url);
                if let Err(ref e) = item {
                    // println!("Error in {}", url);
                    print_error(e);
                }
                item.ok()
            }
        })
        .filter_map(|x| x)
        .collect();
    // Write out the new dist/podcast.xml
    rss_data.set_items(items);
    let output = File::create(format!("dist/{}.xml", podcast)).unwrap();
    rss_data.pretty_write_to(output, b' ', 2).unwrap();
}

// use std::path::Path;
fn main() {
    let podcasts = vec!["spodcast", "diecast"];
    // For podcast in spodcast/diecast
    podcasts.par_iter().for_each(|podcast| handle(podcast));
    // let result = process_document("http://www.shamusyoung.com/twentysidedtale/?p=41977",
    // &Document::from(include_str!("../diecast.html"))).ok();
    // println!("\n{:?}", result);
    // let path = Path::new("mumblo.mp3");
    // let duration = mp3_duration::from_path(&path).unwrap();
    // println!("\n{:?}", duration);
}
