// Copyright © 2014, Peter Atashian

extern crate cookie;
extern crate hyper;
extern crate rustc_serialize;
extern crate url;

use cookie::{CookieJar};
use rustc_serialize::json::{Array, Json};

#[derive(Debug)]
enum Error {
    Json(Json),
}
impl<'a> From<Json> for Error {
    fn from(err: Json) -> Error {
        Error::Json(err)
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
pub struct Config {
    useragent: String,
    username: String,
    password: String,
    baseapi: String,
}
pub struct Mediawiki {
    cookies: CookieJar<'static>,
    config: Config,
    useragent: String,
}
