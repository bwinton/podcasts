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
use select::predicate::Or;
use select::predicate::Predicate;
use url::Url;

pub fn matches(url: &str) -> bool {
    url.starts_with("http://shamusyoung.com")
}

pub fn get_info(url: &str, document: &Document) -> Result<Item> {
    let this_document = Url::parse(url)?;

    let title = document
        .find(Class("entry-title"))
        .next()
        .map(|x| x.text());

    let splash = document
        .find(Or(Class("splash-image"), Class("insetimage")))
        .next()
        .map_or(Some("images/splash_diecast2.jpg"), |x| x.attr("src"))
        .ok_or_else(|| format_err!("missing splash in {}", url))?;
    let splash = this_document.join(splash)?.to_string();

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
        .build().map_err(|desc| format_err!("{}", desc))?;

    let guid = GuidBuilder::default()
        .permalink(false)
        .value(url.to_owned())
        .build().map_err(|desc| format_err!("{}", desc))?;

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
    let description = format_description(&description, &this_document);
    let summary = format_summary(&summary);

    let mp3 = document
        .find(Name("audio").child(Name("source").and(Attr("type", "audio/mpeg"))))
        .next()
        .and_then(|x| x.attr("src"))
        .ok_or_else(|| format_err!("missing mp3 link in {}", url))?;
    let mp3 = this_document.join(mp3)?;
    let mut response = reqwest::get(mp3.as_str())?;
    let length = response
        .headers()
        .get::<ContentLength>()
        .map(|ct_len| **ct_len)
        .ok_or_else(|| format_err!("missing mp3 length for {}", mp3))?
        .to_string();
    let temp = mp3_duration::from_read(&mut response)?;
    let duration = Duration::from_std(temp)?;
    let duration = format_duration(duration.num_seconds());

    let enclosure = EnclosureBuilder::default()
        .url(mp3.as_str())
        .length(length)
        .mime_type("audio/mpeg".to_string())
        .build().map_err(|desc| format_err!("{}", desc))?;

    let itunes = ITunesItemExtensionBuilder::default()
        .author(Some("The Diecast".to_string()))
        .summary(Some(summary))
        .explicit(Some("No".to_string()))
        .duration(duration)
        .image(splash)
        .build().map_err(|desc| format_err!("{}", desc))?;

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
