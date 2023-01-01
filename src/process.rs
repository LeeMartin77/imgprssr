use std::io::{Cursor, Read};
use image::DynamicImage;

use crate::{appconfig::ImgprssrConfig, parameters::OversizedImageHandling};

pub fn process_image_to_buffer(settings: &ImgprssrConfig, mut img: DynamicImage, img_format: image::ImageFormat, params: crate::parameters::ImageParameters) -> Vec<u8> {
  img = process_image(settings, img, params);
  let mut buffer = Cursor::new(Vec::new());
  img.write_to(&mut buffer, img_format).unwrap();
  let mut out = Vec::new();
  buffer.set_position(0);
  buffer.read_to_end(&mut out).unwrap();
  out
}

fn resize_single_edge(settings: &ImgprssrConfig, mut img: DynamicImage, params: &crate::parameters::ImageParameters) -> DynamicImage {
  let scaling_filter = if let Some(flt) = params.scaling_filter { flt } else { settings.default_filter };
  let oversize_handling = if let Some(os) = params.oversized_handling { os } else { settings.default_oversize_handling };
  if let Some(w) = params.width {
    let source_width = img.width();
    if params.height.is_none() && oversize_handling == OversizedImageHandling::Clamp && source_width > w {
      let width_factor = source_width as f32 / w as f32;
      let nheight = img.height() as f32 * width_factor;
      img = img.resize(w, nheight as u32, scaling_filter);
    }
  }
  if let Some(h) = params.height {
    let source_height = img.height();
    if params.width.is_none() && oversize_handling == OversizedImageHandling::Clamp && source_height > h {
      let height_factor = source_height as f32 / h as f32;
      let nwidth = img.width() as f32 * height_factor;
      img = img.resize(nwidth as u32, h, scaling_filter);
    }
  }
  img
}

pub fn process_image(settings: &ImgprssrConfig, mut img: DynamicImage, params: crate::parameters::ImageParameters) -> DynamicImage {
  if (params.height.is_some() || params.width.is_some()) && !(params.height.is_some() && params.width.is_some()) {
    img = resize_single_edge(settings, img, &params);
  }
  img
}


#[cfg(test)]
mod tests {
  use std::path::Path;

  use crate::parameters::{ImageParameters};

  use super::*;

  const TEST_IMAGE_PATH: &str = "./images/test_card_sml.png";

  #[test]
  fn no_params_doesnt_manipulate_image() {
    let img = image::open(Path::new(TEST_IMAGE_PATH)).unwrap();
    let params = ImageParameters { width: None, height: None, scaling_filter: None, oversized_handling: None };
    let cloned_image = img.clone();
    assert_eq!(process_image(&ImgprssrConfig::default(), img, params), cloned_image);
  }


  #[test]
  fn sets_width_with_matching_aspect_height() {
    let source_size = [1200_u32, 600];
    let cases = [[123_u32, 61], [200_u32, 100], [300_u32, 150], [600_u32, 300]];
    for case in cases {
      let img = image::DynamicImage::new_rgb8(source_size[0], source_size[1]);
      let params = ImageParameters { width: Some(case[0]), height: None, scaling_filter: None, oversized_handling: None };
      let processed = process_image(&ImgprssrConfig::default(), img, params);
      assert_eq!(processed.width(), case[0]);
      assert_eq!(processed.height(), case[1]);
    }
  }

  #[test]
  fn sets_height_with_matching_aspect_width() {
    let source_size = [1200_u32, 600];
    let cases = [[123_u32, 246], [200_u32, 400], [150_u32, 300], [300_u32, 600]];
    for case in cases {
      let img = image::DynamicImage::new_rgb8(source_size[0], source_size[1]);
      let params = ImageParameters { width: None, height: Some(case[0]), scaling_filter: None, oversized_handling: None };
      let processed = process_image(&ImgprssrConfig::default(), img, params);
      assert_eq!(processed.height(), case[0]);
      assert_eq!(processed.width(), case[1]);
    }
  }
}