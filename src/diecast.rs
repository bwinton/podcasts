use crate::util::*;

use std::collections::HashMap;

use chrono::Duration;
use chrono::TimeZone;
use chrono::Utc;
use reqwest::header::CONTENT_LENGTH;
use rss::extension::dublincore::DublinCoreExtensionBuilder;
use rss::extension::itunes::ITunesItemExtensionBuilder;
use rss::Channel;
use rss::EnclosureBuilder;
use rss::GuidBuilder;
use rss::Item;
use rss::ItemBuilder;
use select::document::Document;
use select::node::Node;
use select::predicate::Attr;
use select::predicate::Class;
use select::predicate::Name;
use select::predicate::Or;
use select::predicate::Predicate;
use url::Url;

pub fn matches(url: &str) -> bool {
    url.starts_with("http://shamusyoung.com") || url.starts_with("https://www.shamusyoung.com")
}

pub fn get_urls(urls: &HashMap<String, Option<Item>>) -> Result<HashMap<String, Option<Item>>> {
    // Get the urls from https://www.shamusyoung.com/twentysidedtale/?cat=287&feed=rss2 !!!
    let response =
        reqwest::blocking::get("https://www.shamusyoung.com/twentysidedtale/?cat=287&feed=rss2")?;
    let body = response.text()?;
    let channel = Channel::read_from(body.as_bytes())?;
    let mut rv = HashMap::new();
    for item in channel.items() {
        if let Some(link) = item.link() {
            let link = link.to_owned();
            let missing_link = link.clone() + "*";
            if !urls.contains_key(&link) && !urls.contains_key(&missing_link) {
                rv.insert(link, None);
            }
        }
    }
    Ok(rv)
}

pub fn get_item(url: &str, document: &Document) -> Result<Item> {
    let this_document = Url::parse(url)?;
    let http_document =
        Url::parse(&url.replace("https://www.shamusyoung.com", "http://shamusyoung.com"))?;

    let title = document.find(Class("entry-title")).next().map(|x| x.text());

    let splash = document
        .find(Or(Class("splash-image"), Class("insetimage")))
        .next()
        .map_or(Some("images/splash_diecast2.jpg"), |x| x.attr("src"))
        .ok_or_else(|| format_err!("missing splash in {}", url))?;
    let splash = http_document.join(splash)?.to_string();

    let mut pub_date = document
        .find(Class("subhead-box"))
        .nth(1)
        .map(|x| x.text())
        .ok_or_else(|| format_err!("missing date box in {}", url))?;
    pub_date = pub_date
        .split("Posted ")
        .nth(1)
        .ok_or_else(|| format_err!("missing date in {}", url))?
        .to_string();
    pub_date.push_str(" 03:55:20");
    let pub_date = Utc.datetime_from_str(&pub_date, "%A %B %d, %Y %T")?;

    let dc = DublinCoreExtensionBuilder::default()
        .creators(vec!["Shamus Young".to_string()])
        .build()
        .map_err(|desc| format_err!("{}", desc))?;

    let guid = GuidBuilder::default()
        .permalink(false)
        .value(url.to_owned())
        .build()
        .map_err(|desc| format_err!("{}", desc))?;

    let mut description = Vec::new();
    let mut summary = Vec::new();
    if let Some(temp) = document.find(Class("entry-text")).next() {
        let children: Vec<_> = temp
            .children()
            .filter(|node| {
                node.name() != Some("div")
                    && node.find(Name("iframe")).next() == None
                    && node.find(Name("script")).next() == None
                    && node.find(Name("audio")).next() == None
            })
            .collect();
        description.extend(children.iter().map(Node::html));
        summary.extend(children.iter().map(|node| {
            let mut rv = node.text();
            if node.name() == Some("p") {
                rv.push_str("\n");
            }
            rv
        }));
    }
    let description = format_description(&description, &this_document);
    let summary = format_summary(&summary);

    let mp3 = document
        .find(Name("audio").child(Name("source").and(Attr("type", "audio/mpeg"))))
        .next()
        .and_then(|x| x.attr("src"))
        .ok_or_else(|| format_err!("missing mp3 link in {}", url))?;
    let mp3 = http_document.join(mp3)?;
    let mut response = reqwest::blocking::get(mp3.as_str())?;

    let temp = mp3_duration::from_read(&mut response)?;
    let duration = Duration::from_std(temp)?;
    let duration = format_duration(duration.num_seconds());

    let length = &response
        .headers()
        .get(CONTENT_LENGTH)
        .ok_or_else(|| format_err!("missing mp3 length for {}", mp3))?
        .to_str()?;

    let enclosure = EnclosureBuilder::default()
        .url(mp3.as_str())
        .length(*length)
        .mime_type("audio/mpeg".to_string())
        .build()
        .map_err(|desc| format_err!("{}", desc))?;

    let itunes = ITunesItemExtensionBuilder::default()
        .author(Some("The Diecast".to_string()))
        .summary(Some(summary))
        .explicit(Some("No".to_string()))
        .duration(duration)
        .image(splash)
        .build()
        .map_err(|desc| format_err!("{}", desc))?;

    ItemBuilder::default()
        .title(title)
        .dublin_core_ext(dc)
        .pub_date(pub_date.to_rfc2822().replace("  ", " "))
        .link(Some(url.to_owned()))
        .guid(guid)
        .description(Some(description))
        .itunes_ext(itunes)
        .enclosure(enclosure)
        .build()
        .map_err(|desc| format_err!("{}", desc))
}
