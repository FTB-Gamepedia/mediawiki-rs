use crate::{Csrf, Error, Json, Mediawiki, QueryBuilder, Token};
pub trait Oredict {
    fn query_ores(&self) -> QueryBuilder;
    fn delete_ores(&self, token: &Token<Csrf>, ids: &str) -> Result<Json, Error>;
}
impl Oredict for Mediawiki {
    fn query_ores(&self) -> QueryBuilder {
        let mut query = self.query("oredictentries");
        query.arg("odlimit", "5000");
        query.arg("list", "oredictsearch");
        query
    }
    fn delete_ores(&self, token: &Token<Csrf>, ids: &str) -> Result<Json, Error> {
        let mut request = self.request();
        request.arg("action", "deleteoredict");
        request.arg("odtoken", &*token.0);
        request.arg("odids", ids);
        request.post()
    }
}
