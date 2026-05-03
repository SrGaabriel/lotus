use crate::core::{
    TermArena,
    TermId,
};

#[derive(Debug, Clone, PartialEq, Eq, salsa::Update)]
pub struct DefBody<'db> {
    pub arena: TermArena<'db>,
    pub value: TermId,
}
