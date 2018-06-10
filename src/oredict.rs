// Copyright © 2016-2018, Peter Atashian
use {
    Mediawiki, QueryBuilder,
};
pub trait Oredict {
    fn query_ores(&self) -> QueryBuilder;
}
impl Oredict for Mediawiki {
    fn query_ores(&self) -> QueryBuilder {
        let mut query = self.query("oredictentries");
        query.arg("odlimit", "5000");
        query.arg("list", "oredictsearch");
        query
    }
}
