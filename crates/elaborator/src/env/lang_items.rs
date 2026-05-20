use std::str::FromStr;

use ast::traits::AstNode;
use diagnostics::Diagnostic;
use rustc_hash::FxHashMap;
use salsa::Accumulator;
use strum::{
    Display,
    EnumString,
};

use crate::{
    ElabDatabase,
    ElabDb,
    ItemId,
};

pub type LanguageItems<'db> = FxHashMap<LangItem, ItemId<'db>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display)]
pub enum LangItem {
    #[strum(serialize = "type.boolean")]
    Boolean,
    #[strum(serialize = "type.int8")]
    Int8,
    #[strum(serialize = "type.int16")]
    Int16,
    #[strum(serialize = "type.int32")]
    Int32,
    #[strum(serialize = "type.int64")]
    Int64,
}

unsafe impl salsa::Update for LangItem {
    unsafe fn maybe_update(old_pointer: *mut Self, new_value: Self) -> bool {
        unsafe {
            if *old_pointer == new_value {
                false
            } else {
                *old_pointer = new_value;
                true
            }
        }
    }
}

#[salsa::tracked(returns(ref))]
pub fn file_lang_items(db: &dyn ElabDatabase, file: db::SourceFile) -> LanguageItems<'_> {
    let tree = db.item_tree(file);
    let parse = ast::parse_file(db, file);
    let source = parse.tree();

    let mut lang_items = FxHashMap::default();

    for &id in tree.items(db) {
        let Some(decl) = source.decl().nth(id.ast_index(db) as usize) else {
            continue;
        };

        let attrs = match &decl {
            ast::Decl::DefDecl(def) => def.attribute(),
            ast::Decl::InductiveDecl(ind) => ind.attribute(),
        };

        for attr in attrs {
            if let Some(lang_name) = parse_lang_attr(db, file, &attr) {
                lang_items.insert(lang_name, id);
            }
        }
    }

    lang_items
}

fn parse_lang_attr(
    db: &dyn ElabDatabase,
    file: db::SourceFile,
    attr: &ast::Attribute,
) -> Option<LangItem> {
    let ident = attr.identifier()?;
    if ident.text()? != "lang" {
        return None;
    }

    let lit = attr.attribute_value()?;
    let ast::AttributeValue::StringLit(s) = lit else {
        let diagnostic = Diagnostic::builder(
            diagnostics::Severity::Error,
            "invalid lang item name",
            file,
            attr.syntax().text_range(),
        )
        .build();
        diagnostic.accumulate(db);
        return None;
    };
    let text = s.unquoted()?;
    LangItem::from_str(text).ok()
}
