use std::path::Path;
use imgprssr_core;
use image;
use hyper::{Request, Body, Response, StatusCode};

pub fn get_source_image(settings: &imgprssr_core::appconfig::ImgprssrConfig, req: Request<Body>) -> Result<(image::DynamicImage, image::ImageFormat, imgprssr_core::parameters::ImageParameters), Response<Body>> {
  let target_path = req.uri().path();
  let params_res: Result<imgprssr_core::parameters::ImageParameters, _> = req.uri().query().unwrap_or("").parse();
  if params_res.is_err() {
      return Err(Response::builder()
          .status(StatusCode::BAD_REQUEST)
          .body("Bad Request".into()).unwrap().into())
  }
  let params = params_res.unwrap();
  
  let raw_path = format!("{}{}", settings.image_source, target_path);
  let path = Path::new(&raw_path);
  let img_res = image::open(path);
  match img_res {
      Ok(img) => {
          let img_format = image::ImageFormat::from_path(path).unwrap();
          Ok((img, img_format, params))
      },
      Err(_) => {
          Err(Response::builder()
              .status(StatusCode::NOT_FOUND)
              .body("Not Found".into()).unwrap().into())
      }
  }
}