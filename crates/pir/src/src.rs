use crate::types::Type;

#[salsa::tracked(debug)]
pub struct PirFile<'db> {
    #[tracked]
    #[returns(ref)]
    pub items: Vec<PirItem<'db>>,
}

#[salsa::tracked(debug)]
pub struct PirItem<'db> {
    #[tracked]
    pub item: elaborator::ItemId<'db>,

    #[tracked]
    #[returns(ref)]
    pub ty: Type<'db>,
}
