use pretty_env_logger;
use hyper::{Server,Request,Response,Client};
use hyper::{Body,Method,StatusCode,header,HeaderMap};
use hyper::client::connect::HttpInfo;
use std::net::SocketAddr;


pub fn Ipfromrequerst(req: &Request<Body>,remote_addr: String) -> Result<String,hyper::Error> {
    // let addr = remote_addr.ip().to_string();
    if req.headers().contains_key("x-forwarded-for"){
        let ip_addr = req.headers().get("x-forwarded-for").unwrap();
        Ok(ip_addr.to_str().unwrap().to_string())
    } else { 
        Ok(remote_addr)
    }
}