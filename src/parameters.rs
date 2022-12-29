use std::{str::FromStr, collections::HashMap};

pub const DEFAULT_FILTER: image::imageops::FilterType = image::imageops::FilterType::Nearest;

#[derive(Debug)]
#[derive(PartialEq)]
pub enum ImageParameterParseError {
  WidthParseError,
  FilterParseError
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

      let mut width = None;
      if let Some(num_string) = params.get("width") {
        if let Ok(num) = num_string.parse::<u32>() {
          width = Some(num);
        } else {
          return Err(ImageParameterParseError::WidthParseError);
        }
      }

      if let Some(filter_string) = params.get("filter") {
        match filter_string {
            &"nearest" => filter = image::imageops::FilterType::Nearest,
            &"gaussian" => filter = image::imageops::FilterType::Gaussian,
            &"catmullrom" => filter = image::imageops::FilterType::CatmullRom,
            &"lanczos3" => filter = image::imageops::FilterType::Lanczos3,
            &"triangle" => filter = image::imageops::FilterType::Triangle,
            _ => return Err(ImageParameterParseError::FilterParseError)
        }
      }

      Ok(ImageParameters { 
        width: width, 
        scaling_filter: filter 
      })
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
  fn parses_filters() {
    let cases = [
      (image::imageops::FilterType::Nearest, "nearest"),
      (image::imageops::FilterType::Gaussian, "gaussian"),
      (image::imageops::FilterType::CatmullRom, "catmullrom"),
      (image::imageops::FilterType::Lanczos3, "lanczos3"),
      (image::imageops::FilterType::Triangle, "triangle")
    ];
    for (filter_type, filter_string) in cases {
      let test: ImageParameters = format!("filter={}", filter_string).parse().unwrap();
      assert_eq!(test, ImageParameters { 
        width: None, 
        scaling_filter: filter_type
      });
    }
  }


  #[test]
  fn non_existent_filter_returns_error() {
    let test: Result<ImageParameters, ImageParameterParseError> = "filter=notreal".parse();
    assert_eq!(test, Err(ImageParameterParseError::FilterParseError));
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