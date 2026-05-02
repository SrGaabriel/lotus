use db::SourceFile;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Unique {
    pub id: usize,
    pub source: SourceFile,
}

impl Unique {
    pub const fn new(id: usize, source: SourceFile) -> Self {
        Self { id, source }
    }
}

#[derive(Debug)]
pub struct UniqueGen {
    source: SourceFile,
    next: usize,
}

impl UniqueGen {
    pub fn new(source: SourceFile) -> Self {
        Self { source, next: 0 }
    }

    pub fn fresh(&mut self) -> Unique {
        let id = self.next;
        self.next += 1;
        Unique::new(id, self.source)
    }

    pub fn fresh_name(&mut self, name: String) -> Name {
        let unique = self.fresh();
        Name::Explicit(unique, name)
    }

    pub fn fresh_anonymous(&mut self) -> Name {
        let unique = self.fresh();
        Name::Anonymous(unique)
    }

    pub fn fresh_unnamed(&mut self) -> Unique {
        let id = self.next;
        self.next += 1;
        Unique::new(id, self.source)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Name {
    Explicit(Unique, String),
    Anonymous(Unique),
}

impl core::fmt::Display for Name {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Name::Explicit(_, name) => write!(f, "{name}"),
            Name::Anonymous(_) => write!(f, "_"),
        }
    }
}

impl Name {
    pub fn id(&self) -> &Unique {
        match self {
            Name::Explicit(id, _) | Name::Anonymous(id) => id,
        }
    }
}
