use std::collections::HashMap;

use anyhow::Result;
use proc_macro2::{
    Ident,
    Span,
    TokenStream,
};
use quote::{
    format_ident,
    quote,
};

fn ident(name: &str) -> Ident {
    syn::parse_str::<Ident>(name).unwrap_or_else(|_| Ident::new_raw(name, Span::call_site()))
}

use super::{
    AstSrc,
    Cardinality,
    Field,
};
use crate::codegen::ast::{
    AstEnumSrc,
    AstNodeSrc,
};

pub fn emit(src: &AstSrc) -> Result<String> {
    let nodes = src.nodes.iter().map(emit_node);
    let enums = src.enums.iter().map(emit_enum);

    let file: TokenStream = quote! {
        use crate::traits::{AstNode, AstChildren, child, children, token, token_text};
        use syntax::{ResolvedNode, ResolvedToken, kind::SyntaxKind};

        #(#nodes)*
        #(#enums)*
    };

    let parsed: syn::File = syn::parse2(file)?;
    let mut out = prettyplease::unparse(&parsed);

    out = format!("// @generated\n{out}");
    Ok(out)
}

fn emit_node(n: &AstNodeSrc) -> TokenStream {
    let name = format_ident!("{}", n.name);
    let kind = format_ident!("{}", n.name);

    let mut counts: HashMap<&str, usize> = HashMap::new();
    for f in &n.fields {
        if let Field::Node {
            ty,
            cardinality: Cardinality::Optional,
            ..
        } = f
        {
            *counts.entry(ty.as_str()).or_default() += 1;
        }
    }
    let mut seen: HashMap<&str, usize> = HashMap::new();
    let positions: Vec<Option<usize>> = n
        .fields
        .iter()
        .map(|f| match f {
            Field::Node {
                ty,
                cardinality: Cardinality::Optional,
                ..
            } if counts.get(ty.as_str()).copied().unwrap_or(0) > 1 => {
                let idx = *seen.entry(ty.as_str()).and_modify(|c| *c += 1).or_insert(0);
                Some(idx)
            }
            _ => None,
        })
        .collect();

    let accessors = n
        .fields
        .iter()
        .zip(positions)
        .map(|(f, pos)| emit_field(f, pos));

    let text_fn = match n.fields.as_slice() {
        [Field::Token { kind, .. }] => {
            let k = ident(kind);
            Some(quote! {
                pub fn text(&self) -> Option<&str> {
                    token_text(&self.0, SyntaxKind::#k)
                }
            })
        }
        _ => None,
    };

    quote! {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        #[repr(transparent)]
        pub struct #name(ResolvedNode);

        impl AstNode for #name {
            fn can_cast(k: SyntaxKind) -> bool { k == SyntaxKind::#kind }
            fn cast(node: ResolvedNode) -> Option<Self> {
                Self::can_cast(node.kind()).then_some(Self(node))
            }
            fn syntax(&self) -> &ResolvedNode { &self.0 }
        }

        impl #name {
            #(#accessors)*
            #text_fn
        }
    }
}

fn emit_field(f: &Field, position: Option<usize>) -> TokenStream {
    match f {
        Field::Node {
            name,
            ty,
            cardinality: Cardinality::Optional,
        } => {
            let n = ident(name);
            let t = ident(ty);
            if let Some(idx) = position {
                quote! {
                    pub fn #n(&self) -> Option<#t> { children::<#t>(&self.0).nth(#idx) }
                }
            } else {
                quote! { pub fn #n(&self) -> Option<#t> { child(&self.0) } }
            }
        }
        Field::Node {
            name,
            ty,
            cardinality: Cardinality::Many,
        } => {
            let n = ident(name);
            let t = ident(ty);
            quote! { pub fn #n(&self) -> AstChildren<'_, #t> { children(&self.0) } }
        }
        Field::Token { name, kind } => {
            let n = ident(name);
            let k = ident(kind);
            quote! {
                pub fn #n(&self) -> Option<ResolvedToken> {
                    token(&self.0, SyntaxKind::#k)
                }
            }
        }
    }
}

fn emit_enum(e: &AstEnumSrc) -> TokenStream {
    let name = format_ident!("{}", e.name);
    let variants = e.variants.iter().map(|v| format_ident!("{}", v));
    let variants2 = variants.clone();
    let variants3 = variants.clone();
    let variants4 = variants.clone();
    let variants5 = variants.clone();

    quote! {
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub enum #name {
            #(#variants2(#variants2)),*
        }

        impl AstNode for #name {
            fn can_cast(k: SyntaxKind) -> bool {
                #(#variants3::can_cast(k))||*
            }
            fn cast(node: ResolvedNode) -> Option<Self> {
                #(if let Some(it) = #variants4::cast(node.clone()) {
                    return Some(Self::#variants(it));
                })*
                None
            }
            fn syntax(&self) -> &ResolvedNode {
                match self {
                    #(Self::#variants5(it) => it.syntax()),*
                }
            }
        }
    }
}
