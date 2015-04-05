// Copyright © 2014, Peter Atashian

use cookie::Cookie;
use hyper::Url;
use hyper::client::request::Request;
use hyper::header::common::{
    Cookies,
    SetCookie,
    UserAgent,
};
use hyper::method::Method;
use hyper::net::Fresh;
use serialize::json::{
    decode,
};
use std::io::fs::File;
use url::form_urlencoded::serialize;

fn make_url(args: &[(&str, &str)]) -> String {
    format!("http://ftb.gamepedia.com/api.php?{}", serialize(args.iter().map(|&x| x)))
}

struct WikiApi {
    cookies: Vec<Cookie>,
}
impl Mediawiki {
    fn make_request(&self, url: &str, method: Method) -> Request<Fresh> {
        let mut request = Request::new(method, Url::parse(url).unwrap()).unwrap();
        request.headers_mut().set(UserAgent("PonyButt".into_string()));
        request.headers_mut().set(Cookies(self.cookies.clone()));
        request
    }
    fn login_first(username: &str, password: &str) -> (Vec<Cookie>, String) {
        #[deriving(Decodable, Show)]
        struct JsonLogin {
            login: JsonLoginInner,
        }
        #[deriving(Decodable, Show)]
        struct JsonLoginInner {
            result: String,
            token: String,
            cookieprefix: String,
            sessionid: String,
        }
        let url = make_url(&[("format", "json"), ("action", "login"), ("lgname", username), ("lgpassword", password)]);
        let mut request = Request::new(Method::Post, Url::parse(url[]).unwrap()).unwrap();
        request.headers_mut().set(UserAgent("PonyButt".into_string()));
        let mut response = request.start().unwrap().send().unwrap();
        let text = response.read_to_string().unwrap();
        let login = decode::<JsonLogin>(text[]).unwrap().login;
        let SetCookie(cookies) = response.headers.get::<SetCookie>().unwrap().clone();
        assert!(login.result[] == "NeedToken");
        (cookies, login.token)
    }
    fn login_final(&self, username: &str, password: &str, token: &str) {
        #[deriving(Decodable, Show)]
        struct JsonLoginFinal {
            login: JsonLoginFinalInner,
        }
        #[deriving(Decodable, Show)]
        struct JsonLoginFinalInner {
            result: String,
            lguserid: i32,
            lgusername: String,
            lgtoken: String,
            cookieprefix: String,
            sessionid: String,
        }
        let url = make_url(&[("format", "json"), ("action", "login"), ("lgname", username), ("lgpassword", password), ("lgtoken", token)]);
        let request = self.make_request(url[], Method::Post);
        let mut response = request.start().unwrap().send().unwrap();
        let text = response.read_to_string().unwrap();
        let login = decode::<JsonLoginFinal>(text[]).unwrap().login;
        assert!(login.result[] == "Success");
    }
    fn login() -> WikiApi {
        #[deriving(Decodable, Show)]
        struct LoginConfig {
            username: String,
            password: String,
        }
        let mut file = File::open(&Path::new("work/config.json")).unwrap();
        let data = file.read_to_string().unwrap();
        let config: LoginConfig = decode(data[]).unwrap();
        let (cookies, token) = WikiApi::login_first(config.username[], config.password[]);
        let api = WikiApi {
            cookies: cookies,
        };
        api.login_final(config.username[], config.password[], token[]);
        println!("Logged in: {}", token);
        api
    }
    #[allow(dead_code)]
    fn get_patrol_token(&self) -> String {
        #[deriving(Decodable, Show)]
        struct Token {
            tokens: TokenInner,
        }
        #[deriving(Decodable, Show)]
        struct TokenInner {
            patroltoken: String,
        }
        let url = make_url(&[("format", "json"), ("action", "tokens"), ("type", "patrol")]);
        let request = self.make_request(url[], Method::Get);
        let mut response = request.start().unwrap().send().unwrap();
        let text = response.read_to_string().unwrap();
        let token = decode::<Token>(text[]).unwrap().tokens.patroltoken;
        println!("Patrol token: {}", token);
        token
    }
}
