pub mod emit;

use anyhow::{
    Result,
    bail,
};
use heck::ToSnakeCase;
use ungrammar::{
    Grammar,
    NodeData,
    Rule,
};

pub struct AstSrc {
    pub nodes: Vec<AstNodeSrc>,
    pub enums: Vec<AstEnumSrc>,
}

pub struct AstNodeSrc {
    pub name: String,
    pub fields: Vec<Field>,
}

pub enum Field {
    Token {
        name: String,
        kind: String,
    },
    Node {
        name: String,
        ty: String,
        cardinality: Cardinality,
    },
}

pub enum Cardinality {
    Optional,
    Many,
}

pub struct AstEnumSrc {
    pub name: String,
    pub variants: Vec<String>,
}

pub fn lower(grammar: &Grammar) -> Result<AstSrc> {
    let mut nodes = Vec::new();
    let mut enums = Vec::new();
    let mut tokens = std::collections::BTreeSet::new();

    for node in grammar.iter() {
        let data: &NodeData = &grammar[node];
        let name = data.name.clone();

        if let Some(variants) = lower_alt_of_nodes(grammar, &data.rule) {
            enums.push(AstEnumSrc { name, variants });
        } else {
            let mut fields = Vec::new();
            lower_rule(grammar, &data.rule, None, &mut fields, &mut tokens)?;
            nodes.push(AstNodeSrc { name, fields });
        }
    }

    Ok(AstSrc { nodes, enums })
}

fn lower_alt_of_nodes(grammar: &Grammar, rule: &Rule) -> Option<Vec<String>> {
    match rule {
        Rule::Alt(alts) => {
            let mut out = Vec::with_capacity(alts.len());
            for alt in alts {
                let Rule::Node(node) = alt else { return None };
                out.push(grammar[*node].name.clone());
            }
            Some(out)
        }
        Rule::Node(node) => Some(vec![grammar[*node].name.clone()]),
        _ => None,
    }
}

fn lower_rule(
    grammar: &Grammar,
    rule: &Rule,
    label: Option<&str>,
    fields: &mut Vec<Field>,
    tokens: &mut std::collections::BTreeSet<String>,
) -> Result<()> {
    match rule {
        Rule::Labeled { label, rule } => {
            lower_rule(grammar, rule, Some(label), fields, tokens)?;
        }
        Rule::Node(n) => {
            let ty = grammar[*n].name.clone();
            let name = label.map_or_else(|| ty.to_snake_case(), str::to_string);
            fields.push(Field::Node {
                name,
                ty,
                cardinality: Cardinality::Optional,
            });
        }
        Rule::Token(t) => {
            let text = grammar[*t].name.clone();
            tokens.insert(text.clone());
            let kind = token_kind(&text)?;
            let name = match label {
                Some(l) => l.to_string(),
                None => token_accessor(&text)?,
            };
            fields.push(Field::Token { name, kind });
        }
        Rule::Seq(rules) => {
            for r in rules {
                lower_rule(grammar, r, None, fields, tokens)?;
            }
        }
        Rule::Alt(_) => {
            bail!("internal alt; only top-level enum-style alts supported");
        }
        Rule::Opt(inner) => {
            lower_rule(grammar, inner, label, fields, tokens)?;
        }
        Rule::Rep(inner) => {
            let before = fields.len();
            lower_rule(grammar, inner, label, fields, tokens)?;
            for f in &mut fields[before..] {
                if let Field::Node { cardinality, .. } = f {
                    *cardinality = Cardinality::Many;
                }
            }
        }
    }
    Ok(())
}

fn token_accessor(text: &str) -> Result<String> {
    let base = match text {
        "(" => "l_paren",
        ")" => "r_paren",
        "{" => "l_brace",
        "}" => "r_brace",
        "[" => "l_bracket",
        "]" => "r_bracket",
        "," => "comma",
        ";" => "semicolon",
        "." => "dot",
        "->" => "arrow",
        ":" => "colon",
        "::" => "colon_colon",
        ":=" => "def_eq",
        "ident" => "ident",
        "number_lit" => "number_lit",
        "string_lit" => "string_lit",
        "def" => "def_kw",
        "let" => "let_kw",
        other => anyhow::bail!("no token accessor mapping for token `{other}`"),
    };
    Ok(base.into())
}
fn token_kind(text: &str) -> Result<String> {
    Ok(match text {
        "(" => "LParen",
        ")" => "RParen",
        "{" => "LBrace",
        "}" => "RBrace",
        "[" => "LBracket",
        "]" => "RBracket",
        "," => "Comma",
        ";" => "Semicolon",
        "." => "Dot",
        "->" => "RArrow",
        ":" => "Colon",
        ":=" => "DefEq",
        "::" => "ColonColon",
        "ident" => "Identifier",
        "number_lit" => "NumberLit",
        "string_lit" => "StringLit",
        "def" => "DefKw",
        "let" => "LetKw",
        other => anyhow::bail!("no SyntaxKind mapping for token `{other}`"),
    }
    .into())
}
