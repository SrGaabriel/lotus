use crate::core::{
    TermArena,
    TermId,
};

#[derive(Debug, Clone, PartialEq, Eq, salsa::Update)]
pub struct Body<'db> {
    pub arena: TermArena<'db>,
    pub value: Option<TermId>,
    pub ty: Option<TermId>,
}

impl Body<'_> {
    pub fn empty() -> Self {
        Self {
            arena: TermArena::new(),
            value: None,
            ty: None,
        }
    }
}
