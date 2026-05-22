use crate::core::Term;

#[derive(Debug, Clone, PartialEq, Eq, salsa::Update)]
pub struct DefBody<'db> {
    pub value: Term<'db>,
}
