use std::{path::Path, str::FromStr};
use imgprssr_core;
use image::{self, DynamicImage, ImageError, ImageFormat};
use hyper::{Request, Body, Response, StatusCode, Client, Uri};
use hyper_tls::HttpsConnector;

pub async fn source_image_from_file_or_http(img_source: &str, target_path: &str) -> Result<(DynamicImage, ImageFormat), ImageError> {
    let full_path = format!("{}{}", img_source, target_path);
    // TODO: img_source.starts_with("http://") 
    // TODO: if sourced on http/s, have a client on settings
    if img_source.starts_with("https://") {
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);
        match client.get(Uri::from_str(&full_path).unwrap()).await {
            Ok(res) => {
                println!("here");
                let buf = hyper::body::to_bytes(res).await;
                match buf {
                    Ok(bytes) => {
                        let img = image::load_from_memory(&bytes);
                        match img {
                            Ok(img) => Ok((img, image::guess_format(&bytes).unwrap())),
                            Err(err) => Err(err),
                        }
                    },
                    Err(_) => Err(ImageError::IoError(std::io::Error::new(std::io::ErrorKind::Other ,"Failed to load image"))),
                }
            },
            Err(_) => Err(ImageError::IoError(std::io::Error::new(std::io::ErrorKind::Other ,"Failed to load image"))),
        }
    } else {
        let path = Path::new(&full_path);
        match image::open(path) {
            Ok(img) => Ok((img, image::ImageFormat::from_path(path).unwrap())),
            Err(err) => Err(err),
        }
    }

}

pub async fn get_source_image(settings: &imgprssr_core::appconfig::ImgprssrConfig, req: Request<Body>) -> Result<(image::DynamicImage, image::ImageFormat, imgprssr_core::parameters::ImageParameters), Response<Body>> {
  let target_path = req.uri().path();
  let params_res: Result<imgprssr_core::parameters::ImageParameters, _> = req.uri().query().unwrap_or("").parse();
  if params_res.is_err() {
      return Err(Response::builder()
          .status(StatusCode::BAD_REQUEST)
          .body("Bad Request".into()).unwrap().into())
  }
  let params = params_res.unwrap();
  let img_res = source_image_from_file_or_http(&settings.image_source, target_path).await;
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
    use super::source_image_from_file_or_http;

    // This is quite tied to some files - but honestly
    // might as well just test it this way
    #[tokio::test]
    async fn works_with_local_file_paths() {
        assert!(source_image_from_file_or_http("./images", "/test_card_sml.png").await.is_ok());
    }
    #[tokio::test]
    async fn errors_with_local_file_paths() {
        assert!(source_image_from_file_or_http("./images", "/this_image_doesnt_exist.png").await.is_err());
    }
    #[tokio::test]
    async fn works_with_http_file_addresses() {
        assert!(source_image_from_file_or_http("https://raw.githubusercontent.com/LeeMartin77/imgprssr/main/images", "/test_card_sml.png").await.is_ok());
    }
    #[tokio::test]
    async fn errors_with_http_file_addresses() {
        assert!(source_image_from_file_or_http("https://raw.githubusercontent.com/LeeMartin77/imgprssr/main/images", "/this_image_doesnt_exist.png").await.is_err());
    }
}