use std::io::{Cursor, Read};
use image::DynamicImage;

pub fn process_image_to_buffer(mut img: DynamicImage, img_format: image::ImageFormat, params: crate::parameters::ImageParameters) -> Vec<u8> {
  img = process_image(img, params);
  let mut buffer = Cursor::new(Vec::new());
  img.write_to(&mut buffer, img_format).unwrap();
  let mut out = Vec::new();
  buffer.set_position(0);
  buffer.read_to_end(&mut out).unwrap();
  out
}

pub fn process_image(mut img: DynamicImage, params: crate::parameters::ImageParameters) -> DynamicImage {
  if let Some(w) = params.width {
      let source_width = img.width();
      if source_width != w {
          let width_factor = source_width as f32 / w as f32;
          let nheight = img.height() as f32 * width_factor;
          img = img.resize(w, nheight as u32, image::imageops::FilterType::Nearest);
      }
  }
  img
}


#[cfg(test)]
mod tests {
  use std::path::Path;

  use crate::parameters::ImageParameters;

  use super::*;

  const TEST_IMAGE_PATH: &str = "./images/test_card_sml.png";

  #[test]
  fn no_params_doesnt_manipulate_image() {
    let img = image::open(Path::new(TEST_IMAGE_PATH)).unwrap();
    let params = ImageParameters { width: None };
    let cloned_image = img.clone();
    assert_eq!(process_image(img, params), cloned_image);
  }


  #[test]
  fn sets_width_with_matching_aspect_height() {
    let source_size = [1200_u32, 600];
    let cases = [[200_u32, 100], [300_u32, 150], [600_u32, 300]];
    for case in cases {
      let img = image::DynamicImage::new_rgb8(source_size[0], source_size[1]);
      let params = ImageParameters { width: Some(case[0]) };
      let processed = process_image(img, params);
      assert_eq!(processed.width(), case[0]);
      assert_eq!(processed.height(), case[1]);
    }
  }
}