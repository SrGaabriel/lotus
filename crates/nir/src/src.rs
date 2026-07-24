use crate::types::Type;

#[salsa::tracked(debug)]
pub struct NirFile<'db> {
    #[tracked]
    #[returns(ref)]
    pub items: Vec<NirItem<'db>>,
}

#[salsa::tracked(debug)]
pub struct NirItem<'db> {
    #[tracked]
    pub item: elaborator::ItemId<'db>,

    #[tracked]
    #[returns(ref)]
    pub ty: Type<'db>,
}
