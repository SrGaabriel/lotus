use std::marker::PhantomData;

use syntax::{ResolvedNode, ResolvedToken, kind::SyntaxKind};

pub trait AstNode: Clone {
    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized;
    fn cast(node: ResolvedNode) -> Option<Self>
    where
        Self: Sized;
    fn syntax(&self) -> &ResolvedNode;
}

pub trait AstToken: Clone {
    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized;
    fn cast(token: ResolvedToken) -> Option<Self>
    where
        Self: Sized;
    fn syntax(&self) -> &ResolvedToken;
}

pub struct AstChildren<'a, N> {
    inner: Box<dyn Iterator<Item = &'a ResolvedNode> + 'a>,
    _ph: PhantomData<N>,
}

impl<'a, N: AstNode> AstChildren<'a, N> {
    pub fn new(parent: &'a ResolvedNode) -> Self {
        Self {
            inner: Box::new(parent.children()),
            _ph: PhantomData,
        }
    }
}

impl<N: AstNode> Iterator for AstChildren<'_, N> {
    type Item = N;
    fn next(&mut self) -> Option<N> {
        self.inner.by_ref().find_map(|n| N::cast(n.clone()))
    }
}

pub fn child<N: AstNode>(parent: &ResolvedNode) -> Option<N> {
    AstChildren::<N>::new(parent).next()
}

pub fn children<N: AstNode>(parent: &ResolvedNode) -> AstChildren<'_, N> {
    AstChildren::new(parent)
}

pub fn token(parent: &ResolvedNode, kind: SyntaxKind) -> Option<ResolvedToken> {
    parent
        .children_with_tokens()
        .filter_map(|el| el.into_token().cloned())
        .find(|t| t.kind() == kind)
}
