use futures::{future,Future,Stream};
use pretty_env_logger;
use hyper::{Server,Request,Response,Client};
use hyper::service::service_fn;
use hyper::{Body,Method,StatusCode};
use log::*;

type BoxFut = Box<dyn Future<Item = Response<Body>, Error = hyper::Error> + Send>;

fn echoip(req: Request<Body>) -> BoxFut {
    let mut response = Response::new(Body::empty());

    match (req.method(),req.uri().path()) {
        (&Method::GET,"/") => {
            *response.body_mut() = Body::from("hello world");
        },
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    };
    Box::new(future::ok(response))
}

fn main() {
    let addr = "127.0.0.1:1337".parse().unwrap();
    let server = Server::bind(&addr)
            .serve(|| service_fn(echoip))
            .map_err(|e| eprintln!("server error: {}",e));
    println!("Listening on http://{}", addr);
    hyper::rt::run(server);
}
