use std::fmt::{
    self,
    Write,
};

use crate::{
    Db,
    ElaboratedFile,
    ElaboratedItem,
    core::{
        BinderInfo,
        Level,
        LevelId,
        Literal,
        Term,
        TermArena,
        TermId,
    },
};

pub fn debug_file(db: Db<'_>, file: &ElaboratedFile<'_>) -> String {
    let mut out = String::new();
    write_file(&mut out, db, file).expect("formatting to String is infallible");
    out
}

pub fn write_file(out: &mut dyn Write, db: Db<'_>, file: &ElaboratedFile<'_>) -> fmt::Result {
    writeln!(out, "elaborated module ({} items)", file.items.len())?;
    for item in &file.items {
        writeln!(out)?;
        write_item(out, db, item)?;
    }
    Ok(())
}

fn write_item(out: &mut dyn Write, db: Db<'_>, item: &ElaboratedItem<'_>) -> fmt::Result {
    let name = item.id.name(db).text(db);
    let kind = item.id.kind(db);
    write!(out, "{kind} {name} [#{}]", item.id.ast_index(db))?;

    write!(out, " : ")?;
    write_term(out, db, &item.signature.arena, item.signature.ty, 0)?;
    writeln!(out)?;

    if let Some(body) = item.def_body {
        write!(out, "  := ")?;
        write_term(out, db, &body.arena, body.value, 0)?;
        writeln!(out)?;
    }
    Ok(())
}

pub fn write_term(
    out: &mut dyn Write,
    db: Db<'_>,
    arena: &TermArena<'_>,
    id: TermId,
    prec: u8,
) -> fmt::Result {
    match arena.get_term(id) {
        Term::BVar(i) => write!(out, "#{i}"),
        Term::FVar(u) => write!(out, "?f{}", u.0),
        Term::MVar(u) => write!(out, "?m{}", u.0),
        Term::Const(d) => write!(out, "{}", d.name(db).text(db)),
        Term::Sort(l) => {
            write!(out, "Sort ")?;
            write_level(out, db, arena, *l, 11)
        }
        Term::Unit => write!(out, "()"),
        Term::Lit(lit) => write_lit(out, lit),
        Term::App(f, x) => paren(out, prec, 10, |o| {
            write_term(o, db, arena, *f, 10)?;
            o.write_char(' ')?;
            write_term(o, db, arena, *x, 11)
        }),
        Term::Lam(info, ty, body) => paren(out, prec, 0, |o| {
            o.write_str("λ ")?;
            write_binder(o, db, arena, *info, *ty)?;
            o.write_str(" => ")?;
            write_term(o, db, arena, *body, 0)
        }),
        Term::Pi(info, ty, body) => paren(out, prec, 0, |o| {
            write_binder(o, db, arena, *info, *ty)?;
            o.write_str(" -> ")?;
            write_term(o, db, arena, *body, 0)
        }),
        Term::Sigma(info, ty, body) => paren(out, prec, 0, |o| {
            o.write_str("Σ ")?;
            write_binder(o, db, arena, *info, *ty)?;
            o.write_str(", ")?;
            write_term(o, db, arena, *body, 0)
        }),
        Term::Let(ty, val, body) => paren(out, prec, 0, |o| {
            o.write_str("let : ")?;
            write_term(o, db, arena, *ty, 0)?;
            o.write_str(" := ")?;
            write_term(o, db, arena, *val, 0)?;
            o.write_str("; ")?;
            write_term(o, db, arena, *body, 0)
        }),
    }
}

fn write_binder(
    out: &mut dyn Write,
    db: Db<'_>,
    arena: &TermArena<'_>,
    info: BinderInfo,
    ty: TermId,
) -> fmt::Result {
    let (open, close) = match info {
        BinderInfo::Explicit => ("(", ")"),
        BinderInfo::Implicit => ("{", "}"),
        BinderInfo::StrictImplicit => ("{{", "}}"),
        BinderInfo::InstanceImplicit => ("[", "]"),
    };
    out.write_str(open)?;
    out.write_str("_ : ")?;
    write_term(out, db, arena, ty, 0)?;
    out.write_str(close)
}

fn write_level(
    out: &mut dyn Write,
    db: Db<'_>,
    arena: &TermArena<'_>,
    id: LevelId,
    prec: u8,
) -> fmt::Result {
    let mut succs = 0u32;
    let mut cur = id;
    while let Level::Succ(inner) = arena.get_level(cur) {
        succs += 1;
        cur = *inner;
    }
    match arena.get_level(cur) {
        Level::Zero if succs > 0 => write!(out, "{succs}"),
        Level::Zero => out.write_char('0'),
        Level::Succ(_) => unreachable!(),
        Level::Max(a, b) => paren(out, prec, 5, |o| {
            o.write_str("max ")?;
            write_level(o, db, arena, *a, 6)?;
            o.write_char(' ')?;
            write_level(o, db, arena, *b, 6)?;
            if succs > 0 {
                write!(o, " + {succs}")?;
            }
            Ok(())
        }),
        Level::IMax(a, b) => paren(out, prec, 5, |o| {
            o.write_str("imax ")?;
            write_level(o, db, arena, *a, 6)?;
            o.write_char(' ')?;
            write_level(o, db, arena, *b, 6)?;
            if succs > 0 {
                write!(o, " + {succs}")?;
            }
            Ok(())
        }),
        Level::MVar(u) => {
            write!(out, "?u{}", u.0)?;
            if succs > 0 {
                write!(out, "+{succs}")?;
            }
            Ok(())
        }
        Level::Param(s) => {
            write!(out, "{}", s.text(db))?;
            if succs > 0 {
                write!(out, "+{succs}")?;
            }
            Ok(())
        }
    }
}

fn write_lit(out: &mut dyn Write, lit: &Literal) -> fmt::Result {
    match lit {
        Literal::Number(n) => write!(out, "{n}"),
        Literal::Str(s) => write!(out, "{s:?}"),
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
