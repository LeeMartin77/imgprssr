use std::convert::Infallible;
use std::io::{Cursor, Read};
use std::net::SocketAddr;
use std::path::Path;

use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use parameters::ImageParameters;
mod parameters;

async fn hello_world(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let target_path = req.uri().path();
    
    let raw_path = format!("./images{}", target_path);
    let path = Path::new(&raw_path);
    
    let img_res = image::open(path);

    match img_res {
        Ok(mut img) => {
            let mut buffer = Cursor::new(Vec::new());
            let params_res: Result<ImageParameters, _> = req.uri().query().unwrap_or("").parse();

            if let Ok(params) = params_res {
                if let Some(w) = params.width {
                    let source_width = img.width();
                    if source_width != w {
                        let width_factor = source_width as f32 / w as f32;
                        let nheight = img.height() as f32 * width_factor;
                        img = img.resize(w, nheight as u32, image::imageops::FilterType::Nearest);
                    }
                }
            } else {
                return Ok(Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body("Bad Request".into()).unwrap().into())
            }

            img.write_to(&mut buffer, image::ImageFormat::from_path(path).unwrap()).unwrap();

            let mut out = Vec::new();
            buffer.set_position(0);
            buffer.read_to_end(&mut out).unwrap();

            Ok(Response::builder()
                .status(StatusCode::OK)
                .body(out.into()).unwrap())
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
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(hello_world))
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}