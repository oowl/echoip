

use futures::{future, Future, Stream};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn, service_fn_ok};
use hyper::{header, Body, Chunk, Method, StatusCode, Uri};
use hyper::{Client, Request, Response, Server};
use serde_json;
use std::net::SocketAddr;
use log::*;
use serde::{Serialize, Deserialize};

use futures::future::IntoFuture;

use crate::btapi;
use crate::http;

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type ResponseFuture = Box<dyn Future<Item = Response<Body>, Error = GenericError> + Send>;

#[derive(Serialize, Deserialize, Debug)]
struct Mtrdata {
    service: String,
    ip: String,
}

pub fn index_post(req: Request<Body>, remote_addr: String) -> ResponseFuture {
    Box::new(req.into_body()
        .concat2() 
        .from_err()
        .and_then(move |entire_body| {
            let str = String::from_utf8(entire_body.to_vec()).unwrap();
            let mut data : Mtrdata = serde_json::from_str(&str).unwrap();
            if data.ip == ""{
                data.ip = remote_addr.to_string();
            }
            info!("ip: {:15} post / ,data: {} {},",&remote_addr,data.service,data.ip);
            let res: ResponseFuture = match data.service.as_ref() {
                "bt" => {
                    Box::new(btapi::bt_api_req(&data.ip).map(move |web_res| {
                        let body = Body::wrap_stream(web_res.into_body().map(move |b| {
                            let data_en: btapi::Btdata = serde_json::from_slice(&b).unwrap();
                            let ip_data = btapi::Ipdata::new(data_en,&data.ip);
                            let json = serde_json::to_string(&ip_data).unwrap();
                            Chunk::from(json)
                        }));
                        let mut response = Response::builder()
                            .status(StatusCode::OK)
                            .header(header::CONTENT_TYPE, "application/json")
                            .body(body).unwrap();
                                        response.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
            response.headers_mut().insert(header::ACCESS_CONTROL_ALLOW_HEADERS, "Origin, X-Requested-With, Content-Type, Accept".parse().unwrap());
                        response
                    }))
                },
                _ => {
                        let response = Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(Body::empty()).unwrap();
                        Box::new(future::ok(response))
                }
            };
            res

        })
    )
}


pub fn index_get(req: Request<Body>, remote_addr: String) -> ResponseFuture {
    Box::new(btapi::bt_api_req(&remote_addr).map(move |web_res| {
        Body::wrap_stream(web_res.into_body().map(move |b| {
            let data: btapi::Btdata = serde_json::from_slice(&b).unwrap();
            let ip_data = btapi::Ipdata::new(data,&remote_addr);
            let address = ip_data.l1 + " " + &ip_data.l2 + " " + &ip_data.l3;
            Chunk::from(format!(
                "btapi :\n\nIP     : {}\nAS号码 : {}\n地址   ：{}\n运营商 : {}\n",
                remote_addr,ip_data.as_num,address,ip_data.isp
            ))
        }))
    }).and_then(move |chunk| {
        let response = Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(chunk).unwrap();
        Box::new(future::ok(response))
        })
    )
}
