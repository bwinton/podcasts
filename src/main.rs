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

use util::PodcastError;

use rayon::prelude::*;
use rss::Channel;
use rss::Item;
use select::document::Document;
use std::convert::From;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

fn get_urls(podcast: &str) -> Vec<String> {
    let urls = File::open(format!("{}.urls", podcast)).unwrap();
    let mut buf_reader = BufReader::new(urls);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).unwrap();
    contents.lines().map(|x| x.to_owned()).collect()
}

fn get_rss(podcast: &str) -> Result<Channel, PodcastError> {
    let xml = File::open(format!("dist/{}.xml", podcast))?;
    Channel::read_from(BufReader::new(xml)).map_err(|error| PodcastError::new(error.description()))
}

fn process_document(url: &str, document: &Document) -> Result<Item, PodcastError> {
    match url {
        x if spodcast::matches(x) => spodcast::get_info(url, document),
        x if diecast::matches(x) => diecast::get_info(url, document),
        _ => Err(PodcastError::new(&format!("Unknown podcast {}", url))),
    }
}

fn get_item(url: &str) -> Result<Item, PodcastError> {
    // Get the html and build an Item.
    let mut response = reqwest::get(url)?;
    let body = response.text()?;
    let document = Document::from(body.as_str());

    process_document(url, &document)
}

fn handle(podcast: &str) {
    // Read podcast.urls and dist/podcast.xml
    let urls = get_urls(podcast);
    let mut rss_data = get_rss(podcast).unwrap();
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
                if item.is_err() {
                    println!("Error! {} {:?}", url, item);
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
