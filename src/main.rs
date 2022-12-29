use std::convert::Infallible;
use std::io::{Cursor, Read};
use std::net::SocketAddr;
use std::path::Path;

use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use image::DynamicImage;
use parameters::ImageParameters;
mod parameters;

fn process_image(mut img: DynamicImage, img_format: image::ImageFormat, params: ImageParameters) -> Vec<u8> {
    let mut buffer = Cursor::new(Vec::new());

    if let Some(w) = params.width {
        let source_width = img.width();
        if source_width != w {
            let width_factor = source_width as f32 / w as f32;
            let nheight = img.height() as f32 * width_factor;
            img = img.resize(w, nheight as u32, image::imageops::FilterType::Nearest);
        }
    }

    img.write_to(&mut buffer, img_format).unwrap();

    let mut out = Vec::new();
    buffer.set_position(0);
    buffer.read_to_end(&mut out).unwrap();
    out
}

async fn handle_image_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let target_path = req.uri().path();
    let params_res: Result<ImageParameters, _> = req.uri().query().unwrap_or("").parse();
    if params_res.is_err() {
        return Ok(Response::builder()
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
            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(process_image(img, img_format, params).into()).unwrap())
        },
        Err(_) => {
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body("Not Found".into()).unwrap().into())
        }
    }
    
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(handle_image_request))
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}