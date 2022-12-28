use std::collections::HashMap;
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
    let query_string = req.uri().query().unwrap_or("");
    let query_parts = query_string.split("&");
    let mut params: HashMap<&str, &str> = HashMap::new();

    for q in query_parts {
        let mut prts = q.split("=").into_iter();
        let key = prts.next().unwrap();
        let val = prts.next().unwrap_or("true");
        params.insert(key, val);
    }
    let raw_path = format!("./images{}", target_path);
    let path = Path::new(&raw_path);
    let now = Instant::now();
    
    println!("Preload: {:?}", now.elapsed());
    let img_res = image::open(path);
    println!("Postload: {:?}", now.elapsed());

    match img_res {
        Ok(mut img) => {
            let mut buffer = Cursor::new(Vec::new());

            let width = params.get("width");

            if let Some(w) = width {
                let wir: Result<u32, _> = w.parse();
                let source_width = img.width();
                if let Ok(width_int) = wir  { 
                    if source_width != width_int {
                        let width_factor = source_width as f32 / width_int as f32;
                        let nheight = img.height() as f32 * width_factor;
                        img = img.resize(width_int, nheight as u32, image::imageops::FilterType::Nearest);
                    }
                }
            }

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