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
        LevelKind,
        Literal,
        Term,
        TermKind,
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
    write_term(out, db, item.signature.ty, 0)?;
    writeln!(out)?;

    if let Some(body) = item.def_body.as_ref() {
        write!(out, "  := ")?;
        write_term(out, db, body.value, 0)?;
        writeln!(out)?;
    }
    Ok(())
}

pub fn write_term<'db>(out: &mut dyn Write, db: Db<'db>, term: Term<'db>, prec: u8) -> fmt::Result {
    match term.kind(db) {
        TermKind::BVar(i) => write!(out, "#{i}"),
        TermKind::FVar(u) => write!(out, "?f{}", u.0),
        TermKind::MVar(u) => write!(out, "?m{}", u.0),
        TermKind::Const(d) => write!(out, "{}", d.name(db).text(db)),
        TermKind::Sort(l) => {
            write!(out, "Sort ")?;
            write_level(out, db, *l, 11)
        }
        TermKind::Lit(lit) => write_lit(out, lit),
        TermKind::App(f, x) => paren(out, prec, 10, |o| {
            write_term(o, db, *f, 10)?;
            o.write_char(' ')?;
            write_term(o, db, *x, 11)
        }),
        TermKind::Lam(info, ty, body) => paren(out, prec, 0, |o| {
            o.write_str("λ ")?;
            write_binder(o, db, *info, *ty)?;
            o.write_str(" => ")?;
            write_term(o, db, *body, 0)
        }),
        TermKind::Pi(info, ty, body) => paren(out, prec, 0, |o| {
            write_binder(o, db, *info, *ty)?;
            o.write_str(" -> ")?;
            write_term(o, db, *body, 0)
        }),
        TermKind::Sigma(info, ty, body) => paren(out, prec, 0, |o| {
            o.write_str("Σ ")?;
            write_binder(o, db, *info, *ty)?;
            o.write_str(", ")?;
            write_term(o, db, *body, 0)
        }),
        TermKind::Let(ty, val, body) => paren(out, prec, 0, |o| {
            o.write_str("let : ")?;
            write_term(o, db, *ty, 0)?;
            o.write_str(" := ")?;
            write_term(o, db, *val, 0)?;
            o.write_str(" in ")?;
            write_term(o, db, *body, 0)
        }),
    }
}

fn write_binder<'db>(
    out: &mut dyn Write,
    db: Db<'db>,
    info: BinderInfo,
    ty: Term<'db>,
) -> fmt::Result {
    let (open, close) = match info {
        BinderInfo::Explicit => ("(", ")"),
        BinderInfo::Implicit => ("{", "}"),
        BinderInfo::StrictImplicit => ("{{", "}}"),
        BinderInfo::InstanceImplicit => ("[", "]"),
    };
    out.write_str(open)?;
    out.write_str("_ : ")?;
    write_term(out, db, ty, 0)?;
    out.write_str(close)
}

fn write_level<'db>(out: &mut dyn Write, db: Db<'db>, level: Level<'db>, prec: u8) -> fmt::Result {
    let mut succs = 0u32;
    let mut cur = level;
    while let LevelKind::Succ(inner) = cur.kind(db) {
        succs += 1;
        cur = *inner;
    }
    match cur.kind(db) {
        LevelKind::Zero if succs > 0 => write!(out, "{succs}"),
        LevelKind::Zero => out.write_char('0'),
        LevelKind::Succ(_) => unreachable!(),
        LevelKind::Max(a, b) => paren(out, prec, 5, |o| {
            o.write_str("max ")?;
            write_level(o, db, *a, 6)?;
            o.write_char(' ')?;
            write_level(o, db, *b, 6)?;
            if succs > 0 {
                write!(o, " + {succs}")?;
            }
            Ok(())
        }),
        LevelKind::IMax(a, b) => paren(out, prec, 5, |o| {
            o.write_str("imax ")?;
            write_level(o, db, *a, 6)?;
            o.write_char(' ')?;
            write_level(o, db, *b, 6)?;
            if succs > 0 {
                write!(o, " + {succs}")?;
            }
            Ok(())
        }),
        LevelKind::MVar(u) => {
            write!(out, "?u{}", u.0)?;
            if succs > 0 {
                write!(out, "+{succs}")?;
            }
            Ok(())
        }
        LevelKind::Param(s) => {
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
