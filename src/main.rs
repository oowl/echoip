use futures::{future,Future,Stream};
use pretty_env_logger;
use hyper::{Server,Request,Response,Client};
use hyper::service::{service_fn,make_service_fn,service_fn_ok};
use hyper::{Body,Method,StatusCode};
use hyper::server::conn::AddrStream;
use log::*;
use std::net::SocketAddr;

mod http;
mod types;

// type BoxFut = Box<dyn Future<Item = Response<Body>, Error = hyper::Error> + Send>;

// fn echoip(req: Request<Body>,) -> BoxFut {
//     let mut response = Response::new(Body::empty());

//     match (req.method(),req.uri().path()) {
//         (&Method::GET,"/") => {
//             let ip = http::Ipfromrequerst(req).unwrap();
//             *response.body_mut() = Body::from(ip.to_string());
//         },
//         (&Method::POST,"/") => {
//             *response.body_mut() = req.into_body();
//         }
//         _ => {
//             *response.status_mut() = StatusCode::NOT_FOUND;
//         }
//     };
//     Box::new(future::ok(response))
// }



fn main() {
    let addr = "0.0.0.0:1337".parse().unwrap();
    let make_ser = make_service_fn(|conn: &AddrStream| {
        let remote_addr = conn.remote_addr();
        service_fn_ok(move |req: Request<Body>| {
            let mut response = Response::new(Body::empty());

            match (req.method(),req.uri().path()) {
                (&Method::GET,"/") => {
                    let ip = http::Ipfromrequerst(&req,&remote_addr).unwrap();
                    *response.body_mut() = Body::from(ip);
                },
                (&Method::POST,"/") => {
                    *response.body_mut() = req.into_body();
                }
                _ => {
                    *response.status_mut() = StatusCode::NOT_FOUND;
                }
            };
            response
        })
    });
    let server = Server::bind(&addr)
        .serve(make_ser)
        .map_err(|e| eprintln!("server error: {}",e));;
    println!("Listening on http://{}", addr);
    hyper::rt::run(server);
}
