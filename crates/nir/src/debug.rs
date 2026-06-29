use std::fmt::{
    self,
    Write,
};

use crate::{
    Db,
    src::{
        NirFile,
        NirItem,
    },
    types::Type,
};

pub fn debug_file(db: Db<'_>, file: &NirFile<'_>) -> String {
    let mut out = String::new();
    write_file(&mut out, db, file).expect("formatting to String is infallible");
    out
}

pub fn write_file(out: &mut dyn Write, db: Db<'_>, file: &NirFile<'_>) -> fmt::Result {
    let items = file.items(db);
    writeln!(out, "nir module ({} items)", items.len())?;
    for item in items {
        writeln!(out)?;
        write_item(out, db, *item)?;
    }
    Ok(())
}

fn write_item<'db>(out: &mut dyn Write, db: Db<'db>, item: NirItem<'db>) -> fmt::Result {
    let source = item.item(db);
    let name = source.name(db).text(db);
    let kind = source.kind(db);
    write!(out, "{kind} {name} [#{}]", source.ast_index(db))?;

    let ty = item.ty(db);
    write!(out, " : ")?;
    write_type(out, db, ty, 0)?;

    writeln!(out)
}

pub fn write_type(out: &mut dyn Write, db: Db<'_>, ty: &Type, prec: u8) -> fmt::Result {
    match ty {
        Type::Param(index) => write!(out, "T{index}"),
        Type::Int8 => out.write_str("i8"),
        Type::Int16 => out.write_str("i16"),
        Type::Int32 => out.write_str("i32"),
        Type::Int64 => out.write_str("i64"),
        Type::UInt8 => out.write_str("u8"),
        Type::UInt16 => out.write_str("u16"),
        Type::UInt32 => out.write_str("u32"),
        Type::UInt64 => out.write_str("u64"),
        Type::Float32 => out.write_str("f32"),
        Type::Float64 => out.write_str("f64"),
        Type::Bool => out.write_str("bool"),
        Type::Adt { id, args } => paren(out, prec, 0, |o| {
            let name = id.name(db).text(db);
            o.write_str(name)?;
            if !args.is_empty() {
                o.write_char('<')?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        o.write_str(", ")?;
                    }
                    write_type(o, db, arg, 0)?;
                }
                o.write_char('>')?;
            }
            Ok(())
        }),
        Type::Function { params, ret } => paren(out, prec, 0, |o| {
            o.write_str("fn(")?;
            for (i, param) in params.iter().enumerate() {
                if i > 0 {
                    o.write_str(", ")?;
                }
                write_type(o, db, param, 0)?;
            }
            o.write_str(") -> ")?;
            write_type(o, db, ret, 1)
        }),
        Type::Todo(text) => write!(out, "|todo {text}|"),
    }
}

fn paren(
    out: &mut dyn Write,
    outer: u8,
    inner: u8,
    f: impl FnOnce(&mut dyn Write) -> fmt::Result,
) -> fmt::Result {
    if outer > inner {
        out.write_char('(')?;
        f(out)?;
        out.write_char(')')
    } else {
        f(out)
    }
}
