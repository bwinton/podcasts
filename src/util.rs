extern crate chrono;
extern crate failure;
extern crate regex;
extern crate reqwest;
extern crate time;
extern crate url;

use std::error::Error;
use std::io::Error as IOError;
use std::string::String;

use regex::Captures;
use regex::Regex;
use url::Url;

#[derive(Debug)]
#[derive(Default)]
pub struct PodcastError {
    description: String,
}

impl PodcastError {
    pub fn new(description: &str) -> Self {
        PodcastError {
            description: description.to_owned(),
        }
    }
}

impl From<reqwest::Error> for PodcastError {
    fn from(error: reqwest::Error) -> Self {
        println!("Reqwest Error! {:?}", error);
        PodcastError::new(&error.to_string())
    }
}

impl From<String> for PodcastError {
    fn from(error: String) -> Self {
        println!("String Error! {:?}", error);
        PodcastError::new(&error.to_string())
    }
}

impl From<IOError> for PodcastError {
    fn from(error: IOError) -> Self {
        println!("IO Error! {:?}", error);
        PodcastError::new(error.description())
    }
}

impl From<failure::Error> for PodcastError {
    fn from(error: failure::Error) -> Self {
        println!("Failure Error! {:?}", error);
        PodcastError::new("failure")
    }
}

impl From<time::OutOfRangeError> for PodcastError {
    fn from(error: time::OutOfRangeError) -> Self {
        println!("Time Error! {:?}", error);
        PodcastError::new(error.description())
    }
}

impl From<chrono::ParseError> for PodcastError {
    fn from(error: chrono::ParseError) -> Self {
        println!("Chrono Error! {:?}", error);
        PodcastError::new(error.description())
    }
}

impl From<url::ParseError> for PodcastError {
    fn from(error: url::ParseError) -> Self {
        println!("Url Error! {:?}", error);
        PodcastError::new(error.description())
    }
}

pub fn format_duration(duration: i64) -> String {
    let mut rv = vec![];
    let mut duration = duration;
    if duration > 3600 {
        rv.push(format!("{}", duration / 3600));
        duration -= duration / 3600 * 3600;
    }
    if rv.is_empty() {
        rv.push(format!("{}", duration / 60));
    } else {
        rv.push(format!("{:02}", duration / 60));
    }
    duration -= duration / 60 * 60;
    rv.push(format!("{:02}", duration));
    rv.join(":").to_string()
}

pub fn format_description(description: &[String], base: &Url) -> String {
    lazy_static! {
      static ref HREF_RE: Regex = Regex::new("(src|href)=\"([^\"]*)\"").unwrap();
      static ref LANG_RE: Regex = Regex::new("(srcset|download|data-[^=]*)=\"[^\"]*\"").unwrap();
      static ref GARBAGE_RE: Regex = Regex::new("(Â®| â€.| ðŸ¦)").unwrap();
    }

    let mut rv = HREF_RE
        .replace_all(&description.join(""), |caps: &Captures| {
            format!("{}=\"{}\"", &caps[1], base.join(&caps[2]).unwrap())
        })
        .to_string();
    rv = LANG_RE.replace_all(&rv, "").to_string();
    rv = GARBAGE_RE.replace_all(&rv, "").to_string();
    rv
}

pub fn format_summary(summary: &[String]) -> String {
    lazy_static! {
      static ref GARBAGE_RE: Regex = Regex::new("(Â®| â€.| ðŸ¦)").unwrap();
    }

    let mut rv = summary.join("");
    rv = GARBAGE_RE.replace_all(&rv, "").to_string();
    rv = rv.replace("'", "’");

    if let Some((idx, _)) = rv.char_indices().nth(3990) {
        rv = rv[..idx].to_string();
        rv.push_str("…");
    }
    rv
}
