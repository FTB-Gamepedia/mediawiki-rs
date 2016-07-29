// Copyright © 2016, Peter Atashian

extern crate cookie;
extern crate hyper;
extern crate rustc_serialize;
extern crate url;

use cookie::{CookieJar};
use hyper::{Url};
use hyper::client::request::{Request};
use hyper::client::response::{Response};
use hyper::error::Error as HyperError;
use hyper::header::{ContentType, Cookie, SetCookie, UserAgent};
use hyper::method::{Method};
use hyper::status::{StatusCode};
use rustc_serialize::json::{Array, DecoderError, Json, Object, ParserError, decode};
use std::borrow::{Borrow};
use std::cell::{RefCell};
use std::collections::{HashMap};
use std::fs::{File};
use std::io::{Error as IoError, Read, Write};
use std::marker::{PhantomData};
use std::path::{Path};
use std::thread::{sleep};
use std::time::{Duration};
use url::ParseError as UrlError;
use url::form_urlencoded::{Serializer};

#[derive(Debug)]
pub enum Error {
    Json(Json),
    Decode(DecoderError),
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
impl From<DecoderError> for Error {
    fn from(err: DecoderError) -> Error {
        Error::Decode(err)
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

pub trait JsonFun<'a> {
    fn get(self, &str) -> Result<&'a Json, Error>;
    fn string(self) -> Result<&'a str, Error>;
    fn array(self) -> Result<&'a Array, Error>;
    fn integer(self) -> Result<i64, Error>;
    fn object(self) -> Result<&'a Object, Error>;
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
    fn object(self) -> Result<&'a Object, Error> {
        Ok(try!(self.as_object().ok_or(self.clone())))
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
    fn object(self) -> Result<&'a Object, Error> {
        self.and_then(|x| x.object())
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
    pub fn login(config: Config) -> Result<Mediawiki, Error> {
        let mw = Mediawiki {
            cookies: RefCell::new(CookieJar::new(&[])),
            config: config,
        };
        try!(mw.do_login(None));
        Ok(mw)
    }
    pub fn login_file<P: AsRef<Path>>(path: P) -> Result<Mediawiki, Error> {
        let mut config = try!(File::open(path));
        let mut s = String::new();
        try!(config.read_to_string(&mut s));
        let config = try!(decode(&s));
        Mediawiki::login(config)
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
        let mut resp = try!(self.post_request(&self.config.baseapi, &args));
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
    fn do_request(
        &self, url: Url, method: Method, body: Option<&str>,
    ) -> Result<Response, Error> {
        let mut request = try!(Request::new(method, url));
        try!(request.set_read_timeout(Some(Duration::from_secs(5))));
        request.headers_mut().set(UserAgent(self.config.useragent.clone()));
        request.headers_mut().set(Cookie::from_cookie_jar(&self.cookies.borrow()));
        if body.is_some() {
            request.headers_mut().set(ContentType("application/x-www-form-urlencoded".parse().unwrap()));
        }
        let mut request = try!(request.start());
        if let Some(body) = body {
            try!(request.write_all(body.as_bytes()));
        }
        let response = try!(request.send());
        if let Some(cookies) = response.headers.get::<SetCookie>() {
            cookies.apply_to_cookie_jar(&mut self.cookies.borrow_mut());
        }
        Ok(response)
    }
    fn get_request<I, K, V>(
        &self, base: &str, args: I,
    ) -> Result<Response, Error> where
        I: IntoIterator, I::Item: Borrow<(K, V)>, K: AsRef<str>, V: AsRef<str>,
    {
        let query = Serializer::new(String::new()).extend_pairs(args).finish();
        let url = try!(Url::parse(&format!("{}?{}", base, query)));
        loop {
            let r = try!(self.do_request(url.clone(), Method::Get, None));
            if r.status == StatusCode::Ok {
                return Ok(r)
            }
            println!("{:?}", r);
            sleep(Duration::from_secs(5))
        }
    }
    fn post_request<I, K, V>(
        &self, base: &str, args: I,
    ) -> Result<Response, Error> where
        I: IntoIterator, I::Item: Borrow<(K, V)>, K: AsRef<str>, V: AsRef<str>,
    {
        let query = Serializer::new(String::new()).extend_pairs(args).finish();
        let url = try!(Url::parse(base));
        loop {
            let r = try!(self.do_request(url.clone(), Method::Post, Some(&*query)));
            if r.status == StatusCode::Ok {
                return Ok(r)
            }
            println!("{:?}", r);
            sleep(Duration::from_secs(5))
        }
    }
    pub fn query_recentchanges(&self, limit: u32) -> Query {
        let mut args: HashMap<String, String> = [
            ("continue", ""), ("list", "recentchanges"), ("format", "json"),
            ("action", "query"), ("rcdir", "older"),
            ("rcprop", "user|userid|comment|timestamp|title|ids|sha1|sizes|redirect|loginfo|tags|flags"),
        ].iter().map(|&(a, b)| (a.into(), b.into())).collect();
        args.insert("limit".into(), limit.to_string());
        Query {
            mw: &self,
            name: "recentchanges".into(),
            buf: Vec::new(),
            args: args,
            done: false,
        }
    }
    pub fn get_token<T>(&self) -> Result<Token<T>, Error> where T: TokenType {
        let args = [("format", "json"), ("action", "query"),
            ("meta", "tokens"), ("type", T::in_type())];
        let mut resp = try!(self.get_request(&self.config.baseapi, &args));
        let mut body = String::new();
        try!(resp.read_to_string(&mut body));
        let json: Json = try!(Json::from_str(&body));
        json.get("query").get("tokens").get(T::out_type()).string().map(Token::new)
    }
    pub fn query_tiles(&self, modab: String) -> Query {
        let mut args: HashMap<String, String> = [
            ("format", "json"), ("action", "query"), ("continue", ""), ("list", "tiles"), ("tslimit", "5000"),
        ].iter().map(|&(a, b)| (a.into(), b.into())).collect();
        args.insert("tsmod".into(), modab);
        Query {
            mw: &self,
            name: "tiles".into(),
            buf: Vec::new(),
            args: args,
            done: false,
        }
    }
    pub fn delete_tiles(&self, token: &Token<Csrf>, ids: String) -> Result<Json, Error> {
        let args = [("format", "json"), ("action", "deletetiles"),
            ("tstoken", &*token.0), ("tsids", &*ids)];
        let mut resp = try!(self.post_request(&self.config.baseapi, &args));
        let mut body = String::new();
        try!(resp.read_to_string(&mut body));
        let json: Json = try!(Json::from_str(&body));
        Ok(json)
    }
    pub fn add_tile(
        &self, token: &Token<Csrf>, tsmod: &str, tsname: &str, tsx: u32, tsy: u32,
    ) -> Result<Json, Error> {
        let (x, y) = (tsx.to_string(), tsy.to_string());
        let args = [
            ("format", "json"), ("action", "addtile"), ("tstoken", &*token.0), ("tsmod", tsmod),
            ("tsname", tsname), ("tsx", &*x), ("tsy", &*y),
        ];
        let mut resp = try!(self.post_request(&self.config.baseapi, &args));
        let mut body = String::new();
        try!(resp.read_to_string(&mut body));
        let json: Json = try!(Json::from_str(&body));
        Ok(json)
    }
    pub fn create_sheet(
        &self, token: &Token<Csrf>, tsmod: &str, tssizes: &str
    ) -> Result<Json, Error> {
        let args = [
            ("format", "json"), ("action", "createsheet"), ("tstoken", &*token.0),
            ("tsmod", tsmod), ("tssizes", tssizes),
        ];
        let mut resp = try!(self.post_request(&self.config.baseapi, &args));
        let mut body = String::new();
        try!(resp.read_to_string(&mut body));
        let json: Json = try!(Json::from_str(&body));
        Ok(json)
    }
}
pub struct Query<'a> {
    mw: &'a Mediawiki,
    name: String,
    buf: Vec<Json>,
    args: HashMap<String, String>,
    done: bool,
}
impl<'a> Query<'a> {
    fn fill(&mut self) -> Result<bool, Error> {
        let mut resp = try!(self.mw.get_request(&self.mw.config.baseapi, &self.args));
        let mut body = String::new();
        try!(resp.read_to_string(&mut body));
        let json: Json = try!(Json::from_str(&body));
        let buf = try!(json.get("query").get(&self.name));
        self.buf.clone_from(try!(buf.array()));
        self.buf.reverse();
        if let Ok(cont) = json.get("continue") {
            for (key, val) in try!(cont.object()) {
                self.args.insert(key.clone(), try!(val.string()).into());
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }
}
impl<'a> Iterator for Query<'a> {
    type Item = Result<Json, Error>;
    fn next(&mut self) -> Option<Result<Json, Error>> {
        if self.buf.is_empty() {
            if self.done { return None }
            match self.fill() {
                Err(e) => return Some(Err(e)),
                Ok(false) => self.done = true,
                Ok(true) => (),
            }
        }
        self.buf.pop().map(|c| Ok(c))
    }
}
pub trait TokenType {
    fn in_type() -> &'static str;
    fn out_type() -> &'static str;
}
#[derive(Debug)] pub struct Token<T>(String, PhantomData<T>);
impl<T> Token<T> {
    fn new(token: &str) -> Token<T> {
        Token(token.to_owned(), PhantomData)
    }
}
#[derive(Debug)] pub struct Csrf;
impl TokenType for Csrf {
    fn in_type() -> &'static str { "csrf" }
    fn out_type() -> &'static str { "csrftoken" }
}
#[derive(Debug)] pub struct Watch;
impl TokenType for Watch {
    fn in_type() -> &'static str { "watch" }
    fn out_type() -> &'static str { "watchtoken" }
}
#[derive(Debug)] pub struct Patrol;
impl TokenType for Patrol {
    fn in_type() -> &'static str { "patrol" }
    fn out_type() -> &'static str { "patroltoken" }
}
#[derive(Debug)] pub struct Rollback;
impl TokenType for Rollback {
    fn in_type() -> &'static str { "rollback" }
    fn out_type() -> &'static str { "rollbacktoken" }
}
#[derive(Debug)] pub struct UserRights;
impl TokenType for UserRights {
    fn in_type() -> &'static str { "userrights" }
    fn out_type() -> &'static str { "userrightstoken" }
}
