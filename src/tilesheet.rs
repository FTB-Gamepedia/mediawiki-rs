use crate::{Csrf, Error, Json, Mediawiki, QueryBuilder, Token};
pub trait Tilesheet {
    fn query_tiles(&self, tsmod: Option<&str>) -> QueryBuilder;
    fn query_tile_translations(&self, tsid: i64) -> QueryBuilder;
    fn add_tiles(
        &self,
        token: &Token<Csrf>,
        tsmod: &str,
        tsimport: &str,
        summary: Option<&str>,
    ) -> Result<Json, Error>;
    fn delete_sheet(
        &self,
        token: &Token<Csrf>,
        tsmods: &str,
        tssummary: Option<&str>,
    ) -> Result<Json, Error>;
    fn delete_tiles(
        &self,
        token: &Token<Csrf>,
        tsids: &str,
        summary: Option<&str>,
    ) -> Result<Json, Error>;
    fn query_sheets(&self) -> QueryBuilder;
    fn create_sheet(&self, token: &Token<Csrf>, tsmod: &str, tssizes: &str) -> Result<Json, Error>;
    #[allow(clippy::too_many_arguments)]
    fn edit_tile(
        &self,
        token: &Token<Csrf>,
        id: &str,
        summary: Option<&str>,
        toname: Option<&str>,
        tomod: Option<&str>,
        tox: Option<&str>,
        toy: Option<&str>,
        toz: Option<&str>,
    ) -> Result<Json, Error>;
}
impl Tilesheet for Mediawiki {
    fn query_tiles(&self, tsmod: Option<&str>) -> QueryBuilder {
        let mut query = self.query("tiles");
        query.arg("list", "tiles");
        query.arg("tslimit", "5000");
        query.argo("tsmod", tsmod);
        query
    }
    fn query_sheets(&self) -> QueryBuilder {
        let mut query = self.query("tilesheets");
        query.arg("list", "tilesheets");
        query.arg("tslimit", "5000");
        query
    }
    fn query_tile_translations(&self, tsid: i64) -> QueryBuilder {
        let mut query = self.query("tiles");
        query.arg("list", "tiletranslations");
        query.arg("tsid", tsid.to_string());
        query
    }
    fn delete_sheet(
        &self,
        token: &Token<Csrf>,
        tsmods: &str,
        tssummary: Option<&str>,
    ) -> Result<Json, Error> {
        let mut request = self.request();
        request.arg("action", "deletesheet");
        request.arg("tstoken", &*token.0);
        request.arg("tsmods", tsmods);
        request.argo("tssummary", tssummary);
        request.post()
    }
    fn delete_tiles(
        &self,
        token: &Token<Csrf>,
        tsids: &str,
        summary: Option<&str>,
    ) -> Result<Json, Error> {
        let mut request = self.request();
        request.arg("action", "deletetiles");
        request.arg("tstoken", &*token.0);
        request.arg("tsids", tsids);
        request.argo("tssummary", summary);
        request.post()
    }
    fn add_tiles(
        &self,
        token: &Token<Csrf>,
        tsmod: &str,
        tsimport: &str,
        summary: Option<&str>,
    ) -> Result<Json, Error> {
        let mut request = self.request();
        request.arg("action", "addtiles");
        request.arg("tstoken", &*token.0);
        request.arg("tsmod", tsmod);
        request.arg("tsimport", tsimport);
        request.argo("tssummary", summary);
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
    fn edit_tile(
        &self,
        token: &Token<Csrf>,
        id: &str,
        summary: Option<&str>,
        toname: Option<&str>,
        tomod: Option<&str>,
        tox: Option<&str>,
        toy: Option<&str>,
        toz: Option<&str>,
    ) -> Result<Json, Error> {
        let mut request = self.request();
        request.arg("action", "edittile");
        request.arg("tstoken", &*token.0);
        request.arg("tsid", id);
        request.argo("tssummary", summary);
        request.argo("tstoname", toname);
        request.argo("tstomod", tomod);
        request.argo("tstox", tox);
        request.argo("tstoy", toy);
        request.argo("tstoz", toz);
        request.post()
    }
}
