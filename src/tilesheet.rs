use crate::{Csrf, Error, Json, Mediawiki, QueryBuilder, Token};
pub trait Tilesheet {
    fn query_tiles(&self, tsmod: Option<&str>) -> QueryBuilder;
    fn add_tiles(&self, token: &Token<Csrf>, tsmod: &str, tsimport: &str) -> Result<Json, Error>;
    fn delete_tiles(&self, token: &Token<Csrf>, ids: &str) -> Result<Json, Error>;
    fn query_sheets(&self) -> QueryBuilder;
    fn create_sheet(&self, token: &Token<Csrf>, tsmod: &str, tssizes: &str) -> Result<Json, Error>;
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
    fn query_sheets(&self) -> QueryBuilder {
        let mut query = self.query("tilesheets");
        query.arg("list", "tilesheets");
        query.arg("tslimit", "5000");
        query
    }
    fn delete_tiles(&self, token: &Token<Csrf>, ids: &str) -> Result<Json, Error> {
        let mut request = self.request();
        request.arg("action", "deletetiles");
        request.arg("tstoken", &*token.0);
        request.arg("tsids", ids);
        request.post()
    }
    fn add_tiles(&self, token: &Token<Csrf>, tsmod: &str, tsimport: &str) -> Result<Json, Error> {
        let mut request = self.request();
        request.arg("action", "addtiles");
        request.arg("tstoken", &*token.0);
        request.arg("tsmod", tsmod);
        request.arg("tsimport", tsimport);
        request.post()
    }
    fn create_sheet(&self, token: &Token<Csrf>, tsmod: &str, tssizes: &str) -> Result<Json, Error> {
        let mut request = self.request();
        request.arg("action", "createsheet");
        request.arg("tstoken", &*token.0);
        request.arg("tsmod", tsmod);
        request.arg("tssizes", tssizes);
        request.post()
    }
}
