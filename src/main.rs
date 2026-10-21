use std::convert::Infallible;
use std::net::SocketAddr;

use bytes::Bytes;
use futures::stream::StreamExt;
use http_body_util::Full;
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use hyper_util::server::graceful::GracefulShutdown;
use imgprssr::{appconfig, process, server_config, source};
use signal_hook::consts::signal::*;
use signal_hook_tokio::Signals;

async fn handle_image_request(
    settings: appconfig::ImgprssrConfig,
    req: hyper::Request<Incoming>,
) -> Result<hyper::Response<Full<Bytes>>, Infallible> {
    let sourced = source::get_source_image(&settings, req).await;
    match sourced {
        Ok((img, img_format, params)) => {
            let buf = process::process_image_to_buffer(&settings, img, img_format, params);
            Ok(hyper::Response::new(Full::new(Bytes::from(buf))))
        }
        Err(err_res) => Ok(err_res),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut signals = Signals::new(&[
        SIGTERM,
        SIGINT,
        SIGQUIT,
    ])?;

    let ((bind, port), settings) = server_config::generate_app_config().unwrap();

    let addr = SocketAddr::from((bind.clone(), port.clone()));

    println!(
        "imgprssr sourcing images from {}",
        match &settings.image_source {
            appconfig::ImgSource::Folder(fldr) => fldr,
            appconfig::ImgSource::Https((_, addr)) => addr,
            appconfig::ImgSource::Http((_, addr)) => addr,
        }
    );

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!(
        "imgprssr running on {}.{}.{}.{}:{}",
        bind[0], bind[1], bind[2], bind[3], port
    );

    let server = Builder::new(TokioExecutor::new());
    let graceful = GracefulShutdown::new();

    loop {
        tokio::select! {
            conn = listener.accept() => {
                let (stream, _peer_addr) = match conn {
                    Ok(conn) => conn,
                    Err(e) => {
                        eprintln!("accept error: {}", e);
                        continue;
                    }
                };

                let settings = settings.clone();
                let stream = TokioIo::new(stream);

                let conn = server.serve_connection_with_upgrades(
                    stream,
                    service_fn(move |req| {
                        let settings = settings.clone();
                        handle_image_request(settings, req)
                    }),
                );

                let conn = graceful.watch(conn.into_owned());

                tokio::spawn(async move {
                    if let Err(err) = conn.await {
                        eprintln!("connection error: {}", err);
                    }
                });
            },
            _ = signals.next() => {
                drop(listener);
                println!("signal received, starting shutdown");
                break;
            }
        }
    }

    graceful.shutdown().await;
    Ok(())
}
