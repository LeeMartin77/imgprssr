use std::path::Path;
use imgprssr_core;
use image::{self, DynamicImage, ImageError, ImageFormat};
use hyper::{Request, Body, Response, StatusCode};

pub fn source_image_from_file(img_source: &str, target_path: &str) -> Result<(DynamicImage, ImageFormat), ImageError> {
    let raw_path = format!("{}{}", img_source, target_path);
    let path = Path::new(&raw_path);
    match image::open(path) {
        Ok(img) => Ok((img, image::ImageFormat::from_path(path).unwrap())),
        Err(err) => Err(err),
    }
}

pub fn get_source_image(settings: &imgprssr_core::appconfig::ImgprssrConfig, req: Request<Body>) -> Result<(image::DynamicImage, image::ImageFormat, imgprssr_core::parameters::ImageParameters), Response<Body>> {
  let target_path = req.uri().path();
  let params_res: Result<imgprssr_core::parameters::ImageParameters, _> = req.uri().query().unwrap_or("").parse();
  if params_res.is_err() {
      return Err(Response::builder()
          .status(StatusCode::BAD_REQUEST)
          .body("Bad Request".into()).unwrap().into())
  }
  let params = params_res.unwrap();
  let img_res = source_image_from_file(&settings.image_source, target_path);
  match img_res {
      Ok((img, fmt)) => {
          Ok((img, fmt, params))
      },
      Err(_) => {
          Err(Response::builder()
              .status(StatusCode::NOT_FOUND)
              .body("Not Found".into()).unwrap().into())
      }
  }
}

#[cfg(test)]
mod tests {
    use super::source_image_from_file;

    #[test]
    fn works_with_local_file_paths() {
        // This is quite tied to some files - but honestly
        // might as well just test it this way
        assert!(source_image_from_file("./images", "/test_card_sml.png").is_ok());
        assert!(source_image_from_file("./images", "/this_image_doesnt_exist.png").is_err());
    }
}