use futures::{future,Future,Stream};
use pretty_env_logger;
use hyper::{Server,Request,Response,Client};
use hyper::service::{service_fn,make_service_fn,service_fn_ok};
use hyper::{Body,Method,StatusCode,header,Uri,Chunk};
use hyper::server::conn::AddrStream;
use log::*;
use std::net::SocketAddr;

mod http;
mod types;

static URL: &str = "https://btapi.ipip.net/host/info";
static UA: &str = "ipip/tt";
static AE: &str = "gzip";


type GenericError = Box<dyn std::error::Error + Send + Sync>;
type ResponseFuture = Box<dyn Future<Item=Response<Body>, Error=GenericError> + Send>;

fn index_get(req: Request<Body>,remote_addr: SocketAddr) -> ResponseFuture {
    let mut response = Response::new(Body::empty());
    let ip = http::Ipfromrequerst(req,&remote_addr).unwrap();
    *response.body_mut() = Body::from(ip);
    Box::new(future::ok(response))
}

fn bt_api(req: Request<Body>,remote_addr: SocketAddr) -> ResponseFuture {
    let ip = http::Ipfromrequerst(req,&remote_addr).unwrap();
    dbg!(&ip);
    let url = format!("https://btapi.ipip.net/host/info?ip={}&host=&lang={}",ip,"cn");
    let bt_url = url.parse::<Uri>().unwrap();
    dbg!(&bt_url);
    // let req = Request::builder()
    //     .method(Method::GET)
    //     .uri(bt_url)
    //     .header(header::USER_AGENT, UA)
    //     .header(header::ACCEPT_ENCODING, AE)
    //     .body(Body::empty())
    //     .unwrap();
    let client = Client::new();
    Box::new(client.get(bt_url)
        .from_err().map(|web_res| {
        dbg!(&web_res);
        // Compare the JSON we sent (before) with what we received (after):
        let body = Body::wrap_stream(web_res.into_body().map(|b| {

            Chunk::from(format!("<b>Response</b>: {}",
                                std::str::from_utf8(&b).unwrap()))
        }));
        Response::new(body)
    }))
}


fn echoip(req: Request<Body>,remote_addr: SocketAddr) -> ResponseFuture {
    let mut response = Response::new(Body::empty());

    match (req.method(),req.uri().path()) {
        (&Method::GET,"/") => {
            index_get(req, remote_addr)
        },
        (&Method::POST,"/") => {
            *response.body_mut() = req.into_body();
            Box::new(future::ok(response))
        },
        (&Method::GET,"/bt") => {
            bt_api(req, remote_addr)
        },
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
            Box::new(future::ok(response))
        }
    }
}



fn main() {
    let addr = "0.0.0.0:1337".parse().unwrap();
    let make_ser = make_service_fn(|conn: &AddrStream| {
        let remote_addr = conn.remote_addr();
        service_fn(move |req: Request<Body>| {
            echoip(req,remote_addr)
        })
    });
    let server = Server::bind(&addr)
        .serve(make_ser)
        .map_err(|e| eprintln!("server error: {}",e));;
    println!("Listening on http://{}", addr);
    hyper::rt::run(server);
}
