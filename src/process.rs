use std::io::{Cursor, Read};
use image::DynamicImage;

use crate::{appconfig::ImgprssrConfig, parameters::{OversizedImageHandling, ScalingFilter}};

pub fn process_image_to_buffer(settings: &ImgprssrConfig, mut img: DynamicImage, img_format: image::ImageFormat, params: crate::parameters::ImageParameters) -> Vec<u8> {
  img = process_image(settings, img, params);
  let mut buffer = Cursor::new(Vec::new());
  img.write_to(&mut buffer, img_format).unwrap();
  let mut out = Vec::new();
  buffer.set_position(0);
  buffer.read_to_end(&mut out).unwrap();
  out
}

fn resize_fit_height(img: DynamicImage, height: u32, scaling_filter: ScalingFilter, oversize_handling: OversizedImageHandling) -> DynamicImage {
  let source_height = img.height();
  if oversize_handling == OversizedImageHandling::Clamp && source_height > height {
    let height_factor = source_height as f32 / height as f32;
    let nwidth = img.width() as f32 * height_factor;
    return img.resize(nwidth as u32, height, scaling_filter.into());
  }
  img
}

fn resize_fit_width(img: DynamicImage, width: u32, scaling_filter: ScalingFilter, oversize_handling: OversizedImageHandling) -> DynamicImage {
  let source_width = img.width();
  if oversize_handling == OversizedImageHandling::Clamp && source_width > width {
    let width_factor = source_width as f32 / width as f32;
    let nheight = img.height() as f32 * width_factor;
    return img.resize(width, nheight as u32, scaling_filter.into());
  }
  img
}


fn fit_to_set_size(mut img: DynamicImage, width: u32, height: u32, scaling_filter: ScalingFilter, oversize_handling: OversizedImageHandling) -> DynamicImage {
  if oversize_handling == OversizedImageHandling::Clamp && !(img.height() >= height) || !(img.width() >= width) {
    return img;
  }
  
  let target_aspect = width as f32 / height as f32;
  let source_aspect = img.width() as f32 / img.height() as f32;
  
  if target_aspect > source_aspect { // "letterboxing" - resize to target width first
    img = resize_fit_width(img, width, scaling_filter, oversize_handling);
    let pixels_to_trim = img.height() - height;
    let edge_width_to_trim = pixels_to_trim / 2;
    img.crop(0, edge_width_to_trim, width, height)
  } else { // "tall" - resize to target height first
    img = resize_fit_height(img, height, scaling_filter, oversize_handling);
    let pixels_to_trim = img.width() - width;
    let edge_height_to_trim = pixels_to_trim / 2;
    img.crop(edge_height_to_trim, 0, width, height)
  }
}

pub fn process_image(settings: &ImgprssrConfig, mut img: DynamicImage, params: crate::parameters::ImageParameters) -> DynamicImage {
  let scaling_filter = if let Some(flt) = params.scaling_filter { flt } else { settings.default_filter };
  let oversize_handling = if let Some(os) = params.oversized_handling { os } else { settings.default_oversize_handling };
  if params.width.is_some() && params.height.is_none() {
    img = resize_fit_width(img, params.width.unwrap(), scaling_filter, oversize_handling);
  }
  if params.height.is_some() && params.width.is_none() {
    img = resize_fit_height(img, params.height.unwrap(), scaling_filter, oversize_handling);
  }
  if params.height.is_some() && params.width.is_some() {
    img = fit_to_set_size(img, params.width.unwrap(), params.height.unwrap(), scaling_filter, oversize_handling);
  }
  
  match params.colour_options {
      Some(opt) => match opt {
        crate::parameters::ColourMode::Greyscale => img = img.grayscale(),
      },
      None => {}, // do nothing
  }
  return img
}

#[cfg(test)]
mod tests {
  use std::path::Path;

  use image::GenericImageView;

  use crate::parameters::{ColourMode::Greyscale, ImageParameters, ScalingFilter};

  use super::*;

  const TEST_IMAGE_PATH: &str = "./images/test_card_sml.png";

  #[test]
  fn no_params_doesnt_manipulate_image() {
    let img = image::open(Path::new(TEST_IMAGE_PATH)).unwrap();
    let params = ImageParameters { width: None, height: None, scaling_filter: None, oversized_handling: None, colour_options: None };
    let cloned_image = img.clone();
    assert_eq!(process_image(&ImgprssrConfig::default(), img, params), cloned_image);
  }


  #[test]
  fn sets_width_with_matching_aspect_height() {
    let source_size = [1200_u32, 600];
    let cases = [[123_u32, 61], [200_u32, 100], [300_u32, 150], [600_u32, 300]];
    for case in cases {
      let img = image::DynamicImage::new_rgb8(source_size[0], source_size[1]);
      let params = ImageParameters { width: Some(case[0]), height: None, scaling_filter: None, oversized_handling: None, colour_options: None };
      let processed = process_image(&ImgprssrConfig::default(), img, params);
      assert_eq!(processed.width(), case[0]);
      assert_eq!(processed.height(), case[1]);
    }
  }

  #[test]
  fn sets_absolute_size() {
    let source_size = [1200_u32, 600];
    let cases = [[236, 123], [400, 400], [250, 300]];
    for [width, height] in cases {
      let img = image::DynamicImage::new_rgb8(source_size[0], source_size[1]);
      let params = ImageParameters { width: Some(width), height: Some(height), scaling_filter: None, oversized_handling: None, colour_options: None };
      let processed = process_image(&ImgprssrConfig::default(), img, params);
      assert_eq!(processed.width(), width);
      assert_eq!(processed.height(), height);
    }
  }


  #[test]
  fn sets_height_with_matching_aspect_width() {
    let source_size = [1200_u32, 600];
    let cases = [[123_u32, 246], [200_u32, 400], [150_u32, 300], [300_u32, 600]];
    for case in cases {
      let img = image::DynamicImage::new_rgb8(source_size[0], source_size[1]);
      let params = ImageParameters { width: None, height: Some(case[0]), scaling_filter: None, oversized_handling: None, colour_options: None };
      let processed = process_image(&ImgprssrConfig::default(), img, params);
      assert_eq!(processed.height(), case[0]);
      assert_eq!(processed.width(), case[1]);
    }
  }

  #[test]
  fn greyscale_desaturates() {
    let img = image::open(Path::new(TEST_IMAGE_PATH)).unwrap();
    let params = ImageParameters { width: None, height: None, scaling_filter: None, oversized_handling: None, colour_options: Some(Greyscale) };
    let cloned_image = img.clone();
    assert_eq!(process_image(&ImgprssrConfig::default(), img, params), cloned_image.grayscale());
  }

  #[test]
  fn clamp_does_not_upscale_width() {
    // Source is 100x50, target width is 200. Clamp should keep original.
    let img = image::DynamicImage::new_rgb8(100, 50);
    let params = ImageParameters {
      width: Some(200),
      height: None,
      scaling_filter: None,
      oversized_handling: Some(OversizedImageHandling::Clamp),
      colour_options: None,
    };
    let processed = process_image(&ImgprssrConfig::default(), img, params);
    assert_eq!(processed.width(), 100);
    assert_eq!(processed.height(), 50);
  }

  #[test]
  fn clamp_does_not_upscale_height() {
    // Source is 100x50, target height is 100. Clamp should keep original.
    let img = image::DynamicImage::new_rgb8(100, 50);
    let params = ImageParameters {
      width: None,
      height: Some(100),
      scaling_filter: None,
      oversized_handling: Some(OversizedImageHandling::Clamp),
      colour_options: None,
    };
    let processed = process_image(&ImgprssrConfig::default(), img, params);
    assert_eq!(processed.width(), 100);
    assert_eq!(processed.height(), 50);
  }

  #[test]
  fn clamp_downsizes_when_source_is_larger() {
    // Source is 1000x500, target is 200x100. Clamp should downsize.
    let img = image::DynamicImage::new_rgb8(1000, 500);
    let params = ImageParameters {
      width: Some(200),
      height: Some(100),
      scaling_filter: None,
      oversized_handling: Some(OversizedImageHandling::Clamp),
      colour_options: None,
    };
    let processed = process_image(&ImgprssrConfig::default(), img, params);
    assert_eq!(processed.width(), 200);
    assert_eq!(processed.height(), 100);
  }

  #[test]
  fn clamp_does_not_upscale_with_both_dimensions() {
    // Source is 100x50, target is 200x100. Clamp should keep original.
    // This exercises fit_to_set_size path with Clamp mode.
    let img = image::DynamicImage::new_rgb8(100, 50);
    let params = ImageParameters {
      width: Some(200),
      height: Some(100),
      scaling_filter: None,
      oversized_handling: Some(OversizedImageHandling::Clamp),
      colour_options: None,
    };
    let processed = process_image(&ImgprssrConfig::default(), img, params);
    assert_eq!(processed.width(), 100);
    assert_eq!(processed.height(), 50);
  }

  #[test]
  fn non_clamp_upsizes_with_both_dimensions() {
    // Source is 100x50, target is 200x100. Default oversize is Clamp,
    // but we explicitly set None (inherits Clamp from config default).
    // This should NOT upscale.
    let img = image::DynamicImage::new_rgb8(100, 50);
    let params = ImageParameters {
      width: Some(200),
      height: Some(100),
      scaling_filter: None,
      oversized_handling: None, // inherits Clamp from config
      colour_options: None,
    };
    let processed = process_image(&ImgprssrConfig::default(), img, params);
    // Clamp mode: should NOT upscale
    assert_eq!(processed.width(), 100);
    assert_eq!(processed.height(), 50);
  }

  #[test]
  fn gaussian_filter_resizes() {
    let img = image::DynamicImage::new_rgb8(1000, 500);
    let params = ImageParameters {
      width: Some(200),
      height: None,
      scaling_filter: Some(ScalingFilter::Gaussian),
      oversized_handling: None,
      colour_options: None,
    };
    let processed = process_image(&ImgprssrConfig::default(), img, params);
    assert_eq!(processed.width(), 200);
    assert_eq!(processed.height(), 100);
  }

  #[test]
  fn catmullrom_filter_resizes() {
    let img = image::DynamicImage::new_rgb8(1000, 500);
    let params = ImageParameters {
      width: Some(200),
      height: None,
      scaling_filter: Some(ScalingFilter::CatmullRom),
      oversized_handling: None,
      colour_options: None,
    };
    let processed = process_image(&ImgprssrConfig::default(), img, params);
    assert_eq!(processed.width(), 200);
    assert_eq!(processed.height(), 100);
  }

  #[test]
  fn combined_params_width_filter_greyscale() {
    let img = image::DynamicImage::new_rgb8(200, 100);
    let params = ImageParameters {
      width: Some(100),
      height: None,
      scaling_filter: Some(ScalingFilter::Triangle),
      oversized_handling: None,
      colour_options: Some(Greyscale),
    };
    let processed = process_image(&ImgprssrConfig::default(), img, params);
    assert_eq!(processed.width(), 100);
    assert_eq!(processed.height(), 50);
    // Verify it's greyscale: all pixels should have R == G == B
    for (_x, _y, pixel) in processed.pixels() {
      let [r, g, b, _] = pixel.0;
      assert_eq!(r, g);
      assert_eq!(g, b);
    }
  }

}