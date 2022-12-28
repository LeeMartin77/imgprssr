use std::convert::Infallible;
use std::net::SocketAddr;


use std::fs::{File, DirBuilder};
use std::io::Read;
//use std::io::prelude::*;
use std::path::Path;
//use std::error::Error;

use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use uuid::Uuid;

async fn hello_world(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let target_path = req.uri().path();
    let raw_path = format!("./images{}", target_path);
    let path = Path::new(&raw_path);
    let img_res = image::open(path);
    match img_res {
        Ok(img) => {
            let temp_dir = format!("/tmp/{}", Uuid::new_v4());
            let temp_path =  format!("{}{}", temp_dir, target_path);
            println!("{}",temp_path);
            DirBuilder::new().create(temp_dir).unwrap();
            File::create(&temp_path).unwrap();
            img.save(&temp_path).unwrap();
            let mut file = File::open(&temp_path).unwrap();
            let mut buff = vec![];
            file.read_to_end(&mut buff).unwrap();
            Ok(Response::builder()
            .status(StatusCode::OK)
            .body(buff.into()).unwrap())
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