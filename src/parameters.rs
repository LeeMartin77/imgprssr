use std::{str::FromStr, collections::HashMap};

#[derive(Debug)]
#[derive(PartialEq)]
pub enum ImageParameterParseError {
  WidthParseError,
  HeightParseError,
  FilterParseError,
  OversizeParseError,
  ColourParseError
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
#[derive(Clone)]
#[derive(Copy)]
pub enum ColourMode {
  Greyscale
}

impl FromStr for ColourMode {
    type Err = std::fmt::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "greyscale" => Ok(ColourMode::Greyscale),
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
  pub oversized_handling: Option<OversizedImageHandling>,
  pub colour_options: Option<ColourMode>
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
        oversized_handling: None,
        colour_options: None
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

      if let Some(stng) = params.get("colour") {
        if let Ok(val) = stng.parse::<ColourMode>() {
          img_params.colour_options = Some(val);
        } else {
          return Err(ImageParameterParseError::ColourParseError);
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
      oversized_handling: None,
      colour_options: None
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
        oversized_handling: None,
        colour_options: None
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
        oversized_handling: None,
        colour_options: None
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
        oversized_handling: None,
        colour_options: None
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
        oversized_handling: Some(filter_type),
        colour_options: None
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
  fn parses_colour_handling() {
    let cases = [
      (ColourMode::Greyscale, "greyscale"),
    ];
    for (colour_mode, filter_string) in cases {
      let test: ImageParameters = format!("colour={}", filter_string).parse().unwrap();
      assert_eq!(test, ImageParameters { 
        width: None, 
        height: None,
        scaling_filter: None,
        oversized_handling: None,
        colour_options: Some(colour_mode)
      });
    }
  }

  #[test]
  fn errors_colour_handling() {
    let cases = [
      "akmdsas",
    ];
    for filter_string in cases {
      let test: Result<ImageParameters, ImageParameterParseError> = format!("colour={}", filter_string).parse();
      assert_eq!(test, Err(ImageParameterParseError::ColourParseError));
    }
  }

  #[test]
  fn non_existent_filter_returns_error() {
    let test: Result<ImageParameters, ImageParameterParseError> = "filter=notreal".parse();
    assert_eq!(test, Err(ImageParameterParseError::FilterParseError));
  }

  // ---------------------------------------------------------------------------
  // Property-based tests (proptest)
  // ---------------------------------------------------------------------------

  mod proptest_tests {
    use proptest::prelude::*;

    use super::*;

    // Valid filter strings are always parsed successfully.
    proptest! {
      #[test]
      fn valid_filter_strings_parse(filter in "nearest|gaussian|catmullrom|lanczos3|triangle") {
        let result: Result<ImageParameters, _> = format!("filter={filter}").parse();
        assert!(result.is_ok());
      }

      // Invalid filter strings always fail with FilterParseError.
      #[test]
      fn invalid_filter_strings_fail(filter in "[^a-z]*") {
        let result: Result<ImageParameters, _> = format!("filter={filter}").parse();
        assert_eq!(result, Err(ImageParameterParseError::FilterParseError));
      }

      // Valid u32 widths parse successfully.
      #[test]
      fn valid_width_parses(width in 1u32..=u32::MAX) {
        let result: ImageParameters = format!("width={width}").parse().unwrap();
        assert_eq!(result.width, Some(width));
      }

      // Non-numeric width values always fail.
      #[test]
      fn non_numeric_width_fails(width in "[^0-9]+") {
        let result: Result<ImageParameters, _> = format!("width={width}").parse();
        assert_eq!(result, Err(ImageParameterParseError::WidthParseError));
      }

      // Non-numeric height values always fail.
      #[test]
      fn non_numeric_height_fails(height in "[^0-9]+") {
        let result: Result<ImageParameters, _> = format!("height={height}").parse();
        assert_eq!(result, Err(ImageParameterParseError::HeightParseError));
      }

      // Valid height values parse successfully.
      #[test]
      fn valid_height_parses(height in 1u32..=u32::MAX) {
        let result: ImageParameters = format!("height={height}").parse().unwrap();
        assert_eq!(result.height, Some(height));
      }

      // Empty query string yields all-None params.
      #[test]
      fn empty_string_parses_to_none(empty in "[a-z]{0}") {
        let result: ImageParameters = empty.parse().unwrap();
        assert!(result.width.is_none());
        assert!(result.height.is_none());
        assert!(result.scaling_filter.is_none());
        assert!(result.oversized_handling.is_none());
        assert!(result.colour_options.is_none());
      }

      // Combined width + height parses both.
      #[test]
      fn combined_width_and_height(width in 1u32..10000u32, height in 1u32..10000u32) {
        let result: ImageParameters = format!("width={width}&height={height}").parse().unwrap();
        assert_eq!(result.width, Some(width));
        assert_eq!(result.height, Some(height));
      }

      // Oversized handling accepts only "clamp".
      #[test]
      fn valid_oversize_parses(val in "clamp") {
        let result: ImageParameters = format!("oversizehandling={val}").parse().unwrap();
        assert_eq!(result.oversized_handling, Some(OversizedImageHandling::Clamp));
      }

      // Colour accepts only "greyscale".
      #[test]
      fn valid_colour_parses(val in "greyscale") {
        let result: ImageParameters = format!("colour={val}").parse().unwrap();
        assert_eq!(result.colour_options, Some(ColourMode::Greyscale));
      }

      // Boolean-like values for width/height fail.
      #[test]
      fn boolean_values_for_size_fail(val in "true|false|yes|no|on|off") {
        let w: Result<ImageParameters, _> = format!("width={val}").parse();
        let h: Result<ImageParameters, _> = format!("height={val}").parse();
        assert_eq!(w, Err(ImageParameterParseError::WidthParseError));
        assert_eq!(h, Err(ImageParameterParseError::HeightParseError));
      }

      // Negative numbers (parsed as large u32 overflow) fail.
      #[test]
      fn negative_numbers_fail(val in "-[0-9]+") {
        let w: Result<ImageParameters, _> = format!("width={val}").parse();
        let h: Result<ImageParameters, _> = format!("height={val}").parse();
        assert!(w.is_err() || w.as_ref().map(|p| p.width.is_none()).unwrap_or(false));
        assert!(h.is_err() || h.as_ref().map(|p| p.height.is_none()).unwrap_or(false));
      }
    }
  }
}