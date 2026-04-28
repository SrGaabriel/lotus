#[salsa::db]
#[derive(Clone, Default)]
pub struct Db {
    storage: salsa::Storage<Db>,
}

#[salsa::db]
impl salsa::Database for Db {}
