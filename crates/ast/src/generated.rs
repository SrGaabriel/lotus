use crate::traits::{AstChildren, AstNode, child, children, token};
use syntax::{ResolvedNode, ResolvedToken, red::SyntaxKind};
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct SourceFile(ResolvedNode);
impl AstNode for SourceFile {
    fn can_cast(k: SyntaxKind) -> bool {
        k == SyntaxKind::SourceFile
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        Self::can_cast(node.kind()).then_some(Self(node))
    }
    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}
impl SourceFile {
    pub fn decl(&self) -> AstChildren<'_, Decl> {
        children(&self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct DefDecl(ResolvedNode);
impl AstNode for DefDecl {
    fn can_cast(k: SyntaxKind) -> bool {
        k == SyntaxKind::DefDecl
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        Self::can_cast(node.kind()).then_some(Self(node))
    }
    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}
impl DefDecl {
    pub fn def_kw(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::DefKw)
    }
    pub fn name(&self) -> Option<Name> {
        child(&self.0)
    }
    pub fn def_eq(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::DefEq)
    }
    pub fn expr(&self) -> Option<Expr> {
        child(&self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Name(ResolvedNode);
impl AstNode for Name {
    fn can_cast(k: SyntaxKind) -> bool {
        k == SyntaxKind::Name
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        Self::can_cast(node.kind()).then_some(Self(node))
    }
    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}
impl Name {
    pub fn ident(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::Identifier)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct NumberLit(ResolvedNode);
impl AstNode for NumberLit {
    fn can_cast(k: SyntaxKind) -> bool {
        k == SyntaxKind::NumberLit
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        Self::can_cast(node.kind()).then_some(Self(node))
    }
    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}
impl NumberLit {
    pub fn number_lit(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::NumberLit)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct StringLit(ResolvedNode);
impl AstNode for StringLit {
    fn can_cast(k: SyntaxKind) -> bool {
        k == SyntaxKind::StringLit
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        Self::can_cast(node.kind()).then_some(Self(node))
    }
    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}
impl StringLit {
    pub fn string_lit(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::StringLit)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Decl {
    DefDecl(DefDecl),
}
impl AstNode for Decl {
    fn can_cast(k: SyntaxKind) -> bool {
        DefDecl::can_cast(k)
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        if let Some(it) = DefDecl::cast(node.clone()) {
            return Some(Self::DefDecl(it));
        }
        None
    }
    fn syntax(&self) -> &ResolvedNode {
        match self {
            Self::DefDecl(it) => it.syntax(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    Literal(Literal),
    Name(Name),
}
impl AstNode for Expr {
    fn can_cast(k: SyntaxKind) -> bool {
        Literal::can_cast(k) || Name::can_cast(k)
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        if let Some(it) = Literal::cast(node.clone()) {
            return Some(Self::Literal(it));
        }
        if let Some(it) = Name::cast(node.clone()) {
            return Some(Self::Name(it));
        }
        None
    }
    fn syntax(&self) -> &ResolvedNode {
        match self {
            Self::Literal(it) => it.syntax(),
            Self::Name(it) => it.syntax(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Literal {
    NumberLit(NumberLit),
    StringLit(StringLit),
}
impl AstNode for Literal {
    fn can_cast(k: SyntaxKind) -> bool {
        NumberLit::can_cast(k) || StringLit::can_cast(k)
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        if let Some(it) = NumberLit::cast(node.clone()) {
            return Some(Self::NumberLit(it));
        }
        if let Some(it) = StringLit::cast(node.clone()) {
            return Some(Self::StringLit(it));
        }
        None
    }
    fn syntax(&self) -> &ResolvedNode {
        match self {
            Self::NumberLit(it) => it.syntax(),
            Self::StringLit(it) => it.syntax(),
        }
    }
}
