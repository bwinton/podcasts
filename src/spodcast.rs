use crate::util::*;

use chrono::Duration;
use chrono::TimeZone;
use chrono::Utc;
use reqwest::header::CONTENT_LENGTH;
use rss::extension::dublincore::DublinCoreExtensionBuilder;
use rss::extension::itunes::ITunesItemExtensionBuilder;
use rss::EnclosureBuilder;
use rss::GuidBuilder;
use rss::Item;
use rss::ItemBuilder;
use select::document::Document;
use select::predicate::Attr;
use select::predicate::Class;
use select::predicate::Name;
use select::predicate::Predicate;

pub fn matches(url: &str) -> bool {
    url.starts_with("http://spoilerwarning.net")
}

pub fn get_info(url: &str, document: &Document) -> Result<Item> {
    // Starts with http://spoilerwarning.net
    let title = document
        .find(Class("title").and(Class("single-title")))
        .next()
        .map(|x| x.text());
    let mut date_str = document
        .find(Class("post-date-ribbon"))
        .next()
        .map(|x| x.text())
        .ok_or_else(|| format_err!("missing date in {}", url))?;
    date_str.push_str(" 02:55:20");
    let pub_date = Utc.datetime_from_str(&date_str, "%B %d, %Y %T")?;

    let dc = DublinCoreExtensionBuilder::default()
        .creators(vec!["The Spodcast".to_string()])
        .build()
        .map_err(|desc| format_err!("{}", desc))?;

    let guid = GuidBuilder::default()
        .permalink(false)
        .value(url.to_owned())
        .build()
        .map_err(|desc| format_err!("{}", desc))?;

    let mut description = Vec::new();
    let mut summary = Vec::new();
    if let Some(temp) = document.find(Attr("id", "content")).next() {
        description.extend(
            temp.children()
                .skip(4)
                .filter(|node| node.name() == Some("p"))
                .map(|node| node.html()),
        );
        summary.extend(
            temp.children()
                .skip(4)
                .filter(|node| node.name() == Some("p"))
                .map(|node| node.text()),
        );
    }

    let mp3 = document
        .find(Name("audio").child(Name("source").and(Attr("type", "audio/mpeg"))))
        .next()
        .and_then(|x| x.attr("src"))
        .ok_or_else(|| format_err!("missing mp3 link in {}", url))?;
    let mut response = reqwest::blocking::get(mp3)?;
    let duration = Duration::from_std(mp3_duration::from_read(&mut response)?)?;
    let duration_string = format_duration(duration.num_seconds());

    let length = &response
        .headers()
        .get(CONTENT_LENGTH)
        .ok_or_else(|| format_err!("missing mp3 length for {}", mp3))?
        .to_str()?;

    let enclosure = EnclosureBuilder::default()
        .url(mp3)
        .length(*length)
        .mime_type("audio/mpeg".to_string())
        .build()
        .map_err(|desc| format_err!("{}", desc))?;

    let itunes = ITunesItemExtensionBuilder::default()
        .author(Some("The Spodcast".to_string()))
        .summary(Some(summary.join("\n\n")))
        .explicit(Some("No".to_string()))
        .duration(duration_string)
        .image(Some(
            "https://bwinton.github.io/podcasts/spodcast/title.png".to_string(),
        ))
        .build()
        .map_err(|desc| format_err!("{}", desc))?;

    ItemBuilder::default()
        .title(title)
        .dublin_core_ext(dc)
        .pub_date(pub_date.to_rfc2822().replace("  ", " "))
        .link(Some(url.to_owned()))
        .guid(guid)
        .description(description.join("\n"))
        .itunes_ext(itunes)
        .enclosure(enclosure)
        .build()
        .map_err(|desc| format_err!("{}", desc))
}
