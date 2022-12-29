use std::{str::FromStr, collections::HashMap};

pub const DEFAULT_FILTER: image::imageops::FilterType = image::imageops::FilterType::Nearest;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum ImageParameterParseError {
  WidthParseError
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct ImageParameters {
  pub width: Option<u32>,
  pub scaling_filter: image::imageops::FilterType
}

impl FromStr for ImageParameters {
    type Err = ImageParameterParseError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
      let query_parts = string.split("&");
      let mut params: HashMap<&str, &str> = HashMap::new();

      let mut filter = DEFAULT_FILTER;

      for q in query_parts {
          let mut prts = q.split("=").into_iter();
          let key = prts.next().unwrap();
          let val = prts.next().unwrap_or("true");
          params.insert(key, val);
      }
      let width = params.get("width");
      match width {
        Some(num_string) => {
          let num_parsed: Result<u32, _> = num_string.parse();
          match num_parsed {
            Ok(num) => Ok(ImageParameters { 
              width: Some(num), 
              scaling_filter: filter 
            }),
            Err(_) => Err(ImageParameterParseError::WidthParseError),
        }
        },
        None => Ok(ImageParameters { 
          width: None, 
          scaling_filter: filter 
        }),
      }
    }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_works() {
    let test: ImageParameters = "".parse().unwrap();
    assert_eq!(test, ImageParameters { 
      width: None, 
      scaling_filter: DEFAULT_FILTER
    });
  }

  #[test]
  fn parses_width() {
    let cases = [100_u32, 100, 200, 300, 400, 500, 600];
    for width in cases {
      let test: ImageParameters = format!("width={}", width).parse().unwrap();
      assert_eq!(test, ImageParameters { 
        width: Some(width), 
        scaling_filter: DEFAULT_FILTER
      });
    }
  }

  #[test]
  fn invalid_width_returns_err() {
    let cases = ["width=asdw", "width=300.1", "width=300px", "width=true", "width=", "width"];
    for case in cases {
      let test: Result<ImageParameters, ImageParameterParseError> = case.parse();
      assert_eq!(test.unwrap_err(), ImageParameterParseError::WidthParseError);
    }
  }
}