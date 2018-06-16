// Copyright © 2016-2018, Peter Atashian
use {
    Csrf, Error, Mediawiki, QueryBuilder, Token, Json,
};
pub trait Tilesheet {
    fn query_tiles(&self, tsmod: Option<&str>) -> QueryBuilder;
    fn delete_tiles(&self, token: &Token<Csrf>, ids: &str) -> Result<Json, Error>;
}
impl Tilesheet for Mediawiki {
    fn query_tiles(&self, tsmod: Option<&str>) -> QueryBuilder {
        let mut query = self.query("tiles");
        query.arg("list", "tiles");
        query.arg("tslimit", "5000");
        if let Some(tsmod) = tsmod {
            query.arg("tsmod", tsmod);
        }
        query
    }
    fn delete_tiles(&self, token: &Token<Csrf>, ids: &str) -> Result<Json, Error> {
        let mut request = self.request();
        request.arg("action", "deletetiles");
        request.arg("tstoken", &*token.0);
        request.arg("tsids", ids);
        request.post()
    }
    /*
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
    */
}
