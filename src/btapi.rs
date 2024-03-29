use futures::{future, Future, Stream};
use futures::future::FutureResult;
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn, service_fn_ok};
use hyper::{header, Body, Chunk, Method, StatusCode, Uri};
use hyper::{Client, Request, Response, Server};
use serde_json;
use std::net::SocketAddr;
use log::*;

use serde::{Serialize, Deserialize};
use crate::http;

static URL: &str = "http://btapi.ipip.net/host/info";
static UA: &str = "ipip/tt";
static AE: &str = "gzip";


type GenericError = Box<dyn std::error::Error + Send + Sync>;
type ResponseFuture = Box<dyn Future<Item = Response<Body>, Error = GenericError> + Send>;


#[derive(Serialize, Deserialize, Debug,Clone)]
pub struct Btdata {
    r#as : String,
    area: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Ipdata {
    pub as_num: String,
    pub l1: String,
    pub l2: String,
    pub l3: String,
    pub l4: String,
    pub isp: String,
    pub lat: String,
    pub lng: String,
    pub ip: String
}

impl Ipdata {
    pub fn new(bt: Btdata,remote_addr: &str) -> Ipdata {
        let as_num = bt.r#as;
        let area: Vec<&str> = bt.area.split("\t").collect();
        Ipdata {
            as_num: as_num,
            l1: area[0].to_string(),
            l2: area[1].to_string(),
            l3: area[2].to_string(),
            l4: area[3].to_string(),
            isp: area[4].to_string(),
            lat: area[5].to_string(),
            lng: area[6].to_string(),
            ip: remote_addr.to_string()
        }
    }
}

pub fn bt_api_req(ip_addr: &str) -> ResponseFuture {
   let url = format!(
        "http://btapi.ipip.net/host/info?ip={}&host=&lang={}",
        &ip_addr, "cn"
    );
    let bt_url = url.parse::<Uri>().unwrap();
    // dbg!(&bt_url);
    let req = Request::builder()
        .method(Method::GET)
        .uri(bt_url)
        .header(header::USER_AGENT, UA)
        .body(Body::empty())
        .unwrap();
    let client = Client::new();
    Box::new(client.request(req).from_err())
}


pub fn bt_api(req: Request<Body>, remote_addr: String) -> ResponseFuture {
    let ip  = remote_addr;
    // dbg!(&ip);
    let url = format!(
        "http://btapi.ipip.net/host/info?ip={}&host=&lang={}",
        &ip, "cn"
    );
    let req_useragent = req.headers().get("User-Agent").unwrap().to_str().unwrap().to_string(); 
    let bt_url = url.parse::<Uri>().unwrap();
    // dbg!(&bt_url);
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
            .map( move |web_res| {
                let body = Body::wrap_stream(web_res.into_body().map(move |b| {
                    let data: Btdata = serde_json::from_slice(&b).unwrap();
                    let ip_data = Ipdata::new(data,&ip);
                    let address = ip_data.l1 + " " + &ip_data.l2 + " " + &ip_data.l3;
                    // info!("ip: {:15},address: {:10},isp: {:8}",ip,address,ip_data.isp);
                    if req_useragent.contains("Gecko") {
                        Chunk::from(format!(
                            "IP     : {}</br>AS号码 : {}</br>地址   ：{}</br>运营商 : {}</br>",
                            ip_data.ip,ip_data.as_num,address,ip_data.isp
                        ))
                    } else {
                        Chunk::from(format!(
                            "IP     : {}\nAS号码 : {}\n地址   ：{}\n运营商 : {}\n",
                            ip_data.ip,ip_data.as_num,address,ip_data.isp
                        ))
                    }
                }));
                let mut res = Response::new(body);
                res.headers_mut().insert(header::CONTENT_TYPE, "text/html; charset=utf-8".parse().unwrap());
                res
            })
    )
}