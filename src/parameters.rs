use std::{str::FromStr, collections::HashMap};

#[derive(Debug)]
#[derive(PartialEq)]
pub enum ImageParameterParseError {
  WidthParseError,
  HeightParseError,
  FilterParseError,
  OversizeParseError
}

#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone)]
#[derive(Copy)]
pub enum OversizedImageHandling {
  Clamp
}

impl FromStr for OversizedImageHandling {
    type Err = std::fmt::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "clamp" => Ok(OversizedImageHandling::Clamp),
            _ => Err(std::fmt::Error)
        }
    }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub struct ImageParameters {
  pub width: Option<u32>,
  pub height: Option<u32>,
  pub scaling_filter: Option<image::imageops::FilterType>,
  pub oversized_handling: Option<OversizedImageHandling>
}

pub fn str_to_filter(filter_string: &str) -> Result<image::imageops::FilterType, ImageParameterParseError> {
  match filter_string {
    "nearest" => Ok(image::imageops::FilterType::Nearest),
    "gaussian" => Ok(image::imageops::FilterType::Gaussian),
    "catmullrom" => Ok(image::imageops::FilterType::CatmullRom),
    "lanczos3" => Ok(image::imageops::FilterType::Lanczos3),
    "triangle" => Ok(image::imageops::FilterType::Triangle),
    _ => Err(ImageParameterParseError::FilterParseError)
}
}

impl FromStr for ImageParameters {
    type Err = ImageParameterParseError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
      let query_parts = string.split("&");
      let mut params: HashMap<&str, &str> = HashMap::new();

      let mut img_params = ImageParameters { 
        width: None, 
        height: None,
        scaling_filter: None,
        oversized_handling: None
      };

      for q in query_parts {
          let mut prts = q.split("=").into_iter();
          let key = prts.next().unwrap();
          let val = prts.next().unwrap_or("true");
          params.insert(key, val);
      }

      if let Some(num_string) = params.get("width") {
        if let Ok(num) = num_string.parse::<u32>() {
          img_params.width = Some(num);
        } else {
          return Err(ImageParameterParseError::WidthParseError);
        }
      }

      if let Some(num_string) = params.get("height") {
        if let Ok(num) = num_string.parse::<u32>() {
          img_params.height = Some(num);
        } else {
          return Err(ImageParameterParseError::HeightParseError);
        }
      }

      if let Some(stng) = params.get("oversizehandling") {
        if let Ok(val) = stng.parse::<OversizedImageHandling>() {
          img_params.oversized_handling = Some(val);
        } else {
          return Err(ImageParameterParseError::OversizeParseError);
        }
      }

      if let Some(filter_string) = params.get("filter") {
        match str_to_filter(filter_string) {
            Ok(flt) => img_params.scaling_filter = Some(flt),
            Err(err) => return Err(err),
        }
      }

      Ok(img_params)
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
      height: None,
      scaling_filter: None,
      oversized_handling: None
    });
  }

  #[test]
  fn parses_width() {
    let cases = [100_u32, 100, 200, 300, 400, 500, 600];
    for width in cases {
      let test: ImageParameters = format!("width={}", width).parse().unwrap();
      assert_eq!(test, ImageParameters { 
        width: Some(width), 
        height: None,
        scaling_filter: None,
        oversized_handling: None
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

  #[test]
  fn parses_height() {
    let cases = [100_u32, 100, 200, 300, 400, 500, 600];
    for height in cases {
      let test: ImageParameters = format!("height={}", height).parse().unwrap();
      assert_eq!(test, ImageParameters { 
        width: None,
        height: Some(height),
        scaling_filter: None,
        oversized_handling: None
      });
    }
  }

  #[test]
  fn invalid_height_returns_err() {
    let cases = ["height=asdw", "height=300.1", "height=300px", "height=true", "height=", "height"];
    for case in cases {
      let test: Result<ImageParameters, ImageParameterParseError> = case.parse();
      assert_eq!(test.unwrap_err(), ImageParameterParseError::HeightParseError);
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
        height: None,
        scaling_filter: Some(filter_type),
        oversized_handling: None
      });
    }
  }

  #[test]
  fn parses_oversize_handling() {
    let cases = [
      (OversizedImageHandling::Clamp, "clamp"),
    ];
    for (filter_type, filter_string) in cases {
      let test: ImageParameters = format!("oversizehandling={}", filter_string).parse().unwrap();
      assert_eq!(test, ImageParameters { 
        width: None, 
        height: None,
        scaling_filter: None,
        oversized_handling: Some(filter_type)
      });
    }
  }

  #[test]
  fn errors_oversize_handling() {
    let cases = [
      "akmdsas",
    ];
    for filter_string in cases {
      let test: Result<ImageParameters, ImageParameterParseError> = format!("oversizehandling={}", filter_string).parse();
      assert_eq!(test, Err(ImageParameterParseError::OversizeParseError));
    }
  }

  #[test]
  fn non_existent_filter_returns_error() {
    let test: Result<ImageParameters, ImageParameterParseError> = "filter=notreal".parse();
    assert_eq!(test, Err(ImageParameterParseError::FilterParseError));
  }
}