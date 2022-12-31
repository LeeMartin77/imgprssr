use std::collections::HashMap;

use config::Config;

#[derive(Debug)]

pub enum ImgprssrConfigErr {
  InvalidValues(Vec<String>)
}

#[derive(Clone)]
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
fn from_hashmap(_hmp: std::collections::HashMap<String, String>) -> Result<ImgprssrConfig, ImgprssrConfigErr> {
  let config = ImgprssrConfig::default();
  Ok(config)
}