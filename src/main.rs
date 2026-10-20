use std::convert::Infallible;
use std::net::SocketAddr;

use imgprssr::{appconfig, process, server_config};
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;

use futures::stream::StreamExt;

use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
mod source;

async fn handle_image_request(settings: appconfig::ImgprssrConfig, req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let sourced = source::get_source_image(&settings, req).await;
    match sourced {
        Ok((img, img_format, params)) => Ok(Response::builder()
            .status(StatusCode::OK)
            .body(process::process_image_to_buffer(&settings, img, img_format, params).into()).unwrap()),
        Err(err_res) => Ok(err_res),
    }
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let mut signals = Signals::new(&[
        SIGTERM,
        SIGINT,
        SIGQUIT,
    ])?;

    let ((bind, port), settings) = server_config::generate_app_config().unwrap();
    
    let addr = SocketAddr::from((bind.clone(), port.clone()));

    println!("imgprssr sourcing images from {}", match &settings.image_source {
        appconfig::ImgSource::Folder(fldr) => fldr,
        appconfig::ImgSource::Https((_, addr)) => addr,
        appconfig::ImgSource::Http((_, addr)) => addr,
    });

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

    println!("imgprssr running on {}.{}.{}.{}:{}", bind[0], bind[1], bind[2], bind[3], port);
    
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
    Ok(())
}