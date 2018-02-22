extern crate rayon;
extern crate reqwest;
extern crate rss;
extern crate select;

use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

use rayon::prelude::*;
use rss::Channel;
use rss::EnclosureBuilder;
use rss::extension::dublincore::DublinCoreExtensionBuilder;
use rss::extension::itunes::ITunesItemExtensionBuilder;
use rss::GuidBuilder;
use rss::Item;
use rss::ItemBuilder;
use select::document::Document;
// use select::predicate::{Predicate, Attr, Class, Name};
use select::predicate::Class;
use select::predicate::Predicate;

fn get_urls(podcast: &str) -> Vec<String> {
  let urls = File::open(format!("{}.urls", podcast)).unwrap();
  let mut buf_reader = BufReader::new(urls);
  let mut contents = String::new();
  buf_reader.read_to_string(&mut contents).unwrap();
  contents.lines().map(|x| x.to_owned()).collect()
}

fn get_rss(podcast: &str) -> Channel {
  let xml = File::open(format!("dist/{}.xml", podcast)).unwrap();
  Channel::read_from(BufReader::new(xml)).unwrap()
}

fn get_item(url: &str) -> Option<Item> {
  // Get the html and build an Item.
  if let Ok(mut response) = reqwest::get(url) {
    if let Ok(body) = response.text() {
      let document = Document::from(body.as_str());

      let title = document.find(Class("title").and(Class("single-title"))).next().map(|x| x.text());
      println!("title: {:?}", title);

      // <dc:creator>The Spodcast</dc:creator>
      let dc = DublinCoreExtensionBuilder::default()
        .creators(vec!["The Spodcast".to_string()])
        .build().ok();
      println!("dc: {:?}", dc);

      // <pubDate>Tue, 13 Feb 2018 11:24:53 +0000</pubDate>
      let pub_date = document.find(Class("post-date-ribbon")).next().map(|x| x.text());
      println!("date: {:?}", pub_date);
      // <link>http://spoilerwarning.net/index.php/2018/02/13/the-spodcast-24-the-infectious-madness-of-video-game-lore/</link>
      // <guid isPermaLink="false">http://spoilerwarning.net/index.php/2018/02/13/the-spodcast-24-the-infectious-madness-of-video-game-lore/</guid>
      let guid = GuidBuilder::default()
        .permalink(false)
        .value(url.to_owned())
        .build().ok();
      println!("guid: {:?}", guid);
      // <description><![CDATA[<div></div>]]></description>
      // <itunes:author>The Diecast</itunes:author>
      // <itunes:summary>...</itunes:summary>
      // <itunes:explicit>no</itunes:explicit>
      // <itunes:duration>1:01:19</itunes:duration>
      // <itunes:image href="https://bwinton.github.io/podcasts/spodcast/title.png"/>
      let itunes = ITunesItemExtensionBuilder::default()
        .author(Some("The Spodcast".to_string()))
        // .summary?
        .explicit(Some("No".to_string()))
        // .duration
        .image(Some("https://bwinton.github.io/podcasts/spodcast/title.png".to_string()))
        .build().ok();
      println!("itunes: {:?}", itunes);
      // <enclosure url="http://spoilerwarning.net/spodcast/spodcast24.mp3" length="58866796" type="audio/mpeg"/>
      let enclosure = EnclosureBuilder::default()
        // .url()
        // .length()
        .mime_type("audio/mpeg".to_string())
        .build().ok();
      println!("enclosure: {:?}", enclosure);

      ItemBuilder::default()
        .title(title)
        .pub_date(pub_date)
        .dublin_core_ext(dc)
        .link(Some(url.to_owned()))
        .guid(guid)
        .itunes_ext(itunes)
        .enclosure(enclosure)
        .build().ok()
    } else { None }
  } else { None }
}

fn handle(podcast: &str) {
  // Read podcast.urls and dist/podcast.xml
  let urls = get_urls(podcast);
  let mut rss_data = get_rss(podcast);
  println!("{}: {}/{}", podcast, rss_data.items().len(), urls.len());
  let items: Vec<_> = urls.par_iter().map(|url| {
    if let Some(found) = rss_data.items().iter().find(|item| item.link() == Some(url)) {
      // Some(found.clone())
      None
    } else {
      // Find any missing urls.
      let item = get_item(url);
      // println!("Missing {:?}", item);
      item
    }
  }).filter_map(|x| x).collect();
  // Write out the new dist/podcast.xml
  rss_data.set_items(items);
  println!("{}", rss_data.to_string());
  // for item in rss_data.items() {
  //   if item.description().unwrap_or("").len() > 4000 {
  //     println!("{} {}", item.link().unwrap(), item.description().unwrap_or("").len());
  //   }
  // }
}

fn main() {
  // let podcasts = vec!["spodcast", "diecast"];
  let podcasts = vec!["spodcast"];
  // For podcast in spodcast/diecast
  podcasts.par_iter().for_each(|podcast| handle(podcast));
}
