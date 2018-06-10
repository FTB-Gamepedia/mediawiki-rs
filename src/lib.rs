// Copyright © 2016-2018, Peter Atashian

#![feature(try_trait)]

extern crate cookie;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use cookie::{
    CookieJar, ParseError as CookieError,
};
use reqwest::{
    Client, Method, StatusCode,
    Error as ReqwestError,
    header::{Cookie, SetCookie, UserAgent},
};
use serde_json::{
    Error as ParseError, Value as Json,
};
use std::{
    cell::RefCell,
    collections::HashMap,
    fs::File,
    io::Error as IoError,
    marker::PhantomData,
    option::NoneError,
    path::Path,
    thread::sleep,
    time::Duration,
};

pub mod tilesheet;
pub mod oredict;

#[derive(Debug)]
pub enum Error {
    Json(Json),
    Io(IoError),
    Parse(ParseError),
    Cookie(CookieError),
    Reqwest(ReqwestError),
    None,
}
impl From<Json> for Error {
    fn from(err: Json) -> Error {
        Error::Json(err)
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
impl From<ReqwestError> for Error {
    fn from(err: ReqwestError) -> Error {
        Error::Reqwest(err)
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
    client: Client,
}
impl Mediawiki {
    pub fn login_config(config: Config) -> Result<Mediawiki, Error> {
        let mw = Mediawiki {
            cookies: RefCell::new(CookieJar::new()),
            config: config,
            client: Client::new(),
        };
        mw.login(None)?;
        Ok(mw)
    }
    pub fn login_path<P: AsRef<Path>>(path: P) -> Result<Mediawiki, Error> {
        let file = File::open(path)?;
        let config: Config = serde_json::from_reader(file)?;
        Mediawiki::login_config(config)
    }
    pub fn login(&self, token: Option<&str>) -> Result<(), Error> {
        let mut request = self.request();
        request
            .arg("action", "login")
            .arg("lgname", self.config.username.clone())
            .arg("lgpassword", self.config.password.clone());
        if let Some(token) = token {
            request.arg("lgtoken", token);
        }
        let json = request.post()?;
        let inner = json.get("login")?;
        let result = inner.get("result")?.as_str()?;
        match result {
            "NeedToken" => self.login(Some(inner.get("token")?.as_str()?)),
            "Success" => {
                println!("Logged in to MediaWiki");
                Ok(())
            },
            _ => Err(json.clone().into()),
        }
    }
    pub fn request(&self) -> RequestBuilder {
        RequestBuilder::new(self)
    }
    pub fn get_token<T>(&self) -> Result<Token<T>, Error> where T: TokenType {
        let json = self.request()
            .arg("action", "query")
            .arg("meta", "tokens")
            .arg("type", T::in_type())
            .get()?;
        Ok(Token::new(json.get("query")?.get("tokens")?.get(T::out_type())?.as_str()?))
    }
    pub fn query<T: Into<String>>(&self, list: T) -> QueryBuilder {
        let list = list.into();
        let mut request = self.request();
        request.arg("action", "query");
        request.arg("continue", "");
        request.arg("list", &*list);
        QueryBuilder {
            req: request,
            list: list,
        }
    }
    pub fn query_recentchanges(&self, limit: u32) -> QueryBuilder {
        let mut query = self.query("recentchanges");
        query.arg("rcdir", "older");
        query.arg("rcprop", "user|userid|comment|timestamp|title|ids|sha1|sizes|redirect|loginfo|tags|flags");
        query.arg("limit", limit.to_string());
        query
    }
}
pub struct RequestBuilder<'a> {
    mw: &'a Mediawiki,
    args: HashMap<String, String>,
}
impl<'a> RequestBuilder<'a> {
    fn new(mw: &'a Mediawiki) -> RequestBuilder<'a> {
        let mut request = RequestBuilder {
            mw: mw,
            args: HashMap::new(),
        };
        request.arg("format", "json");
        request
    }
    pub fn arg<T, U>(&mut self, key: T, val: U) -> &mut Self where T: Into<String>, U: Into<String> {
        self.args.insert(key.into(), val.into());
        self
    }
    fn request(&self, method: Method) -> Result<Json, Error> {
        let response = loop {
            let mut request = self.mw.client.request(method.clone(), &self.mw.config.baseapi);
            request.header(UserAgent::new(self.mw.config.useragent.clone()));
            let mut cookies = Cookie::new();
            for cookie in self.mw.cookies.borrow().iter() {
                cookies.append(cookie.name().to_owned(), cookie.value().to_owned());
            }
            request.header(cookies);
            match method {
                Method::Get => request.query(&self.args),
                Method::Post => request.form(&self.args),
                _ => unreachable!(),
            };
            let mut response = request.send()?;
            if let Some(cookies) = response.headers().get::<SetCookie>() {
                for cookie in cookies.iter() {
                    self.mw.cookies.borrow_mut()
                        .add(cookie::Cookie::parse(&**cookie)?.into_owned());
                }
            }
            if response.status() == StatusCode::Ok {
                break response;
            }
            println!("{:?}", response);
            sleep(Duration::from_secs(5))
        };
        let json: Json = serde_json::from_reader(response)?;
        Ok(json)
    }
    fn post(&self) -> Result<Json, Error> {
        self.request(Method::Post)
    }
    fn get(&self) -> Result<Json, Error> {
        self.request(Method::Get)
    }
}
pub struct QueryBuilder<'a> {
    req: RequestBuilder<'a>,
    list: String,
}
impl<'a> QueryBuilder<'a> {
    pub fn arg<T, U>(&mut self, key: T, val: U) -> &mut Self where T: Into<String>, U: Into<String> {
        self.req.arg(key, val);
        self
    }
}
impl<'a> IntoIterator for QueryBuilder<'a> {
    type Item = Result<Json, Error>;
    type IntoIter = Query<'a>;
    fn into_iter(self) -> Query<'a> {
        Query {
            req: self.req,
            list: self.list,
            buf: Vec::new(),
            done: false,
        }
    }
}
pub struct Query<'a> {
    req: RequestBuilder<'a>,
    list: String,
    buf: Vec<Json>,
    done: bool,
}
impl<'a> Query<'a> {
    fn fill(&mut self) -> Result<bool, Error> {
        let json = self.req.get()?;
        let buf = json.get("query")?.get(&self.list)?;
        self.buf.clone_from(buf.as_array()?);
        self.buf.reverse();
        if let Some(cont) = json.get("continue") {
            for (key, val) in cont.as_object()? {
                self.req.arg(&**key, val.as_str()?);
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
