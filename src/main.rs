use std::convert::Infallible;
use std::net::SocketAddr;

use std::io::{Read, Cursor};
//use std::io::prelude::*;
use std::path::Path;
use std::time::Instant;
//use std::error::Error;

use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};

async fn hello_world(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let target_path = req.uri().path();
    let raw_path = format!("./images{}", target_path);
    let path = Path::new(&raw_path);
    let now = Instant::now();
    
    println!("Preload: {:?}", now.elapsed());
    let img_res = image::open(path);
    println!("Postload: {:?}", now.elapsed());

    match img_res {
        Ok(img) => {
            let mut buffer = Cursor::new(Vec::new());

            println!("Prewrite: {:?}", now.elapsed());
            img.write_to(&mut buffer, image::ImageFormat::from_path(path).unwrap()).unwrap();
            println!("Postwrite: {:?}", now.elapsed());

            println!("Prebuffer: {:?}", now.elapsed());
            let mut out = Vec::new();
            buffer.set_position(0);
            buffer.read_to_end(&mut out).unwrap();
            println!("Postbuffer: {:?}", now.elapsed());

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