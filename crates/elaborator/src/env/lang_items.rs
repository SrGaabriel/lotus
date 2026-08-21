use std::str::FromStr;

use ast::traits::AstNode;
use diagnostics::Diagnostic;
use rustc_hash::FxHashMap;
use salsa::Accumulator;
use strum::{
    Display,
    EnumString,
};
use text_size::TextRange;

use crate::{
    ElabDatabase,
    ElabDb,
    ItemId,
    env::Namespace,
    ids::ItemKind,
};

pub type LanguageItems<'db> = FxHashMap<LangItem, ItemId<'db>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumString, Display, salsa::Update)]
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
    #[strum(serialize = "type.uint8")]
    UInt8,
    #[strum(serialize = "type.uint16")]
    UInt16,
    #[strum(serialize = "type.uint32")]
    UInt32,
    #[strum(serialize = "type.uint64")]
    UInt64,
    #[strum(serialize = "type.float32")]
    Float32,
    #[strum(serialize = "type.float64")]
    Float64,
    #[strum(serialize = "type.str")]
    Str,
    #[strum(serialize = "type.unit")]
    Unit,
    #[strum(serialize = "constructor.unit")]
    UnitConstructor,
}

#[salsa::tracked(returns(ref))]
pub fn file_lang_items(db: &dyn ElabDatabase, file: db::SourceFile) -> LanguageItems<'_> {
    let tree = db.item_tree(file);
    let parse = ast::parse_file(db, file);
    let source = parse.tree();

    let mut lang_items = FxHashMap::default();

    for &id in tree.items(db) {
        let attrs: Vec<ast::Attribute> = match id.kind(db) {
            ItemKind::Def | ItemKind::Inductive => {
                let Some(decl) = source.decl().nth(id.ast_index(db) as usize) else {
                    continue;
                };
                match &decl {
                    ast::Decl::DefDecl(def) => def.attribute().collect(),
                    ast::Decl::InductiveDecl(ind) => ind.attribute().collect(),
                    ast::Decl::ImportDecl(_) => continue,
                }
            }
            ItemKind::Constructor => {
                let Some(parent) = id.parent(db) else {
                    continue;
                };
                let Some(ast::Decl::InductiveDecl(ind)) =
                    source.decl().nth(parent.ast_index(db) as usize)
                else {
                    continue;
                };
                let Some(ctors) = ind.inductive_constructors() else {
                    continue;
                };
                let Some(ctor) = ctors.constructor_decl().nth(id.ast_index(db) as usize) else {
                    continue;
                };
                ctor.attribute().collect()
            }
        };

        for attr in attrs {
            if let Some(lang_name) = parse_lang_attr(db, file, &attr) {
                lang_items.insert(lang_name, id);
            }
        }
    }

    lang_items
}

pub type LangItemCandidates<'db> = FxHashMap<LangItem, Vec<ItemId<'db>>>;

#[salsa::tracked(returns(ref))]
pub fn visible_lang_items(db: &dyn ElabDatabase, file: db::SourceFile) -> LangItemCandidates<'_> {
    let namespace = db.def_map(file);
    let mut visible: LangItemCandidates<'_> = FxHashMap::default();
    collect_namespace(db, namespace, &mut visible);
    for candidates in visible.values_mut() {
        candidates.sort_by(|a, b| {
            (a.file(db).path(db), a.ast_index(db)).cmp(&(b.file(db).path(db), b.ast_index(db)))
        });
    }
    visible
}

fn collect_namespace<'db>(
    db: &'db dyn ElabDatabase,
    namespace: Namespace<'db>,
    out: &mut LangItemCandidates<'db>,
) {
    for &item in namespace.decls(db).values() {
        let defining = db.lang_items(item.file(db));
        for (&lang_item, &id) in defining {
            if id == item {
                let candidates = out.entry(lang_item).or_default();
                if !candidates.contains(&item) {
                    candidates.push(item);
                }
            }
        }
    }
    for &child in namespace.children(db).values() {
        collect_namespace(db, child, out);
    }
}

pub fn item_range(db: &dyn ElabDatabase, item: ItemId<'_>) -> Option<TextRange> {
    let parse = ast::parse_file(db, item.file(db));
    let source = parse.tree();
    match item.kind(db) {
        ItemKind::Def | ItemKind::Inductive => Some(
            source
                .decl()
                .nth(item.ast_index(db) as usize)?
                .syntax()
                .text_range(),
        ),
        ItemKind::Constructor => {
            let parent = item.parent(db)?;
            let ast::Decl::InductiveDecl(ind) = source.decl().nth(parent.ast_index(db) as usize)?
            else {
                return None;
            };
            Some(
                ind.inductive_constructors()?
                    .constructor_decl()
                    .nth(item.ast_index(db) as usize)?
                    .syntax()
                    .text_range(),
            )
        }
    }
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
