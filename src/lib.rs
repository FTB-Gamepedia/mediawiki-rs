// Copyright © 2016, Peter Atashian

#![feature(try_trait)]

extern crate cookie;
extern crate hyper;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate url;

use cookie::{CookieJar, ParseError as CookieError};
use hyper::{Url};
use hyper::client::request::{Request};
use hyper::client::response::{Response};
use hyper::error::Error as HyperError;
use hyper::header::{ContentType, Cookie, SetCookie, UserAgent};
use hyper::method::{Method};
use hyper::status::{StatusCode};
use serde_json::{Error as ParseError, Value as Json};
use std::borrow::{Borrow};
use std::cell::{RefCell};
use std::collections::{HashMap};
use std::fs::{File};
use std::io::{Error as IoError, Write};
use std::marker::{PhantomData};
use std::option::NoneError;
use std::path::{Path};
use std::thread::{sleep};
use std::time::{Duration};
use url::ParseError as UrlError;
use url::form_urlencoded::{Serializer};

#[derive(Debug)]
pub enum Error {
    Json(Json),
    Url(UrlError),
    Hyper(HyperError),
    Io(IoError),
    Parse(ParseError),
    Cookie(CookieError),
    None,
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
impl From<ParseError> for Error {
    fn from(err: ParseError) -> Error {
        Error::Parse(err)
    }
}
impl From<CookieError> for Error {
    fn from(err: CookieError) -> Error {
        Error::Cookie(err)
    }
}
impl From<NoneError> for Error {
    fn from(_: NoneError) -> Error {
        Error::None
    }
}

#[derive(Deserialize)]
pub struct Config {
    useragent: String,
    username: String,
    password: String,
    baseapi: String,
}
pub struct Mediawiki {
    cookies: RefCell<CookieJar>,
    config: Config,
}
impl Mediawiki {
    pub fn login(config: Config) -> Result<Mediawiki, Error> {
        let mw = Mediawiki {
            cookies: RefCell::new(CookieJar::new()),
            config: config,
        };
        mw.do_login(None)?;
        Ok(mw)
    }
    pub fn login_file<P: AsRef<Path>>(path: P) -> Result<Mediawiki, Error> {
        let file = File::open(path)?;
        let config: Config = serde_json::from_reader(file)?;
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
        let resp = self.post_request(&self.config.baseapi, &args)?;
        let json: Json = serde_json::from_reader(resp)?;
        let inner = json.get("login")?;
        let result = inner.get("result")?.as_str()?;
        match result {
            "NeedToken" => self.do_login(Some(inner.get("token")?.as_str()?)),
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
        let mut request = Request::new(method, url)?;
        request.set_read_timeout(Some(Duration::from_secs(60)))?;
        request.headers_mut().set(UserAgent(self.config.useragent.clone()));
        request.headers_mut().set(Cookie(self.cookies.borrow().iter().map(|cookie| {
            format!("{}", cookie)
        }).collect()));
        if body.is_some() {
            request.headers_mut().set(ContentType("application/x-www-form-urlencoded".parse().unwrap()));
        }
        let mut request = request.start()?;
        if let Some(body) = body {
            request.write_all(body.as_bytes())?;
        }
        let response = request.send()?;
        if let Some(cookies) = response.headers.get::<SetCookie>() {
            for cookie in &cookies.0 {
                self.cookies.borrow_mut().add(cookie::Cookie::parse(&**cookie)?.into_owned());
            }
        }
        Ok(response)
    }
    fn get_request<I, K, V>(
        &self, base: &str, args: I,
    ) -> Result<Response, Error> where
        I: IntoIterator, I::Item: Borrow<(K, V)>, K: AsRef<str>, V: AsRef<str>,
    {
        let query = Serializer::new(String::new()).extend_pairs(args).finish();
        let url = Url::parse(&format!("{}?{}", base, query))?;
        loop {
            let r = self.do_request(url.clone(), Method::Get, None)?;
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
        let url = Url::parse(base)?;
        loop {
            let r = self.do_request(url.clone(), Method::Post, Some(&*query))?;
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
        let resp = try!(self.get_request(&self.config.baseapi, &args));
        let json: Json = serde_json::from_reader(resp)?;
        Ok(Token::new(json.get("query")?.get("tokens")?.get(T::out_type())?.as_str()?))
    }
    pub fn query_tiles(&self, tsmod: Option<&str>) -> Query {
        let mut args: HashMap<String, String> = [
            ("format", "json"), ("action", "query"), ("continue", ""), ("list", "tiles"), ("tslimit", "5000"),
        ].iter().map(|&(a, b)| (a.into(), b.into())).collect();
        if let Some(tsmod) = tsmod {
            args.insert("tsmod".into(), tsmod.into());
        }
        Query {
            mw: &self,
            name: "tiles".into(),
            buf: Vec::new(),
            args: args,
            done: false,
        }
    }
    pub fn delete_tiles(&self, token: &Token<Csrf>, ids: &str) -> Result<Json, Error> {
        let args = [
            ("format", "json"), ("action", "deletetiles"),
            ("tstoken", &*token.0), ("tsids", ids),
        ];
        let resp = try!(self.post_request(&self.config.baseapi, &args));
        let json: Json = serde_json::from_reader(resp)?;
        Ok(json)
    }
    pub fn add_tiles(
        &self, token: &Token<Csrf>, tsmod: &str, tsimport: &str,
    ) -> Result<Json, Error> {
        let args = [
            ("format", "json"), ("action", "addtiles"), ("tstoken", &*token.0), ("tsmod", tsmod),
            ("tsimport", tsimport),
        ];
        let resp = try!(self.post_request(&self.config.baseapi, &args));
        let json: Json = serde_json::from_reader(resp)?;
        Ok(json)
    }
    pub fn create_sheet(
        &self, token: &Token<Csrf>, tsmod: &str, tssizes: &str
    ) -> Result<Json, Error> {
        let args = [
            ("format", "json"), ("action", "createsheet"), ("tstoken", &*token.0),
            ("tsmod", tsmod), ("tssizes", tssizes),
        ];
        let resp = try!(self.post_request(&self.config.baseapi, &args));
        let json: Json = serde_json::from_reader(resp)?;
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
        let resp = self.mw.get_request(&self.mw.config.baseapi, &self.args)?;
        let json: Json = serde_json::from_reader(resp)?;
        let buf = json.get("query")?.get(&self.name)?;
        self.buf.clone_from(buf.as_array()?);
        self.buf.reverse();
        if let Some(cont) = json.get("continue") {
            for (key, val) in cont.as_object()? {
                self.args.insert(key.clone(), val.as_str()?.into());
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
