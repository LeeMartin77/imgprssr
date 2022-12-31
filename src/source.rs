use std::path::Path;

use hyper::{Request, Body, Response, StatusCode};

use crate::parameters;

pub fn get_source_image(req: Request<Body>) -> Result<(image::DynamicImage, image::ImageFormat, parameters::ImageParameters), Response<Body>> {
  let target_path = req.uri().path();
  let params_res: Result<parameters::ImageParameters, _> = req.uri().query().unwrap_or("").parse();
  if params_res.is_err() {
      return Err(Response::builder()
          .status(StatusCode::BAD_REQUEST)
          .body("Bad Request".into()).unwrap().into())
  }
  let params = params_res.unwrap();
  
  let raw_path = format!("./images{}", target_path);
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