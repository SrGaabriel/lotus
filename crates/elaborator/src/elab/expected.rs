use text_size::TextRange;

use crate::core::Term;

#[derive(Debug, Clone, Copy)]
pub struct Expected<'db> {
    pub ty: Term<'db>,
    pub reason: ExpectedReason,
}

#[derive(Debug, Clone, Copy)]
pub enum ExpectedReason {
    ReturnType { annotation: TextRange },
    Annotation { range: TextRange },
    None,
}

impl<'db> Expected<'db> {
    pub fn new(ty: Term<'db>, reason: ExpectedReason) -> Self {
        Self { ty, reason }
    }

    pub fn any(ty: Term<'db>) -> Self {
        Self {
            ty,
            reason: ExpectedReason::None,
        }
    }
}
