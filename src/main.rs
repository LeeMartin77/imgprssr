use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;

use appconfig::{ImgprssrConfig, ImgprssrConfigErr};
use config::Config;
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;

use futures::stream::StreamExt;

use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
mod parameters;
mod process;
mod appconfig;
mod source;

async fn handle_image_request(settings: ImgprssrConfig, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let sourced = source::get_source_image(&settings, req);
    match sourced {
        Ok((img, img_format, params)) => Ok(Response::builder()
            .status(StatusCode::OK)
            .body(process::process_image_to_buffer(&settings, img, img_format, params).into()).unwrap()),
        Err(err_res) => Ok(err_res),
    }
}


pub fn generate_app_config() -> Result<ImgprssrConfig, ImgprssrConfigErr> {
    appconfig::from_hashmap(Config::builder()
          // ENV Variables are IMGPRSSR_SOMETHING == something
          .add_source(config::Environment::with_prefix("IMGPRSSR"))
          .build()
          .unwrap()
          .try_deserialize::<HashMap<String, String>>()
          .unwrap())
  }

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let mut signals = Signals::new(&[
        SIGTERM,
        SIGINT,
        SIGQUIT,
    ])?;

    let settings: ImgprssrConfig = generate_app_config().unwrap();
    
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    let make_svc = make_service_fn(move |_conn| {
        let settings = settings.clone();
        let mvd_fn = move |req| {
            let settings = settings.clone();
            handle_image_request(settings, req)
        };
        async move { Ok::<_, Infallible>(service_fn(mvd_fn)) }
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