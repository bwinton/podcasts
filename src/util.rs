extern crate chrono;
extern crate regex;
extern crate reqwest;
extern crate time;
extern crate url;

use std::fmt::Write;

use failure::Error;
use regex::{Captures, Regex};
use url::Url;

pub type Result<T> = ::std::result::Result<T, Error>;

pub fn print_error(e: &Error) {
    let mut output = String::new();
    writeln!(output, "Error: {}", e).unwrap();
    for cause in e.causes().skip(1) {
        writeln!(output, "  caused by: {}", cause).unwrap();
    }

    writeln!(output, "{}", e.backtrace()).unwrap();
    print!("{}", output);
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
