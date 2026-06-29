use elaborator::ItemId;
use literals::Literal;

pub type LocalId = usize;

pub enum Atom<'db> {
    Local(LocalId),
    Global(ItemId<'db>),
    Literal(Literal),
    Erased,
}

pub enum LetValue<'db> {
    Atom(Atom<'db>),

    Call {
        callee: Atom<'db>,
        args: Vec<Atom<'db>>,
    },

    Constructor {
        ctor: ItemId<'db>,
        args: Vec<Atom<'db>>,
    },

    Projection {
        value: Atom<'db>,
        field: usize,
    },

    Lambda {
        params: Vec<LocalId>,
        body: Box<Code<'db>>,
    },
}

pub enum Code<'db> {
    Let {
        binder: LocalId,
        ty: Type<'db>,
        value: LetValue<'db>,
        body: Box<Code<'db>>,
    },

    Return(Atom<'db>),
    Unreachable,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, salsa::Update)]
pub enum Type<'db> {
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,

    Bool,
    Param(usize),
    Adt {
        id: ItemId<'db>,
        args: Vec<Type<'db>>,
    },
    Function {
        ret: Box<Type<'db>>,
        params: Vec<Type<'db>>,
    },
    Todo(String),
}
