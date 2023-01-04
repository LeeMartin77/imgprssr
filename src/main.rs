use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;

use config::Config;
use imgprssr::{appconfig, process};
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


pub fn generate_app_config() -> Result<(([u8; 4], u16), appconfig::ImgprssrConfig), appconfig::ImgprssrConfigErr> {
    let raw_hashmap = Config::builder()
        // ENV Variables are IMGPRSSR_SOMETHING == something
        .add_source(config::Environment::with_prefix("IMGPRSSR"))
        .build()
        .unwrap()
        .try_deserialize::<HashMap<String, String>>()
        .unwrap();

    let mut address = [0, 0, 0, 0];
    let mut port = 3000;

    if let Some(config_address) = raw_hashmap.get("address") {
        let split = config_address.split(".");
        let mut parsed = split.map(|a| a.parse::<u8>()).into_iter();
        if parsed.clone().count() != 4 && !parsed.clone().all(|res| res.is_ok()){
            return Err(appconfig::ImgprssrConfigErr::InvalidValues(vec!["address".to_owned()]))
        }
        let mut i = 0;
        while let Some(part) = parsed.next() {
            address[i] = part.unwrap();
            i += 1;
        }
    }

    if let Some(config_port) = raw_hashmap.get("port") {
        if let Ok(prsd_port) = config_port.parse::<u16>() {
            port = prsd_port;
        } else {
            return Err(appconfig::ImgprssrConfigErr::InvalidValues(vec!["port".to_owned()]))
        }
    }
    
    let app_config_res = appconfig::from_hashmap(raw_hashmap);

    match app_config_res {
        Ok(app_config) => Ok(((address, port), app_config)),
        Err(err) => Err(err),
    }
  }

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let mut signals = Signals::new(&[
        SIGTERM,
        SIGINT,
        SIGQUIT,
    ])?;

    let ((bind, port), settings) = generate_app_config().unwrap();
    
    let addr = SocketAddr::from((bind.clone(), port.clone()));

    println!("imgprssr sourcing images from {}", settings.image_source);

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