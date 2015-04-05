// Copyright © 2014, Peter Atashian

extern crate cookie;
extern crate hyper;
extern crate rustc_serialize;
extern crate url;

use cookie::{CookieJar};
use hyper::{Url};
use hyper::client::request::{Request};
use hyper::client::response::{Response};
use hyper::error::Error as HyperError;
use hyper::header::{Cookie, SetCookie, UserAgent};
use hyper::method::{Method};
use hyper::status::{StatusCode};
use rustc_serialize::json::{Array, Json, ParserError};
use std::cell::{RefCell};
use std::io::{Read};
use std::io::Error as IoError;
use url::ParseError as UrlError;

#[derive(Debug)]
pub enum Error {
    Json(Json),
    Url(UrlError),
    Hyper(HyperError),
    Io(IoError),
    Parse(ParserError),
}
impl From<Json> for Error {
    fn from(err: Json) -> Error {
        Error::Json(err)
    }
}
impl From<UrlError> for Error {
    fn from(err: UrlError) -> Error {
        Error::Url(err)
    }
}
impl From<HyperError> for Error {
    fn from(err: HyperError) -> Error {
        Error::Hyper(err)
    }
}
impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Error::Io(err)
    }
}
impl From<ParserError> for Error {
    fn from(err: ParserError) -> Error {
        Error::Parse(err)
    }
}

trait JsonFun<'a> {
    fn get(self, &str) -> Result<&'a Json, Error>;
    fn string(self) -> Result<&'a str, Error>;
    fn array(self) -> Result<&'a Array, Error>;
    fn integer(self) -> Result<i64, Error>;
}
impl<'a> JsonFun<'a> for &'a Json {
    fn get(self, s: &str) -> Result<&'a Json, Error> {
        Ok(try!(self.find(s).ok_or(self.clone())))
    }
    fn string(self) -> Result<&'a str, Error> {
        Ok(try!(self.as_string().ok_or(self.clone())))
    }
    fn array(self) -> Result<&'a Array, Error> {
        Ok(try!(self.as_array().ok_or(self.clone())))
    }
    fn integer(self) -> Result<i64, Error> {
        Ok(try!(self.as_i64().ok_or(self.clone())))
    }
}
impl<'a> JsonFun<'a> for Result<&'a Json, Error> {
    fn get(self, s: &str) -> Result<&'a Json, Error> {
        self.and_then(|x| x.get(s))
    }
    fn string(self) -> Result<&'a str, Error> {
        self.and_then(|x| x.string())
    }
    fn array(self) -> Result<&'a Array, Error> {
        self.and_then(|x| x.array())
    }
    fn integer(self) -> Result<i64, Error> {
        self.and_then(|x| x.integer())
    }
}
#[derive(RustcDecodable)]
pub struct Config {
    useragent: String,
    username: String,
    password: String,
    baseapi: String,
}
pub struct Mediawiki {
    cookies: RefCell<CookieJar<'static>>,
    config: Config,
}
impl Mediawiki {
    fn do_request(
        &self, url: Url, method: Method,
    ) -> Result<Response, Error> {
        let mut request = try!(Request::new(method, url));
        request.headers_mut().set(UserAgent(self.config.useragent.clone()));
        request.headers_mut().set(Cookie::from_cookie_jar(&self.cookies.borrow()));
        let response = try!(try!(request.start()).send());
        if let Some(cookies) = response.headers.get::<SetCookie>() {
            cookies.apply_to_cookie_jar(&mut self.cookies.borrow_mut());
        }
        Ok(response)
    }
    fn request(
        &self, base: &str, args: &[(&str, &str)], method: Method,
    ) -> Result<Response, Error> {
        use std::thread::sleep_ms;
        use url::form_urlencoded::{serialize};
        let url = try!(Url::parse(&format!("{}?{}", base, serialize(args.iter().map(|&x| x)))));
        loop {
            let r = try!(self.do_request(url.clone(), method.clone()));
            if r.status == StatusCode::Ok {
                return Ok(r)
            }
            println!("{:?}", r);
            sleep_ms(1000)
        }
    }
    fn do_login(&self, token: Option<&str>) -> Result<(), Error> {
        let mut args = vec![
            ("format", "json"),
            ("action", "login"),
            ("lgname", &self.config.username),
            ("lgpassword", &self.config.password)];
        if let Some(token) = token {
            args.push(("lgtoken", token));
        }
        let mut resp = try!(self.request(&self.config.baseapi, &args, Method::Post));
        let mut body = String::new();
        try!(resp.read_to_string(&mut body));
        let json: Json = try!(Json::from_str(&body));
        let inner = try!(json.get("login"));
        let result = try!(inner.get("result").string());
        match result {
            "NeedToken" => self.do_login(Some(try!(inner.get("token").string()))),
            "Success" => {
                println!("Logged in to MediaWiki");
                Ok(())
            },
            _ => Err(json.clone().into()),
        }
    }
    pub fn get_rc(&self) -> Result<(Vec<Json>, String), Error> {
        let args = vec![
            ("format", "json"), ("action", "query"),
            ("list", "recentchanges"), ("rclimit", "1"),
            ("rcprop", "user|userid|comment|parsedcomment|flags|timestamp|title|\
            ids|sizes|redirect|patrolled|loginfo|tags|sha1"), ("rcdir", "older")];
        let mut resp = try!(self.request(&self.config.baseapi, &args, Method::Get));
        let mut body = String::new();
        try!(resp.read_to_string(&mut body));
        let json: Json = try!(Json::from_str(&body));
        let rc = try!(json.get("query").get("recentchanges").array());
        let cont = try!(json.get("query-continue").get("recentchanges")
            .get("rccontinue").string());
        Ok((rc.clone(), cont.to_owned()))
    }
    pub fn login(config: Config) -> Result<Mediawiki, Error> {
        let mw = Mediawiki {
            cookies: RefCell::new(CookieJar::new(&[])),
            config: config,
        };
        try!(mw.do_login(None));
        Ok(mw)
    }
}
