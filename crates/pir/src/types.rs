use elaborator::{
    ItemId,
    core::Term,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, salsa::Update)]
pub enum TypeArg<'db> {
    Type(Type<'db>),
    Index(Term<'db>),
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
    Char,
    Param(usize),
    Array(Box<Type<'db>>, usize),
    Record(Vec<Type<'db>>),
    Adt {
        id: ItemId<'db>,
        args: Vec<TypeArg<'db>>,
    },
    Function {
        ret: Box<Type<'db>>,
        params: Vec<Type<'db>>,
    },
    Todo(String),
}
