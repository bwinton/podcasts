#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;

extern crate chrono;
extern crate mp3_duration;
extern crate rayon;
extern crate regex;
extern crate reqwest;
extern crate rss;
extern crate select;
extern crate url;

mod diecast;
mod spodcast;
mod util;
mod vortex_theatre;

use std::collections::HashMap;

use util::*;

use chrono::DateTime;
use failure::ResultExt;
use rayon::prelude::*;
use rss::Channel;
use rss::Item;
use select::document::Document;
use std::borrow::ToOwned;
use std::convert::From;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufReader;

fn get_urls(podcast: &str) -> Result<HashMap<String, Option<Item>>> {
    let urls = File::open(format!("{}.urls", podcast))
        .context(format_err!("Error opening {}.urls for reading", podcast))?;
    let mut buf_reader = BufReader::new(urls);
    let mut contents = String::new();
    buf_reader
        .read_to_string(&mut contents)
        .context(format_err!("Error reading {}.urls", podcast))?;

    let mut result: HashMap<String, Option<Item>> =
        contents.lines().map(|x| (x.to_owned(), None)).collect();
    let new_urls = match podcast {
        "diecast" => diecast::get_urls(&result)?,
        "vortex_theatre" => vortex_theatre::get_urls(&result)?,
        _ => HashMap::new(),
    };

    if !new_urls.is_empty() {
        for (url, item) in new_urls {
            result.insert(url, item);
        }
        // Add the new urls to the results and write it out.
        let mut keys: Vec<String> = result.keys().cloned().collect();
        keys.sort();
        keys.reverse();

        let mut urls = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(format!("{}.urls", podcast))
            .context(format_err!("Error opening {}.urls for writing", podcast))?;
        urls.write_all(&keys.join("\n").as_bytes())?;
    }
    Ok(result)
}

fn get_rss(podcast: &str) -> Result<Channel> {
    let xml = File::open(format!("{}.xml", podcast))
        .context(format_err!("Error opening {}.xml", podcast))?;
    Channel::read_from(BufReader::new(xml))
        .context(format_err!("Error opening {}.xml", podcast))
        .map_err(From::from)
}

fn process_document(url: &str, document: &Document) -> Result<Item> {
    match url {
        x if spodcast::matches(x) => spodcast::get_item(url, document),
        x if diecast::matches(x) => diecast::get_item(url, document),
        x if vortex_theatre::matches(x) => vortex_theatre::get_item(url, document),
        _ => Err(format_err!("Unknown podcast: {}", url)),
    }
}

fn get_item(url: &str) -> Result<Item> {
    // Get the html and build an Item.
    let response = reqwest::blocking::get(url)?;
    let body = response.text()?;
    let document = Document::from(body.as_str());

    process_document(url, &document)
}

pub fn handle(podcast: &str) {
    // Read podcast.urls and podcast.xml
    let urls = match get_urls(podcast) {
        Err(ref e) => {
            print_error(e);
            return;
        }
        Ok(urls) => urls,
    };

    let mut rss_data = match get_rss(podcast) {
        Err(ref e) => {
            print_error(e);
            return;
        }
        Ok(rss_data) => rss_data,
    };

    println!("{}: {}/{}", podcast, rss_data.items().len(), urls.len());
    let mut keys: Vec<String> = urls.keys().cloned().collect();
    keys.sort();
    keys.reverse();
    let mut items: Vec<_> = keys
        .par_iter()
        .map(|url| {
            if url.ends_with('*') {
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
                let item = urls[url].clone().ok_or(|| ()).or_else(|_| get_item(url));
                // println!("{}: {:?}, {:?}", url, urls[url], item);

                if let Err(ref e) = item {
                    // println!("Error in {}", url);
                    print_error(e);
                }
                item.ok()
            }
        })
        .filter_map(|x| x)
        .collect();
    // Write out the new podcast.xml
    items.sort_by(|a, b| {
        let a_date = DateTime::parse_from_rfc2822(a.pub_date().unwrap()).unwrap();
        let b_date = DateTime::parse_from_rfc2822(b.pub_date().unwrap()).unwrap();
        a_date.partial_cmp(&b_date).unwrap()
    });
    items.reverse();
    rss_data.set_items(items);
    let output = File::create(format!("{}.xml", podcast)).unwrap();
    rss_data.pretty_write_to(output, b' ', 2).unwrap();
}
