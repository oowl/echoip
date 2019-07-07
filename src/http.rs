use pretty_env_logger;
use hyper::{Server,Request,Response,Client};
use hyper::{Body,Method,StatusCode};
use hyper::client::connect::HttpInfo;
use std::net::SocketAddr;


pub fn Ipfromrequerst(req: Request<Body>) -> Result<SocketAddr,hyper::Error> {
    let addr = req.extensions().get::<HttpInfo>().expect("something something sets HttpInfo").remote_addr();
    Ok(addr)
}