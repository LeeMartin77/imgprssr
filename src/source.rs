use std::{borrow::Cow, path::Path, str::FromStr};
use image::{self, DynamicImage, ImageError, ImageFormat};
use http_body_util::{BodyExt, Full};
use hyper::{body::Incoming, Request, Response, StatusCode, Uri};
use hyper::body::Bytes;
use hyper_util::client::legacy::{Client, connect::HttpConnector};
use hyper_tls::HttpsConnector;
use crate::{appconfig, parameters};

/// Sanitises a user-supplied path segment: decodes any percent-encoding,
/// then rejects `..` components that could escape the intended directory.
pub fn sanitize_user_path(input: &str) -> Result<String, ImageError> {
    let decoded = urlencoding::decode(input).unwrap_or_else(|_| Cow::Borrowed(input));
    if decoded.contains("..") {
        return Err(ImageError::IoError(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Path traversal not allowed",
        )));
    }
    Ok(decoded.into_owned())
}

async fn handle_response(
    response: Result<Response<Incoming>, hyper_util::client::legacy::Error>,
) -> Result<(DynamicImage, ImageFormat), ImageError> {
    match response {
        Ok(res) => {
            let bytes = res
                .collect()
                .await
                .map(|c| c.to_bytes())
                .map_err(|_| {
                    ImageError::IoError(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Failed to read response body",
                    ))
                })?;
            let img = image::load_from_memory(&bytes);
            match img {
                Ok(img) => Ok((img, image::guess_format(&bytes).unwrap())),
                Err(err) => Err(err),
            }
        }
        Err(_) => Err(ImageError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to load image",
        ))),
    }
}

pub async fn source_image_from_http(
    (client, img_source): &(Client<HttpConnector, String>, String),
    target_path: &str,
) -> Result<(DynamicImage, ImageFormat), ImageError> {
    sanitize_user_path(target_path)?;
    let full_path = format!("{}{}", img_source, target_path);
    handle_response(client.get(Uri::from_str(&full_path).unwrap()).await).await
}

pub async fn source_image_from_https(
    (client, img_source): &(Client<HttpsConnector<HttpConnector>, String>, String),
    target_path: &str,
) -> Result<(DynamicImage, ImageFormat), ImageError> {
    sanitize_user_path(target_path)?;
    let full_path = format!("{}{}", img_source, target_path);
    handle_response(client.get(Uri::from_str(&full_path).unwrap()).await).await
}

pub fn source_image_from_file(
    img_source: &str,
    target_path: &str,
) -> Result<(DynamicImage, ImageFormat), ImageError> {
    sanitize_user_path(target_path)?;
    let full_path = format!("{}{}", img_source, target_path);
    let path = Path::new(&full_path);
    match image::open(path) {
        Ok(img) => Ok((img, image::ImageFormat::from_path(path).unwrap())),
        Err(err) => Err(err),
    }
}

pub async fn get_source_image(
    settings: &appconfig::ImgprssrConfig,
    req: Request<Incoming>,
) -> Result<
    (image::DynamicImage, image::ImageFormat, parameters::ImageParameters),
    Response<Full<Bytes>>,
> {
    let target_path = req.uri().path();
    let params_res: Result<parameters::ImageParameters, _> =
        req.uri().query().unwrap_or("").parse();
    if params_res.is_err() {
        return Err(hyper::Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Full::new(Bytes::from("Bad Request".to_owned())))
            .unwrap());
    }
    let params = params_res.unwrap();
    let img_res = match &settings.image_source {
        appconfig::ImgSource::Folder(fldr) => source_image_from_file(&fldr, target_path),
        appconfig::ImgSource::Https(cfg) => source_image_from_https(cfg, target_path).await,
        appconfig::ImgSource::Http(cfg) => source_image_from_http(cfg, target_path).await,
    };
    match img_res {
        Ok((img, fmt)) => Ok((img, fmt, params)),
        Err(_) => Err(hyper::Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Full::new(Bytes::from("Not Found".to_owned())))
            .unwrap()),
    }
}

#[cfg(test)]
mod tests {
    use hyper_util::client::legacy::Client;
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

    #[test]
    fn path_traversal_rejected() {
        // Attempt to traverse outside the image directory.
        // Without sanitisation this would read /etc/passwd (or panic if unreadable).
        // With the fix, any path containing ".." that escapes the source dir must error.
        let result = source_image_from_file("./images", "/../../etc/passwd");
        assert!(
            result.is_err(),
            "Path traversal should be rejected, not allowed"
        );
    }

    #[test]
    fn path_traversal_double_dot_rejected() {
        // Alternative traversal attempt
        let result = source_image_from_file("./images", "/foo/../../../etc/hosts");
        assert!(
            result.is_err(),
            "Path traversal with multiple '..' should be rejected"
        );
    }

    #[test]
    fn path_traversal_can_read_outside_directory() {
        // Creates a temp directory with a valid PNG outside the source dir.
        // Without sanitisation this reads the file → test fails (RED).
        // With sanitisation this returns Err → test passes (GREEN).
        let temp_base = std::env::temp_dir().join("imgprssr_path_traversal_test");
        let _ = std::fs::create_dir_all(&temp_base);
        let imgs_dir = temp_base.join("imgs");
        let _ = std::fs::create_dir_all(&imgs_dir);

        // Write a valid PNG outside the source directory
        let secret_img = image::DynamicImage::new_rgb8(10, 10);
        let mut secret_buf = Vec::new();
        secret_img
            .write_to(
                &mut std::io::Cursor::new(&mut secret_buf),
                image::ImageFormat::Png,
            )
            .unwrap();
        std::fs::write(temp_base.join("secret.png"), &secret_buf).unwrap();

        // Try to traverse out of imgs/ and read secret.png
        let result = source_image_from_file(imgs_dir.to_str().unwrap(), "/../secret.png");

        // Clean up
        let _ = std::fs::remove_dir_all(&temp_base);

        assert!(
            result.is_err(),
            "Path traversal should be blocked but it successfully read secret.png"
        );
    }
    #[tokio::test]
    async fn works_with_http_file_addresses() {
        let https = HttpsConnector::new();
        let client = Client::builder(hyper_util::rt::TokioExecutor::new())
            .build::<_, String>(https);
        assert!(source_image_from_https(
            &(client, "https://raw.githubusercontent.com/LeeMartin77/imgprssr/main/images".to_owned()),
            "/test_card_sml.png"
        )
        .await
        .is_ok());
    }
    #[tokio::test]
    async fn errors_with_http_file_addresses() {
        let https = HttpsConnector::new();
        let client = Client::builder(hyper_util::rt::TokioExecutor::new())
            .build::<_, String>(https);
        assert!(source_image_from_https(
            &(client, "https://raw.githubusercontent.com/LeeMartin77/imgprssr/main/images".to_owned()),
            "/this_image_doesnt_exist.png"
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn http_path_traversal_rejected() {
        let https = HttpsConnector::new();
        let client = Client::builder(hyper_util::rt::TokioExecutor::new())
            .build::<_, String>(https);
        // Should be rejected before any network call
        assert!(source_image_from_https(
            &(client, "https://example.com".to_owned()),
            "/../../../etc/passwd"
        )
        .await
        .is_err());
    }

    #[tokio::test]
    async fn http_url_encoded_traversal_rejected() {
        let https = HttpsConnector::new();
        let client = Client::builder(hyper_util::rt::TokioExecutor::new())
            .build::<_, String>(https);
        // %2e%2e decodes to .. — should also be rejected
        assert!(source_image_from_https(
            &(client, "https://example.com".to_owned()),
            "/%2e%2e/%2e%2e/etc/passwd"
        )
        .await
        .is_err());
    }
}
