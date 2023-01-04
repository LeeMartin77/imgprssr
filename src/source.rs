use std::{path::Path, str::FromStr};
use image::{self, DynamicImage, ImageError, ImageFormat};
use hyper::{Request, Body, Response, StatusCode, Client, Uri, client::{HttpConnector}};
use hyper_tls::HttpsConnector;
use imgprssr::{appconfig, parameters};

async fn handle_response(response: Result<Response<Body>, hyper::Error>) -> Result<(DynamicImage, ImageFormat), ImageError> {
    match response {
        Ok(res) => {
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
}

pub async fn source_image_from_http((client, img_source): &(Client<HttpConnector>, String), target_path: &str) -> Result<(DynamicImage, ImageFormat), ImageError> {
    let full_path = format!("{}{}", img_source, target_path);
    handle_response(client.get(Uri::from_str(&full_path).unwrap()).await).await
}

pub async fn source_image_from_https((client, img_source): &(Client<HttpsConnector<HttpConnector>>, String), target_path: &str) -> Result<(DynamicImage, ImageFormat), ImageError> {
    let full_path = format!("{}{}", img_source, target_path);
    handle_response(client.get(Uri::from_str(&full_path).unwrap()).await).await
}

pub fn source_image_from_file(img_source: &str, target_path: &str) -> Result<(DynamicImage, ImageFormat), ImageError> {
    let full_path = format!("{}{}", img_source, target_path);
    let path = Path::new(&full_path);
    match image::open(path) {
        Ok(img) => Ok((img, image::ImageFormat::from_path(path).unwrap())),
        Err(err) => Err(err),
    }
}

pub async fn get_source_image(settings: &appconfig::ImgprssrConfig, req: Request<Body>) -> Result<(image::DynamicImage, image::ImageFormat, parameters::ImageParameters), Response<Body>> {
  let target_path = req.uri().path();
  let params_res: Result<parameters::ImageParameters, _> = req.uri().query().unwrap_or("").parse();
  if params_res.is_err() {
      return Err(Response::builder()
          .status(StatusCode::BAD_REQUEST)
          .body("Bad Request".into()).unwrap().into())
  }
  let params = params_res.unwrap();
  let img_res = match &settings.image_source {
    appconfig::ImgSource::Folder(fldr) => source_image_from_file(&fldr, target_path),
    appconfig::ImgSource::Https(cfg) => source_image_from_https(cfg, target_path).await,
    appconfig::ImgSource::Http(cfg) => source_image_from_http(cfg, target_path).await,
  };
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
    use hyper::Client;
    use hyper_tls::HttpsConnector;

    use crate::source::source_image_from_https;

    use super::source_image_from_file;

    // This is quite tied to some files - but honestly
    // might as well just test it this way
    #[test]
    fn works_with_local_file_paths() {
        assert!(source_image_from_file("./images", "/test_card_sml.png").is_ok());
    }
    #[test]
    fn errors_with_local_file_paths() {
        assert!(source_image_from_file("./images", "/this_image_doesnt_exist.png").is_err());
    }
    #[tokio::test]
    async fn works_with_http_file_addresses() {
        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);
        assert!(source_image_from_https(&(client, "https://raw.githubusercontent.com/LeeMartin77/imgprssr/main/images".to_owned()), "/test_card_sml.png").await.is_ok());
    }
    #[tokio::test]
    async fn errors_with_http_file_addresses() {

        let https = HttpsConnector::new();
        let client = Client::builder().build::<_, hyper::Body>(https);
        assert!(source_image_from_https(&(client, "https://raw.githubusercontent.com/LeeMartin77/imgprssr/main/images".to_owned()), "/this_image_doesnt_exist.png").await.is_err());
    }
}