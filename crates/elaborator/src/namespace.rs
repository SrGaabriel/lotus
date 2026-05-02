use std::collections::HashMap;

use crate::unique::Name;

#[derive(Debug, Clone, Default)]
pub struct Namespace {
    pub decls: HashMap<String, Name>,
    pub children: HashMap<String, Namespace>,
}

impl Namespace {
    pub fn lookup_decl(&self, name: &str) -> Option<&Name> {
        self.decls.get(name)
    }

    pub fn child(&self, name: &str) -> Option<&Namespace> {
        self.children.get(name)
    }

    pub fn walk(&self, path: &[String]) -> Option<&Namespace> {
        let mut current = self;
        for segment in path {
            current = current.children.get(segment)?;
        }
        Some(current)
    }

    pub fn resolve(&self, path: &[String], member: &str) -> Option<&Name> {
        self.walk(path)?.lookup_decl(member)
    }
}
