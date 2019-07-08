use futures::{future, Future, Stream};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn, service_fn_ok};
use hyper::{header, Body, Chunk, Method, StatusCode, Uri};
use hyper::{Client, Request, Response, Server};
use serde_json;
use std::net::SocketAddr;

use serde::{Serialize, Deserialize};
use crate::http;

static URL: &str = "http://btapi.ipip.net/host/info";
static UA: &str = "ipip/tt";
static AE: &str = "gzip";


type GenericError = Box<dyn std::error::Error + Send + Sync>;
type ResponseFuture = Box<dyn Future<Item = Response<Body>, Error = GenericError> + Send>;

#[derive(Serialize, Deserialize, Debug)]
struct Btdata {
    r#as : String,
    area: String
}

pub fn bt_api(req: Request<Body>, remote_addr: SocketAddr) -> ResponseFuture {
    let ip  = http::Ipfromrequerst(req, &remote_addr).unwrap();
    dbg!(&ip);
    let url = format!(
        "http://btapi.ipip.net/host/info?ip={}&host=&lang={}",
        &ip, "cn"
    );
    let bt_url = url.parse::<Uri>().unwrap();
    dbg!(&bt_url);
    let req = Request::builder()
        .method(Method::GET)
        .uri(bt_url)
        .header(header::USER_AGENT, UA)
        .body(Body::empty())
        .unwrap();
    let client = Client::new();
    Box::new(
        client
            .request(req)
            .from_err()
            .map( |web_res| {
                dbg!(&web_res);
                let body = Body::wrap_stream(web_res.into_body().map(|b| {
                    let data: Btdata = serde_json::from_slice(&b).unwrap();
                    Chunk::from(format!(
                        "AS号码 : {}\n地址   ：{}\n",
                        data.r#as,data.area
                    ))
                }));
                Response::new(body)
            })
    )
}