use crate::{
    Identifier,
    StringLit,
    generated::{
        Binder,
        Type,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinderInfo {
    Explicit,
    Implicit,
    InstanceImplicit,
}

impl Binder {
    pub fn name(&self) -> Option<Identifier> {
        match self {
            Self::ParenBinder(b) => b.name(),
            Self::BraceBinder(b) => b.name(),
            Self::BracketBinder(b) => b.name(),
        }
    }

    pub fn ty(&self) -> Option<Type> {
        match self {
            Self::ParenBinder(b) => b.r#type(),
            Self::BraceBinder(b) => b.r#type(),
            Self::BracketBinder(b) => b.r#type(),
        }
    }

    pub fn info(&self) -> BinderInfo {
        match self {
            Self::ParenBinder(_) => BinderInfo::Explicit,
            Self::BraceBinder(_) => BinderInfo::Implicit,
            Self::BracketBinder(_) => BinderInfo::InstanceImplicit,
        }
    }
}

impl StringLit {
    pub fn unquoted(&self) -> Option<&str> {
        let text = self.text()?;
        text.strip_prefix('"')?.strip_suffix('"')
    }
}
