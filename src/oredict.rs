use crate::{Csrf, Error, Json, Mediawiki, QueryBuilder, Token};
pub trait Oredict {
    fn query_ores(&self, odmod: Option<&str>) -> QueryBuilder;
    fn delete_ores(&self, token: &Token<Csrf>, ids: &str) -> Result<Json, Error>;
    fn edit_ore(
        &self,
        token: &Token<Csrf>,
        odid: i64,
        odmod: Option<&str>,
        odtag: Option<&str>,
        oditem: Option<&str>,
        odparams: Option<&str>,
    ) -> Result<Json, Error>;
}
impl Oredict for Mediawiki {
    fn query_ores(&self, odmod: Option<&str>) -> QueryBuilder {
        let mut query = self.query("oredictentries");
        query.arg("list", "oredictsearch");
        query.arg("odlimit", "5000");
        if let Some(odmod) = odmod {
            query.arg("odmod", odmod);
        }
        query
    }
    fn delete_ores(&self, token: &Token<Csrf>, ids: &str) -> Result<Json, Error> {
        let mut request = self.request();
        request.arg("action", "deleteoredict");
        request.arg("odtoken", &*token.0);
        request.arg("odids", ids);
        request.post()
    }
    fn edit_ore(
        &self,
        token: &Token<Csrf>,
        odid: i64,
        odmod: Option<&str>,
        odtag: Option<&str>,
        oditem: Option<&str>,
        odparams: Option<&str>,
    ) -> Result<Json, Error> {
        let mut request = self.request();
        request.arg("action", "editoredict");
        request.arg("odtoken", &*token.0);
        request.arg("odid", odid.to_string());
        if let Some(odmod) = odmod {
            request.arg("odmod", odmod);
        }
        if let Some(odtag) = odtag {
            request.arg("odtag", odtag);
        }
        if let Some(oditem) = oditem {
            request.arg("oditem", oditem);
        }
        if let Some(odparams) = odparams {
            request.arg("odparams", odparams);
        }
        request.post()
    }
}
