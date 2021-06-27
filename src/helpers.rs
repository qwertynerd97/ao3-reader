use std::io;
use std::char;
use std::borrow::Cow;
use std::time::SystemTime;
use std::fs::{self, File, Metadata};
use std::path::{Path, PathBuf, Component};
use fxhash::FxHashMap;
use serde::{Serialize, Deserialize};
use lazy_static::lazy_static;
use entities::ENTITIES;
use walkdir::DirEntry;
use anyhow::{Error, Context};
use url::{Url, ParseError};

lazy_static! {
    pub static ref CHARACTER_ENTITIES: FxHashMap<&'static str, &'static str> = {
        let mut m = FxHashMap::default();
        for e in ENTITIES.iter() {
            m.insert(e.entity, e.characters);
        }
        m
    };
}

pub fn ceil(x: usize, y: usize) -> usize {
    x / y  + (x%y != 0) as usize
}

pub fn get_url(url: &str) -> Url {
    let ao3 = Url::parse("https://archiveofourown.org").unwrap();
    let parsed = Url::parse(url);
    match parsed {
        Ok(unwrapped_url) => unwrapped_url,
        Err(e) => if e == ParseError::RelativeUrlWithoutBase { ao3.join(url).unwrap()} else {ao3}
    }

}

pub fn url_strip_page(url: &mut Url) {
    let mut params = FxHashMap::default();
    for (key, value) in url.query_pairs() {
        if key == Cow::Borrowed("page") {
            continue;
        }
        params.insert(key.into_owned(), value.into_owned());
    }

    url.query_pairs_mut().clear().extend_pairs(params.drain());
}


pub fn update_url(url: &mut Url, new_params: Vec<(&str, &str)>) {
    let mut params = FxHashMap::default();
    for (key, value) in url.query_pairs() {
        params.insert(key.into_owned(), value.into_owned());
    }

    for (key, value) in new_params.iter() {
        params.insert(key.to_string(), value.to_string());
    }

    url.query_pairs_mut().clear().extend_pairs(params.drain());
}

pub fn decode_entities(text: &str) -> Cow<str> {
    if text.find('&').is_none() {
        return Cow::Borrowed(text);
    }

    let mut cursor = text;
    let mut buf = String::with_capacity(text.len());

    while let Some(start_index) = cursor.find('&') {
        buf.push_str(&cursor[..start_index]);
        cursor = &cursor[start_index..];
        if let Some(end_index) = cursor.find(';') {
            if let Some(repl) = CHARACTER_ENTITIES.get(&cursor[..=end_index]) {
                buf.push_str(repl);
            } else if cursor[1..].starts_with('#') {
                let radix = if cursor[2..].starts_with('x') {
                    16
                } else {
                    10
                };
                let drift_index = 2 + radix as usize / 16;
                if let Some(ch) = u32::from_str_radix(&cursor[drift_index..end_index], radix)
                                      .ok().and_then(char::from_u32) {
                    buf.push(ch);
                } else {
                    buf.push_str(&cursor[..=end_index]);
                }
            } else {
                buf.push_str(&cursor[..=end_index]);
            }
            cursor = &cursor[end_index+1..];
        } else {
            break;
        }
    }

    buf.push_str(cursor);
    Cow::Owned(buf)
}

pub fn load_json<T, P: AsRef<Path>>(path: P) -> Result<T, Error> where for<'a> T: Deserialize<'a> {
    let file = File::open(path.as_ref())
                    .with_context(|| format!("Cannot open file {}.", path.as_ref().display()))?;
    serde_json::from_reader(file)
               .with_context(|| format!("Cannot parse JSON from {}.", path.as_ref().display()))
               .map_err(Into::into)
}

pub fn save_json<T, P: AsRef<Path>>(data: &T, path: P) -> Result<(), Error> where T: Serialize {
    let file = File::create(path.as_ref())
                    .with_context(|| format!("Cannot create file {}.", path.as_ref().display()))?;
    serde_json::to_writer_pretty(file, data)
               .with_context(|| format!("Cannot serialize to JSON file {}.", path.as_ref().display()))
               .map_err(Into::into)
}

pub fn load_toml<T, P: AsRef<Path>>(path: P) -> Result<T, Error> where for<'a> T: Deserialize<'a> {
    let s = fs::read_to_string(path.as_ref())
               .with_context(|| format!("Cannot read file {}.", path.as_ref().display()))?;
    toml::from_str(&s)
         .with_context(|| format!("Cannot parse TOML content from {}.", path.as_ref().display()))
         .map_err(Into::into)
}

pub fn save_toml<T, P: AsRef<Path>>(data: &T, path: P) -> Result<(), Error> where T: Serialize {
    let s = toml::to_string(data)
                 .context("Cannot convert to TOML format.")?;
    fs::write(path.as_ref(), &s)
       .with_context(|| format!("Cannot write to file {}.", path.as_ref().display()))
       .map_err(Into::into)
}

pub trait Fingerprint {
    fn fingerprint(&self, epoch: SystemTime) -> io::Result<u64>;
}

impl Fingerprint for Metadata {
    fn fingerprint(&self, epoch: SystemTime) -> io::Result<u64> {
        let m = self.modified()?.duration_since(epoch)
                    .map_or_else(|e| e.duration().as_secs(), |v| v.as_secs());
        Ok(m.rotate_left(32) ^ self.len())
    }
}

pub trait Normalize: ToOwned {
    fn normalize(&self) -> Self::Owned;
}

impl Normalize for Path {
    fn normalize(&self) -> PathBuf {
        let mut result = PathBuf::from("");

        for c in self.components() {
            match c {
                Component::ParentDir => { result.pop(); },
                Component::CurDir => (),
                _ => result.push(c),
            }
        }

        result
    }
}

pub trait AsciiExtension {
    fn to_alphabetic_digit(self) -> Option<u32>;
}

impl AsciiExtension for char {
    fn to_alphabetic_digit(self) -> Option<u32> {
        if self.is_ascii_uppercase() {
            Some(self as u32 - 65)
        } else {
            None
        }
    }
}

pub mod datetime_format {
    use chrono::{DateTime, Local, TimeZone};
    use serde::{self, Deserialize, Serializer, Deserializer};

    pub const FORMAT: &str = "%Y-%m-%d %H:%M:%S";

    pub fn serialize<S>(date: &DateTime<Local>, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error> where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        Local.datetime_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

pub mod date_format {
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Serializer, Deserializer};

    pub const FORMAT: &str = "%Y-%m-%d";

    pub fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error> where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

pub trait IsHidden {
    fn is_hidden(&self) -> bool;
}

impl IsHidden for DirEntry {
    fn is_hidden(&self) -> bool {
        self.file_name()
             .to_str()
             .map_or(false, |s| s.starts_with('.'))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entities() {
        assert_eq!(decode_entities("a &amp b"), "a &amp b");
        assert_eq!(decode_entities("a &zZz; b"), "a &zZz; b");
        assert_eq!(decode_entities("a &amp; b"), "a & b");
        assert_eq!(decode_entities("a &#x003E; b"), "a > b");
        assert_eq!(decode_entities("a &#38; b"), "a & b");
        assert_eq!(decode_entities("a &lt; b &gt; c"), "a < b > c");
    }
}
