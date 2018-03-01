use util::*;

use chrono::Duration;
use chrono::TimeZone;
use chrono::Utc;
use mp3_duration;
use reqwest;
use reqwest::header::ContentLength;
use rss::EnclosureBuilder;
use rss::extension::dublincore::DublinCoreExtensionBuilder;
use rss::extension::itunes::ITunesItemExtensionBuilder;
use rss::GuidBuilder;
use rss::Item;
use rss::ItemBuilder;
use select::document::Document;
use select::predicate::Attr;
use select::predicate::Class;
use select::predicate::Name;
use select::predicate::Predicate;
use url::Url;

pub fn matches(url: &str) -> bool {
    url.starts_with("http://shamusyoung.com")
}

pub fn get_info(url: &str, document: &Document) -> Result<Item, PodcastError> {
    let this_document = Url::parse(url)?;

    let title = document
        .find(Class("splash-title"))
        .next()
        .map(|x| x.text());
    let mut date_str = document
        .find(Class("splash-avatar"))
        .next()
        .map(|x| x.text())
        .ok_or_else(|| PodcastError::new("missing avatar"))?;
    date_str = date_str
        .split("on ")
        .nth(1)
        .ok_or_else(|| PodcastError::new("missing date"))?
        .to_string();
    date_str.push_str(" 03:55:20");
    let pub_date = Utc.datetime_from_str(&date_str, "%A %B %d, %Y %T")?;

    let dc = DublinCoreExtensionBuilder::default()
        .creators(vec!["Shamus Young".to_string()])
        .build()?;

    let guid = GuidBuilder::default()
        .permalink(false)
        .value(url.to_owned())
        .build()?;

    let mut description = Vec::new();
    let mut summary = Vec::new();
    if let Some(temp) = document.find(Class("entry-text")).next() {
        let children: Vec<_> = temp.children()
            .filter(|node| {
                node.name() != Some("div") && node.find(Name("iframe")).next() == None
                    && node.find(Name("script")).next() == None
                    && node.find(Name("audio")).next() == None
            })
            .collect();
        description.extend(children.iter().map(|node| node.html()));
        summary.extend(children.iter().map(|node| {
            let mut rv = node.text();
            if node.name() == Some("p") {
                rv.push_str("\n");
            }
            rv
        }));
    }
    let description_string = format_description(&description, &this_document);
    let summary_string = format_summary(&summary);

    let mp3_link = document
        .find(Name("audio").child(Name("source").and(Attr("type", "audio/mpeg"))))
        .next()
        .and_then(|x| x.attr("src"))
        .ok_or_else(|| PodcastError::new("missing mp3 link"))?;
    let mp3 = this_document.join(mp3_link)?;
    let mut response = reqwest::get(mp3.as_str())?;
    let length = response
        .headers()
        .get::<ContentLength>()
        .map(|ct_len| **ct_len)
        .ok_or_else(|| PodcastError::new("missing mp3 length"))?
        .to_string();
    let temp = mp3_duration::from_read(&mut response)?;
    let duration = Duration::from_std(temp)?;
    let duration_string = format_duration(duration.num_seconds());

    let enclosure = EnclosureBuilder::default()
        .url(mp3.as_str())
        .length(length)
        .mime_type("audio/mpeg".to_string())
        .build()?;

    let itunes = ITunesItemExtensionBuilder::default()
        .author(Some("The Diecast".to_string()))
        .summary(Some(summary_string))
        .explicit(Some("No".to_string()))
        .duration(duration_string)
        .image(Some(
            "http://www.shamusyoung.com/twentysidedtale/images/splash_diecast2.jpg".to_string(),
        ))
        .build()?;

    ItemBuilder::default()
        .title(title)
        .dublin_core_ext(dc)
        .pub_date(pub_date.to_rfc2822().replace("  ", " "))
        .link(Some(url.to_owned()))
        .guid(guid)
        .description(Some(description_string))
        .itunes_ext(itunes)
        .enclosure(enclosure)
        .build()
        .map_err(|desc| PodcastError::new(&desc))
}
