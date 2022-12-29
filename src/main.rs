use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::Path;

use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;

use futures::stream::StreamExt;

use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
mod parameters;
mod process;

async fn handle_image_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let validated = validate_request(req);
    match validated {
        Ok((img, img_format, params)) => Ok(Response::builder()
            .status(StatusCode::OK)
            .body(process::process_image_to_buffer(img, img_format, params).into()).unwrap()),
        Err(err_res) => Ok(err_res),
    }
}

fn validate_request(req: Request<Body>) -> Result<(image::DynamicImage, image::ImageFormat, parameters::ImageParameters), Response<Body>> {
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

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let mut signals = Signals::new(&[
        SIGTERM,
        SIGINT,
        SIGQUIT,
    ])?;

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(handle_image_request))
    });

    let server = Server::bind(&addr)
        .serve(make_svc)
        .with_graceful_shutdown(async {
            while let Some(signal) = signals.next().await {
                match signal {
                    SIGTERM | SIGINT | SIGQUIT => {
                        return ();
                    },
                    _ => unreachable!(),
                }
            }
        });

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
    Ok(())
}