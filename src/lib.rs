use cookie::{Cookie, CookieJar, ParseError as CookieError};
use reqwest::{
    blocking::{multipart::Form, Client},
    header::{COOKIE, SET_COOKIE, USER_AGENT},
    Error as ReqwestError, Method, StatusCode,
};
use serde::Deserialize;
use serde_json::{Error as ParseError, Value as Json};
use std::{
    cell::RefCell,
    collections::HashMap,
    fs::File,
    io::{Error as IoError, Read},
    marker::PhantomData,
    path::Path,
    thread::sleep,
    time::Duration,
};

pub mod oredict;
pub mod tilesheet;

#[derive(Debug)]
pub enum Error {
    Json(Json),
    Io(IoError),
    Parse(ParseError),
    Cookie(CookieError),
    Reqwest(ReqwestError),
    Status(String),
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
impl From<String> for Error {
    fn from(err: String) -> Error {
        Error::Status(err)
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
            config,
            client: Client::new(),
        };
        if !mw.config.username.is_empty() {
            mw.login()?;
        }
        Ok(mw)
    }
    pub fn login_path<P: AsRef<Path>>(path: P) -> Result<Mediawiki, Error> {
        let file = File::open(path)?;
        let config: Config = serde_json::from_reader(file)?;
        Mediawiki::login_config(config)
    }
    pub fn login(&self) -> Result<(), Error> {
        let token = self.get_token::<Login>()?;
        let mut request = self.request();
        request
            .arg("action", "login")
            .arg("lgname", self.config.username.clone())
            .arg("lgpassword", self.config.password.clone())
            .arg("lgtoken", token.value());
        let json = request.post()?;
        let result = json["login"]["result"]
            .as_str()
            .ok_or_else(|| Error::Json(json.clone()))?;
        match result {
            "Success" => {
                println!("Logged in to MediaWiki");
                Ok(())
            }
            _ => Err(json.clone().into()),
        }
    }
    pub fn request(&self) -> RequestBuilder {
        RequestBuilder::new(self)
    }
    pub fn get_token<T>(&self) -> Result<Token<T>, Error>
    where
        T: TokenType,
    {
        let json = self
            .request()
            .arg("action", "query")
            .arg("meta", "tokens")
            .arg("type", T::in_type())
            .get()?;
        Ok(Token::new(
            json["query"]["tokens"][T::out_type()]
                .as_str()
                .ok_or_else(|| Error::Json(json.clone()))?,
        ))
    }
    pub fn query<T: Into<String>>(&self, list: T) -> QueryBuilder {
        let list = list.into();
        let mut request = self.request();
        request.arg("action", "query");
        request.arg("continue", "");
        QueryBuilder { req: request, list }
    }
    pub fn query_recentchanges(&self, limit: u32) -> QueryBuilder {
        let mut query = self.query("recentchanges");
        query.arg("list", "recentchanges");
        query.arg("rcdir", "older");
        query.arg(
            "rcprop",
            "user|userid|comment|timestamp|title|ids|sha1|sizes|redirect|loginfo|tags|flags",
        );
        query.arg("limit", limit.to_string());
        query
    }
    pub fn download_file(&self, name: &str) -> Result<Option<Vec<u8>>, Error> {
        let mut request = self.request();
        request.arg("action", "query");
        request.arg("prop", "imageinfo");
        request.arg("titles", format!("File:{name}"));
        request.arg("iiprop", "url");
        let json = request.get()?;
        let images = json["query"]["pages"]
            .as_array()
            .ok_or_else(|| Error::Json(json.clone()))?;
        let image = images.first().ok_or_else(|| Error::Json(json.clone()))?;
        if !image["missing"].is_null() {
            return Ok(None);
        }
        let url = image["imageinfo"][0]["url"]
            .as_str()
            .ok_or_else(|| Error::Json(image.clone()))?;
        let mut response = loop {
            let mut request = self.client.request(Method::GET, url);
            request = request.header(USER_AGENT, &*self.config.useragent);
            let response = request.send()?;
            if response.status() == StatusCode::OK {
                break response;
            }
            println!("{response:?}");
            sleep(Duration::from_secs(5))
        };
        let mut buf = Vec::new();
        response.read_to_end(&mut buf)?;
        Ok(Some(buf))
    }
    pub fn upload(
        &self,
        filename: &str,
        token: &Token<Csrf>,
        file: Upload,
        text: Option<&str>,
        comment: Option<&str>,
        ignorewarnings: bool,
    ) -> Result<Json, Error> {
        let request = self.request();
        let mut form = Form::new()
            .text("format", "json")
            .text("action", "upload")
            .text("filename", filename.to_owned())
            .text("token", token.0.clone());
        if let Some(text) = text {
            form = form.text("text", text.to_owned());
        }
        if let Some(comment) = comment {
            form = form.text("comment", comment.to_owned());
        }
        if ignorewarnings {
            form = form.text("ignorewarnings", "true");
        }
        form = match file {
            Upload::File(file) => form.file("file", file)?,
            Upload::Filekey(filekey) => form.text("filekey", filekey.to_owned()),
            Upload::Url(url) => form.text("url", url.to_owned()),
        };
        request.multipart(form)
    }
}
#[derive(Clone)]
pub struct RequestBuilder<'a> {
    mw: &'a Mediawiki,
    args: HashMap<String, String>,
}
impl<'a> RequestBuilder<'a> {
    fn new(mw: &'a Mediawiki) -> RequestBuilder<'a> {
        let mut request = RequestBuilder {
            mw,
            args: HashMap::new(),
        };
        request.arg("format", "json");
        request.arg("formatversion", "2");
        request
    }
    pub fn arg<T, U>(&mut self, key: T, val: U) -> &mut Self
    where
        T: Into<String>,
        U: Into<String>,
    {
        self.args.insert(key.into(), val.into());
        self
    }
    pub fn argo<T, U>(&mut self, key: T, val: Option<U>) -> &mut Self
    where
        T: Into<String>,
        U: Into<String>,
    {
        if let Some(val) = val {
            self.args.insert(key.into(), val.into());
        }
        self
    }
    fn request(&self, method: Method, multipart: Option<Form>) -> Result<Json, Error> {
        let mut request = self
            .mw
            .client
            .request(method.clone(), &self.mw.config.baseapi);
        request = request.header(USER_AGENT, &*self.mw.config.useragent);
        let cookies = self
            .mw
            .cookies
            .borrow()
            .iter()
            .map(|cookie| format!("{}={}", cookie.name(), cookie.value()))
            .collect::<Vec<_>>();
        request = request.header(COOKIE, cookies.join("; "));
        request = match (multipart, method) {
            (Some(multipart), Method::POST) => request.multipart(multipart),
            (None, Method::GET) => request.query(&self.args),
            (None, Method::POST) => request.form(&self.args),
            _ => unreachable!(),
        };
        //let request = request.build()?;
        //println!("{:?}", request);
        //println!("{:?}", request.body());
        //let mut response = self.mw.client.execute(request)?;
        let response = request.send()?;
        for cookie in response.headers().get_all(SET_COOKIE) {
            self.mw
                .cookies
                .borrow_mut()
                .add(Cookie::parse(String::from_utf8_lossy(cookie.as_bytes()))?.into_owned());
        }
        let status = response.status();
        let text = response.text()?;
        if status.is_success() {
            let json: Json = serde_json::from_str(&text)?;
            if json["error"].is_object() {
                Err(Error::Json(json))
            } else {
                Ok(json)
            }
        } else {
            println!("{text:?}");
            Err(status.to_string().into())
        }
    }
    pub fn post(&self) -> Result<Json, Error> {
        loop {
            match self.request(Method::POST, None) {
                Ok(json) => return Ok(json),
                Err(Error::Json(err)) => return Err(Error::Json(err)),
                Err(status) => println!("{status:?}"),
            }
        }
    }
    pub fn get(&self) -> Result<Json, Error> {
        loop {
            match self.request(Method::GET, None) {
                Ok(json) => return Ok(json),
                Err(status) => println!("{status:?}"),
            }
        }
    }
    fn multipart(&self, multipart: Form) -> Result<Json, Error> {
        self.request(Method::POST, Some(multipart))
    }
}
pub enum Upload<'a> {
    File(&'a Path),
    Filekey(&'a str),
    Url(&'a str),
}
#[derive(Clone)]
pub struct QueryBuilder<'a> {
    req: RequestBuilder<'a>,
    list: String,
}
impl<'a> QueryBuilder<'a> {
    pub fn arg<T, U>(&mut self, key: T, val: U) -> &mut Self
    where
        T: Into<String>,
        U: Into<String>,
    {
        self.req.arg(key, val);
        self
    }
    pub fn argo<T, U>(&mut self, key: T, val: Option<U>) -> &mut Self
    where
        T: Into<String>,
        U: Into<String>,
    {
        self.req.argo(key, val);
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
        let buf = json["query"][&self.list]
            .as_array()
            .ok_or_else(|| Error::Json(json.clone()))?;
        self.buf.clone_from(buf);
        self.buf.reverse();
        if let Json::Object(cont) = &json["continue"] {
            for (key, val) in cont {
                self.req
                    .arg(key, val.as_str().ok_or_else(|| Error::Json(json.clone()))?);
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
            if self.done {
                return None;
            }
            match self.fill() {
                Err(e) => return Some(Err(e)),
                Ok(false) => self.done = true,
                Ok(true) => (),
            }
        }
        self.buf.pop().map(Ok)
    }
}
pub trait TokenType {
    fn in_type() -> &'static str;
    fn out_type() -> &'static str;
}
#[derive(Debug)]
pub struct Token<T>(String, PhantomData<T>);
impl<T> Token<T> {
    fn new(token: &str) -> Token<T> {
        Token(token.to_owned(), PhantomData)
    }
    fn value(&self) -> &str {
        &self.0
    }
}
#[derive(Debug)]
pub struct CreateAccount;
impl TokenType for CreateAccount {
    fn in_type() -> &'static str {
        "createaccount"
    }
    fn out_type() -> &'static str {
        "createaccounttoken"
    }
}
#[derive(Debug)]
pub struct Csrf;
impl TokenType for Csrf {
    fn in_type() -> &'static str {
        "csrf"
    }
    fn out_type() -> &'static str {
        "csrftoken"
    }
}
#[derive(Debug)]
pub struct Login;
impl TokenType for Login {
    fn in_type() -> &'static str {
        "login"
    }
    fn out_type() -> &'static str {
        "logintoken"
    }
}
#[derive(Debug)]
pub struct Patrol;
impl TokenType for Patrol {
    fn in_type() -> &'static str {
        "patrol"
    }
    fn out_type() -> &'static str {
        "patroltoken"
    }
}
#[derive(Debug)]
pub struct Rollback;
impl TokenType for Rollback {
    fn in_type() -> &'static str {
        "rollback"
    }
    fn out_type() -> &'static str {
        "rollbacktoken"
    }
}
#[derive(Debug)]
pub struct UserRights;
impl TokenType for UserRights {
    fn in_type() -> &'static str {
        "userrights"
    }
    fn out_type() -> &'static str {
        "userrightstoken"
    }
}
#[derive(Debug)]
pub struct Watch;
impl TokenType for Watch {
    fn in_type() -> &'static str {
        "watch"
    }
    fn out_type() -> &'static str {
        "watchtoken"
    }
}
