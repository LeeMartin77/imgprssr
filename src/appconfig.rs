use hyper::{Client, client::HttpConnector};
use hyper_tls::HttpsConnector;

use crate::parameters::{str_to_filter, OversizedImageHandling};

#[derive(Debug)]
#[derive(PartialEq)]
pub enum ImgprssrConfigErr {
  InvalidValues(Vec<String>)
}

#[derive(Debug)]
#[derive(Clone)]
pub enum ImgSource {
  Folder(String),
  Https((Client<HttpsConnector<HttpConnector>>, String)),
  Http((Client<HttpConnector>, String))
}

impl PartialEq for ImgSource {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Folder(l0), Self::Folder(r0)) => l0 == r0,
            (Self::Https((_, l0)), Self::Https((_, r0))) => l0 == r0,
            (Self::Http((_, l0)), Self::Http((_, r0))) => l0 == r0,
            _ => false,
        }
    }
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub struct ImgprssrConfig {
  pub default_filter: image::imageops::FilterType,
  pub default_oversize_handling: OversizedImageHandling,
  pub image_source: ImgSource
}

impl ImgprssrConfig {
  pub fn default() -> ImgprssrConfig {
    ImgprssrConfig {
      default_filter: image::imageops::FilterType::Nearest,
      default_oversize_handling: OversizedImageHandling::Clamp,
      image_source: ImgSource::Folder("./images".to_owned())
    }
  }
}

// Really need to look at how to do this with a trait
pub fn from_hashmap(hmp: std::collections::HashMap<String, String>) -> Result<ImgprssrConfig, ImgprssrConfigErr> {
  let mut config = ImgprssrConfig::default();
  let mut errors: Vec<String> = vec![];
  if let Some(val) = hmp.get("default_filter") {
    match str_to_filter(val) {
        Ok(fltr) => config.default_filter = fltr,
        Err(_) => errors.push(format!("default_filter::{val}")),
    }
  }
  if let Some(img_src) = hmp.get("image_source") {
    // TODO: We should actually validate these values
    if img_src.starts_with("https://") {
      let https = HttpsConnector::new();
      let client = Client::builder().build::<_, hyper::Body>(https);
      config.image_source = ImgSource::Https((client, img_src.to_owned()));
    } else if img_src.starts_with("http://") {
      config.image_source = ImgSource::Http((Client::new(), img_src.to_owned()));
    } else {
      config.image_source = ImgSource::Folder(img_src.to_owned());
    }
  }
  if errors.len() > 0 {
    return Err(ImgprssrConfigErr::InvalidValues(errors));
  }
  Ok(config)
}

#[cfg(test)]
mod tests {
  use std::collections::HashMap;

  use super::*;

  #[test]
  fn it_works_with_empty() {
    assert_eq!(from_hashmap(HashMap::new()), Ok(ImgprssrConfig::default()))
  }

  #[test]
  fn valid_string_parsed() {
    let mut hsmp = HashMap::new();
    let mut cnfg = ImgprssrConfig::default();
    hsmp.insert("default_filter".to_owned(), "gaussian".to_owned());
    cnfg.default_filter = image::imageops::FilterType::Gaussian;
    assert_eq!(from_hashmap(hsmp), Ok(cnfg))
  }

  #[test]
  fn invalid_filter_string_returns_err() {
    let mut hsmp = HashMap::new();
    hsmp.insert("default_filter".to_owned(), "not_real_filter".to_owned());
    assert_eq!(from_hashmap(hsmp), Err(
      ImgprssrConfigErr::InvalidValues(vec!["default_filter::not_real_filter".to_owned()])
    ))
  }


  #[test]
  fn valid_file_source_parsed() {
    let mut hsmp = HashMap::new();
    let mut cnfg = ImgprssrConfig::default();
    hsmp.insert("image_source".to_owned(), "./images".to_owned());
    cnfg.image_source = ImgSource::Folder("./images".to_owned());
    assert_eq!(from_hashmap(hsmp), Ok(cnfg))
  }

  #[test]
  fn valid_https_source_parsed() {
    let mut hsmp = HashMap::new();
    let mut cnfg = ImgprssrConfig::default();
    hsmp.insert("image_source".to_owned(), "https://example.com".to_owned());
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    cnfg.image_source = ImgSource::Https((client, "https://example.com".to_owned()));
    assert_eq!(from_hashmap(hsmp), Ok(cnfg))
  }

  #[test]
  fn valid_http_source_parsed() {
    let mut hsmp = HashMap::new();
    let mut cnfg = ImgprssrConfig::default();
    hsmp.insert("image_source".to_owned(), "http://example.com".to_owned());
    let client = Client::new();
    cnfg.image_source = ImgSource::Http((client, "http://example.com".to_owned()));
    assert_eq!(from_hashmap(hsmp), Ok(cnfg))
  }
}