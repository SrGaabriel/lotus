use ast::parse_file;
use db::SourceFile;

use crate::{
    ElabDatabase,
    ids::{
        ItemId,
        ItemKind,
        Symbol,
    },
};

#[salsa::tracked(debug)]
pub struct ItemTree<'db> {
    #[tracked]
    #[returns(ref)]
    pub items: Vec<ItemId<'db>>,
}

#[salsa::tracked]
pub fn item_tree(db: &dyn ElabDatabase, file: SourceFile) -> ItemTree<'_> {
    tracing::info!("Computing item tree for file {:?}", file);
    let parse = parse_file(db, file);
    let source = parse.tree();

    let mut items = Vec::new();
    for (i, decl) in source.decl().enumerate() {
        match decl {
            ast::Decl::DefDecl(def) => {
                let Some(symbol) = decl_symbol(db, def.ident().as_ref()) else {
                    continue;
                };
                items.push(ItemId::new(db, file, symbol, i as u32, ItemKind::Def, None));
            }
            ast::Decl::InductiveDecl(ind) => {
                let Some(symbol) = decl_symbol(db, ind.ident().as_ref()) else {
                    continue;
                };
                let inductive_id =
                    ItemId::new(db, file, symbol, i as u32, ItemKind::Inductive, None);
                items.push(inductive_id);

                if let Some(ctors) = ind.inductive_constructors() {
                    for (ci, ctor) in ctors.constructor_decl().enumerate() {
                        let Some(ctor_sym) = decl_symbol(db, ctor.ident().as_ref()) else {
                            continue;
                        };
                        items.push(ItemId::new(
                            db,
                            file,
                            ctor_sym,
                            ci as u32,
                            ItemKind::Constructor,
                            Some(inductive_id),
                        ));
                    }
                }
            }
        }
    }

    ItemTree::new(db, items)
}

fn decl_symbol<'a>(
    db: &'a dyn ElabDatabase,
    ident: Option<&ast::Identifier>,
) -> Option<Symbol<'a>> {
    let text = ident.as_ref()?.text()?;
    Some(Symbol::from_str(db, text))
}
