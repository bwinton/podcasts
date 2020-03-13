use crate::util::*;

use std::collections::HashMap;

use chrono::Utc;
use reqwest::header::CONTENT_LENGTH;
use rss::extension::dublincore::DublinCoreExtensionBuilder;
use rss::extension::itunes::ITunesItemExtensionBuilder;
use rss::EnclosureBuilder;
use rss::GuidBuilder;
use rss::Item;
use rss::ItemBuilder;
use select::document::Document;
use select::predicate::And;
use select::predicate::Class;
use select::predicate::Name;
use serde_json::Value;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

pub fn matches(url: &str) -> bool {
    url.starts_with("http://spiritlive.ca/")
}

pub fn get_urls(_: &HashMap<String, Option<Item>>) -> Result<HashMap<String, Option<Item>>> {
    // Get the urls and items from https://spiritlive.ca/vortex-theatre-2/ !!!
    let client = reqwest::blocking::Client::builder()
        .user_agent(APP_USER_AGENT)
        .build()?;
    let response = client
        .get("https://spiritlive.ca/vortex-theatre-2/")
        .send()?;
    let body = response.text()?;
    let document = Document::from(body.as_str());
    let script = document
        .find(And(Name("script"), Class("cue-playlist-data")))
        .next()
        .map(|x| x.text())
        .ok_or_else(|| format_err!("missing script in https://spiritlive.ca/vortex-theatre-2/"))?;
    let data: Value = serde_json::from_str(&script)?;
    let mut rv: HashMap<String, Option<Item>> = HashMap::new();
    for audio in data["tracks"].as_array().unwrap() {
        let title = audio["title"].as_str().unwrap().to_owned();

        let dc = DublinCoreExtensionBuilder::default()
            .creators(vec!["Paisley Sears and Shiann Nias".to_string()])
            .build()
            .map_err(|desc| format_err!("{}", desc))?;

        let pub_date = Utc::now();

        let url = audio["audioUrl"].as_str().unwrap();

        let guid = GuidBuilder::default()
            .permalink(false)
            .value(url)
            .build()
            .map_err(|desc| format_err!("{}", desc))?;

        let description = title.clone();
        let summary = title.clone();

        let itunes = ITunesItemExtensionBuilder::default()
            .author(Some("Paisley Sears and Shiann Nias".to_string()))
            .summary(Some(summary))
            .explicit(Some("No".to_string()))
            .duration(Some(audio["length"].as_str().unwrap().to_owned()))
            .image(Some(data["thumbnail"].as_str().unwrap().to_owned()))
            .build()
            .map_err(|desc| format_err!("{}", desc))?;

        let response = reqwest::blocking::get(url)?;
        let length = &response
            .headers()
            .get(CONTENT_LENGTH)
            .ok_or_else(|| format_err!("missing mp3 length for {}", url))?
            .to_str()?;

        let enclosure = EnclosureBuilder::default()
            .url(url)
            .length(*length)
            .mime_type("audio/mpeg".to_string())
            .build()
            .map_err(|desc| format_err!("{}", desc))?;

        let item = ItemBuilder::default()
            .title(Some(title))
            .dublin_core_ext(dc)
            .pub_date(pub_date.to_rfc2822().replace("  ", " "))
            .link(Some(url.to_owned()))
            .guid(guid)
            .description(Some(description))
            .itunes_ext(itunes)
            .enclosure(enclosure)
            .build()
            .map_err(|desc| format_err!("{}", desc))?;
        rv.insert(url.to_owned(), Some(item));
    }
    Ok(rv)
}

pub fn get_item(url: &str, _: &Document) -> Result<Item> {
    println!("Calling get_item for {}!!!!", url);
    Err(format_err!("Can not get item for urls of type {}", url))
}
