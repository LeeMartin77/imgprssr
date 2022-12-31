use std::collections::HashMap;

use config::Config;

use crate::parameters::str_to_filter;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum ImgprssrConfigErr {
  InvalidValues(Vec<String>)
}

#[derive(Clone)]
#[derive(Debug)]
#[derive(PartialEq)]
pub struct ImgprssrConfig {
  pub default_filter: image::imageops::FilterType
}

impl ImgprssrConfig {
  pub fn default() -> ImgprssrConfig {
    ImgprssrConfig {
      default_filter: image::imageops::FilterType::Nearest
    }
  }
}

pub fn generate_app_config() -> Result<ImgprssrConfig, ImgprssrConfigErr> {
  from_hashmap(Config::builder()
        // ENV Variables are IMGPRSSR_SOMETHING == something
        .add_source(config::Environment::with_prefix("IMGPRSSR"))
        .build()
        .unwrap()
        .try_deserialize::<HashMap<String, String>>()
        .unwrap())
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
  if errors.len() > 0 {
    return Err(ImgprssrConfigErr::InvalidValues(errors));
  }
  Ok(config)
}

#[cfg(test)]
mod tests {
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
}