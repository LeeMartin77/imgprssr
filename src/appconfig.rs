use std::net::IpAddr;

use hyper::Uri;
use hyper_util::client::legacy::{Client, connect::HttpConnector};
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
  Https((Client<HttpsConnector<HttpConnector>, String>, String)),
  Http((Client<HttpConnector, String>, String))
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

/// Check whether an IP is private / loopback / link-local (SSRF denylist).
fn is_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            o[0] == 10
            || o[0] == 172 && o[1] >= 16 && o[1] <= 31
            || o[0] == 192 && o[1] == 168
            || o[0] == 127
            || o[0] == 169 && o[1] == 254
            || o[0] == 0
            || o[0] == 100 && o[1] >= 64 && o[1] <= 127
        }
        IpAddr::V6(v6) => v6.is_loopback()
            // fc00::/7 — unique local (IPv6 private)
            || (v6.octets()[0] & 0xfe) == 0xfc
            // fe80::/10 — link-local
            || v6.is_unicast_link_local(),
    }
}

/// Validate that an HTTP(S) source URL does not point to a private IP.
fn validate_http_source(url: &str) -> Result<(), String> {
    use std::str::FromStr;
    let uri = Uri::from_str(url).map_err(|e| format!("Invalid URL: {e}"))?;
    if let Some(host) = uri.host() {
        if let Ok(ip) = host.parse::<IpAddr>() {
            if is_private_ip(ip) {
                return Err(format!(
                    "SSRF protection: '{host}' is a private IP address"
                ));
            }
        }
    }
    Ok(())
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
    if img_src.starts_with("https://") {
        if let Err(e) = validate_http_source(img_src) {
            errors.push(e);
        } else {
            let https = HttpsConnector::new();
            let client = Client::builder(hyper_util::rt::TokioExecutor::new())
                .build::<_, String>(https);
            config.image_source = ImgSource::Https((client, img_src.to_owned()));
        }
    } else if img_src.starts_with("http://") {
        if let Err(e) = validate_http_source(img_src) {
            errors.push(e);
        } else {
            let http = HttpConnector::new();
            let client = Client::builder(hyper_util::rt::TokioExecutor::new())
                .build(http);
            config.image_source = ImgSource::Http((client, img_src.to_owned()));
        }
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
    let client = Client::builder(hyper_util::rt::TokioExecutor::new())
        .build::<_, String>(https);
    cnfg.image_source = ImgSource::Https((client, "https://example.com".to_owned()));
    assert_eq!(from_hashmap(hsmp), Ok(cnfg))
  }

  #[test]
  fn valid_http_source_parsed() {
    let mut hsmp = HashMap::new();
    let mut cnfg = ImgprssrConfig::default();
    hsmp.insert("image_source".to_owned(), "http://example.com".to_owned());
    let http = HttpConnector::new();
    let client = Client::builder(hyper_util::rt::TokioExecutor::new())
        .build(http);
    cnfg.image_source = ImgSource::Http((client, "http://example.com".to_owned()));
    assert_eq!(from_hashmap(hsmp), Ok(cnfg))
  }

  #[test]
  fn ssrf_private_ip_https_rejected() {
    let mut hsmp = HashMap::new();
    hsmp.insert("image_source".to_owned(), "https://169.254.169.254/latest/meta-data/".to_owned());
    assert!(from_hashmap(hsmp).is_err());
  }

  #[test]
  fn ssrf_private_ip_http_rejected() {
    let mut hsmp = HashMap::new();
    hsmp.insert("image_source".to_owned(), "http://127.0.0.1:2379/v1.40/".to_owned());
    assert!(from_hashmap(hsmp).is_err());
  }

  #[test]
  fn ssrf_10_private_range_rejected() {
    let mut hsmp = HashMap::new();
    hsmp.insert("image_source".to_owned(), "http://10.0.0.5/".to_owned());
    assert!(from_hashmap(hsmp).is_err());
  }

  #[test]
  fn ssrf_public_host_allowed() {
    let mut hsmp = HashMap::new();
    hsmp.insert("image_source".to_owned(), "https://example.com/images".to_owned());
    let https = HttpsConnector::new();
    let client = Client::builder(hyper_util::rt::TokioExecutor::new())
        .build::<_, String>(https);
    let expected = ImgSource::Https((client, "https://example.com/images".to_owned()));
    assert_eq!(from_hashmap(hsmp).unwrap().image_source, expected);
  }
}
