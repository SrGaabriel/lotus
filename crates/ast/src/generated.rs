// @generated
use crate::traits::{AstNode, AstChildren, child, children, token, token_text};
use syntax::{ResolvedNode, ResolvedToken, kind::SyntaxKind};
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
    pub fn attribute(&self) -> AstChildren<'_, Attribute> {
        children(&self.0)
    }
    pub fn def_kw(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::DefKw)
    }
    pub fn ident(&self) -> Option<Identifier> {
        child(&self.0)
    }
    pub fn binders(&self) -> AstChildren<'_, Binder> {
        children(&self.0)
    }
    pub fn return_type(&self) -> Option<ReturnType> {
        child(&self.0)
    }
    pub fn def_eq(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::DefEq)
    }
    pub fn body(&self) -> Option<Expr> {
        child(&self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct InductiveDecl(ResolvedNode);
impl AstNode for InductiveDecl {
    fn can_cast(k: SyntaxKind) -> bool {
        k == SyntaxKind::InductiveDecl
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        Self::can_cast(node.kind()).then_some(Self(node))
    }
    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}
impl InductiveDecl {
    pub fn attribute(&self) -> AstChildren<'_, Attribute> {
        children(&self.0)
    }
    pub fn inductive_kw(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::InductiveKw)
    }
    pub fn ident(&self) -> Option<Identifier> {
        child(&self.0)
    }
    pub fn binders(&self) -> AstChildren<'_, Binder> {
        children(&self.0)
    }
    pub fn return_type(&self) -> Option<ReturnType> {
        child(&self.0)
    }
    pub fn def_eq(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::DefEq)
    }
    pub fn inductive_constructors(&self) -> Option<InductiveConstructors> {
        child(&self.0)
    }
    pub fn semicolon(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::Semicolon)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Attribute(ResolvedNode);
impl AstNode for Attribute {
    fn can_cast(k: SyntaxKind) -> bool {
        k == SyntaxKind::Attribute
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        Self::can_cast(node.kind()).then_some(Self(node))
    }
    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}
impl Attribute {
    pub fn at(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::At)
    }
    pub fn l_bracket(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::LBracket)
    }
    pub fn identifier(&self) -> Option<Identifier> {
        child(&self.0)
    }
    pub fn attribute_value(&self) -> Option<AttributeValue> {
        child(&self.0)
    }
    pub fn r_bracket(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::RBracket)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Identifier(ResolvedNode);
impl AstNode for Identifier {
    fn can_cast(k: SyntaxKind) -> bool {
        k == SyntaxKind::Identifier
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        Self::can_cast(node.kind()).then_some(Self(node))
    }
    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}
impl Identifier {
    pub fn ident(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::Identifier)
    }
    pub fn text(&self) -> Option<&str> {
        token_text(&self.0, SyntaxKind::Identifier)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ReturnType(ResolvedNode);
impl AstNode for ReturnType {
    fn can_cast(k: SyntaxKind) -> bool {
        k == SyntaxKind::ReturnType
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        Self::can_cast(node.kind()).then_some(Self(node))
    }
    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}
impl ReturnType {
    pub fn colon(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::Colon)
    }
    pub fn r#type(&self) -> Option<Type> {
        child(&self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct InductiveConstructors(ResolvedNode);
impl AstNode for InductiveConstructors {
    fn can_cast(k: SyntaxKind) -> bool {
        k == SyntaxKind::InductiveConstructors
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        Self::can_cast(node.kind()).then_some(Self(node))
    }
    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}
impl InductiveConstructors {
    pub fn constructor_decl(&self) -> AstChildren<'_, ConstructorDecl> {
        children(&self.0)
    }
    pub fn pipe(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::Pipe)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ConstructorDecl(ResolvedNode);
impl AstNode for ConstructorDecl {
    fn can_cast(k: SyntaxKind) -> bool {
        k == SyntaxKind::ConstructorDecl
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        Self::can_cast(node.kind()).then_some(Self(node))
    }
    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}
impl ConstructorDecl {
    pub fn attribute(&self) -> AstChildren<'_, Attribute> {
        children(&self.0)
    }
    pub fn ident(&self) -> Option<Identifier> {
        child(&self.0)
    }
    pub fn binders(&self) -> AstChildren<'_, Binder> {
        children(&self.0)
    }
    pub fn return_type(&self) -> Option<ReturnType> {
        child(&self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ParenExpr(ResolvedNode);
impl AstNode for ParenExpr {
    fn can_cast(k: SyntaxKind) -> bool {
        k == SyntaxKind::ParenExpr
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        Self::can_cast(node.kind()).then_some(Self(node))
    }
    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}
impl ParenExpr {
    pub fn l_paren(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::LParen)
    }
    pub fn expr(&self) -> Option<Expr> {
        child(&self.0)
    }
    pub fn r_paren(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::RParen)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct BraceBlock(ResolvedNode);
impl AstNode for BraceBlock {
    fn can_cast(k: SyntaxKind) -> bool {
        k == SyntaxKind::BraceBlock
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        Self::can_cast(node.kind()).then_some(Self(node))
    }
    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}
impl BraceBlock {
    pub fn l_brace(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::LBrace)
    }
    pub fn stmt(&self) -> AstChildren<'_, Stmt> {
        children(&self.0)
    }
    pub fn r_brace(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::RBrace)
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
    pub fn path(&self) -> AstChildren<'_, PathSegment> {
        children(&self.0)
    }
    pub fn member(&self) -> Option<Identifier> {
        child(&self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct LetStmt(ResolvedNode);
impl AstNode for LetStmt {
    fn can_cast(k: SyntaxKind) -> bool {
        k == SyntaxKind::LetStmt
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        Self::can_cast(node.kind()).then_some(Self(node))
    }
    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}
impl LetStmt {
    pub fn let_kw(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::LetKw)
    }
    pub fn name(&self) -> Option<Identifier> {
        child(&self.0)
    }
    pub fn type_annotation(&self) -> Option<TypeAnnotation> {
        child(&self.0)
    }
    pub fn def_eq(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::DefEq)
    }
    pub fn expr(&self) -> Option<Expr> {
        child(&self.0)
    }
    pub fn semicolon(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::Semicolon)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct MutationStmt(ResolvedNode);
impl AstNode for MutationStmt {
    fn can_cast(k: SyntaxKind) -> bool {
        k == SyntaxKind::MutationStmt
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        Self::can_cast(node.kind()).then_some(Self(node))
    }
    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}
impl MutationStmt {
    pub fn name(&self) -> Option<Identifier> {
        child(&self.0)
    }
    pub fn def_eq(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::DefEq)
    }
    pub fn expr(&self) -> Option<Expr> {
        child(&self.0)
    }
    pub fn semicolon(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::Semicolon)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ReturnStmt(ResolvedNode);
impl AstNode for ReturnStmt {
    fn can_cast(k: SyntaxKind) -> bool {
        k == SyntaxKind::ReturnStmt
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        Self::can_cast(node.kind()).then_some(Self(node))
    }
    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}
impl ReturnStmt {
    pub fn return_kw(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::ReturnKw)
    }
    pub fn expr(&self) -> Option<Expr> {
        child(&self.0)
    }
    pub fn semicolon(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::Semicolon)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct TypeAnnotation(ResolvedNode);
impl AstNode for TypeAnnotation {
    fn can_cast(k: SyntaxKind) -> bool {
        k == SyntaxKind::TypeAnnotation
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        Self::can_cast(node.kind()).then_some(Self(node))
    }
    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}
impl TypeAnnotation {
    pub fn colon(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::Colon)
    }
    pub fn r#type(&self) -> Option<Type> {
        child(&self.0)
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
    pub fn text(&self) -> Option<&str> {
        token_text(&self.0, SyntaxKind::NumberLit)
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
    pub fn text(&self) -> Option<&str> {
        token_text(&self.0, SyntaxKind::StringLit)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct ParenBinder(ResolvedNode);
impl AstNode for ParenBinder {
    fn can_cast(k: SyntaxKind) -> bool {
        k == SyntaxKind::ParenBinder
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        Self::can_cast(node.kind()).then_some(Self(node))
    }
    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}
impl ParenBinder {
    pub fn l_paren(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::LParen)
    }
    pub fn name(&self) -> Option<Identifier> {
        child(&self.0)
    }
    pub fn colon(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::Colon)
    }
    pub fn r#type(&self) -> Option<Type> {
        child(&self.0)
    }
    pub fn r_paren(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::RParen)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct BraceBinder(ResolvedNode);
impl AstNode for BraceBinder {
    fn can_cast(k: SyntaxKind) -> bool {
        k == SyntaxKind::BraceBinder
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        Self::can_cast(node.kind()).then_some(Self(node))
    }
    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}
impl BraceBinder {
    pub fn l_brace(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::LBrace)
    }
    pub fn name(&self) -> Option<Identifier> {
        child(&self.0)
    }
    pub fn colon(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::Colon)
    }
    pub fn r#type(&self) -> Option<Type> {
        child(&self.0)
    }
    pub fn r_brace(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::RBrace)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct BracketBinder(ResolvedNode);
impl AstNode for BracketBinder {
    fn can_cast(k: SyntaxKind) -> bool {
        k == SyntaxKind::BracketBinder
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        Self::can_cast(node.kind()).then_some(Self(node))
    }
    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}
impl BracketBinder {
    pub fn l_bracket(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::LBracket)
    }
    pub fn name(&self) -> Option<Identifier> {
        child(&self.0)
    }
    pub fn colon(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::Colon)
    }
    pub fn r#type(&self) -> Option<Type> {
        child(&self.0)
    }
    pub fn r_bracket(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::RBracket)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct PiType(ResolvedNode);
impl AstNode for PiType {
    fn can_cast(k: SyntaxKind) -> bool {
        k == SyntaxKind::PiType
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        Self::can_cast(node.kind()).then_some(Self(node))
    }
    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}
impl PiType {
    pub fn params(&self) -> AstChildren<'_, Binder> {
        children(&self.0)
    }
    pub fn arrow(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::RArrow)
    }
    pub fn return_type(&self) -> Option<Type> {
        child(&self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct PathSegment(ResolvedNode);
impl AstNode for PathSegment {
    fn can_cast(k: SyntaxKind) -> bool {
        k == SyntaxKind::PathSegment
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        Self::can_cast(node.kind()).then_some(Self(node))
    }
    fn syntax(&self) -> &ResolvedNode {
        &self.0
    }
}
impl PathSegment {
    pub fn colon_colon(&self) -> Option<ResolvedToken> {
        token(&self.0, SyntaxKind::ColonColon)
    }
    pub fn identifier(&self) -> Option<Identifier> {
        child(&self.0)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Decl {
    DefDecl(DefDecl),
    InductiveDecl(InductiveDecl),
}
impl AstNode for Decl {
    fn can_cast(k: SyntaxKind) -> bool {
        DefDecl::can_cast(k) || InductiveDecl::can_cast(k)
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        if let Some(it) = DefDecl::cast(node.clone()) {
            return Some(Self::DefDecl(it));
        }
        if let Some(it) = InductiveDecl::cast(node.clone()) {
            return Some(Self::InductiveDecl(it));
        }
        None
    }
    fn syntax(&self) -> &ResolvedNode {
        match self {
            Self::DefDecl(it) => it.syntax(),
            Self::InductiveDecl(it) => it.syntax(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Binder {
    ParenBinder(ParenBinder),
    BraceBinder(BraceBinder),
    BracketBinder(BracketBinder),
}
impl AstNode for Binder {
    fn can_cast(k: SyntaxKind) -> bool {
        ParenBinder::can_cast(k) || BraceBinder::can_cast(k)
            || BracketBinder::can_cast(k)
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        if let Some(it) = ParenBinder::cast(node.clone()) {
            return Some(Self::ParenBinder(it));
        }
        if let Some(it) = BraceBinder::cast(node.clone()) {
            return Some(Self::BraceBinder(it));
        }
        if let Some(it) = BracketBinder::cast(node.clone()) {
            return Some(Self::BracketBinder(it));
        }
        None
    }
    fn syntax(&self) -> &ResolvedNode {
        match self {
            Self::ParenBinder(it) => it.syntax(),
            Self::BraceBinder(it) => it.syntax(),
            Self::BracketBinder(it) => it.syntax(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    ParenExpr(ParenExpr),
    BraceBlock(BraceBlock),
    Literal(Literal),
    Name(Name),
}
impl AstNode for Expr {
    fn can_cast(k: SyntaxKind) -> bool {
        ParenExpr::can_cast(k) || BraceBlock::can_cast(k) || Literal::can_cast(k)
            || Name::can_cast(k)
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        if let Some(it) = ParenExpr::cast(node.clone()) {
            return Some(Self::ParenExpr(it));
        }
        if let Some(it) = BraceBlock::cast(node.clone()) {
            return Some(Self::BraceBlock(it));
        }
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
            Self::ParenExpr(it) => it.syntax(),
            Self::BraceBlock(it) => it.syntax(),
            Self::Literal(it) => it.syntax(),
            Self::Name(it) => it.syntax(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Name(Name),
    PiType(PiType),
}
impl AstNode for Type {
    fn can_cast(k: SyntaxKind) -> bool {
        Name::can_cast(k) || PiType::can_cast(k)
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        if let Some(it) = Name::cast(node.clone()) {
            return Some(Self::Name(it));
        }
        if let Some(it) = PiType::cast(node.clone()) {
            return Some(Self::PiType(it));
        }
        None
    }
    fn syntax(&self) -> &ResolvedNode {
        match self {
            Self::Name(it) => it.syntax(),
            Self::PiType(it) => it.syntax(),
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Stmt {
    LetStmt(LetStmt),
    MutationStmt(MutationStmt),
    ReturnStmt(ReturnStmt),
}
impl AstNode for Stmt {
    fn can_cast(k: SyntaxKind) -> bool {
        LetStmt::can_cast(k) || MutationStmt::can_cast(k) || ReturnStmt::can_cast(k)
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        if let Some(it) = LetStmt::cast(node.clone()) {
            return Some(Self::LetStmt(it));
        }
        if let Some(it) = MutationStmt::cast(node.clone()) {
            return Some(Self::MutationStmt(it));
        }
        if let Some(it) = ReturnStmt::cast(node.clone()) {
            return Some(Self::ReturnStmt(it));
        }
        None
    }
    fn syntax(&self) -> &ResolvedNode {
        match self {
            Self::LetStmt(it) => it.syntax(),
            Self::MutationStmt(it) => it.syntax(),
            Self::ReturnStmt(it) => it.syntax(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AttributeValue {
    StringLit(StringLit),
    NumberLit(NumberLit),
}
impl AstNode for AttributeValue {
    fn can_cast(k: SyntaxKind) -> bool {
        StringLit::can_cast(k) || NumberLit::can_cast(k)
    }
    fn cast(node: ResolvedNode) -> Option<Self> {
        if let Some(it) = StringLit::cast(node.clone()) {
            return Some(Self::StringLit(it));
        }
        if let Some(it) = NumberLit::cast(node.clone()) {
            return Some(Self::NumberLit(it));
        }
        None
    }
    fn syntax(&self) -> &ResolvedNode {
        match self {
            Self::StringLit(it) => it.syntax(),
            Self::NumberLit(it) => it.syntax(),
        }
    }
}
