use std::convert::Infallible;
use std::net::SocketAddr;


use std::fs::File;
//use std::io::prelude::*;
use std::path::Path;
//use std::error::Error;

use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};

async fn hello_world(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let raw_path = format!("./images{}", req.uri().path());
    let path = Path::new(&raw_path);
    let file = File::open(&path);
    Ok(Response::new(format!("Hello {}", if file.is_ok() {"found"} else {"notfound"}).into()))
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