use proc_macro::{Punct, TokenStream, TokenTree};
use psilib::grammar::*;

#[proc_macro]
pub fn psi_rulepart(item: TokenStream) -> TokenStream {
    let mut iter = item.into_iter();
    match iter.next() {
        Some(TokenTree::Ident(id)) => format!(
            "{{
                use psilib::grammar::*;
                RulePart::Rule(stringify!({id}).to_owned())
            }}"
        )
        .parse()
        .unwrap(),
        Some(TokenTree::Literal(lit)) => format!(
            "{{
                use psilib::grammar::*;
                RulePart::Literal({lit}.to_owned())
            }}"
        )
        .parse()
        .unwrap(),

        _ => panic!(),
    }
}

#[proc_macro]
pub fn psi_rule(item: TokenStream) -> TokenStream {
    let mut parts = String::new();

    let mut iter = item.into_iter();

    while let Some(tt) = iter.next() {
        match tt {
            ident @ TokenTree::Ident(_) if &ident.to_string() == "_" => {
                parts.push_str("RulePart::Empty");
                break;
            }
            tt @ TokenTree::Ident(_) | tt @ TokenTree::Literal(_) => {
                let ts = TokenStream::from_iter([tt]);

                parts.push_str(&psi_rulepart(ts).to_string());
                parts.push_str(",");
            }

            _ => panic!("unexpected token: '{tt}'"),
        }
    }

    format!(
        "{{
        use psilib::grammar::*;
        
        RuleDef {{
            parts: vec![{parts}]
        }}
    }}"
    )
    .parse()
    .unwrap()
}


#[proc_macro]
pub fn psi_rule_entry(item: TokenStream) -> TokenStream {
    let mut iter = item.into_iter();
    
    let mut defs = String::new();
    let precedence = 0;

    match iter.next() {
        Some(TokenTree::Literal(lit)) => {
            
        } 
    }

    format!("{{
        use psilib::grammar::*;
        RuleEntry {
            precedence: 0,
            definitions: vec![{defs}]
        }
    }}").parse().unwrap()
}

#[proc_macro]
pub fn psi(ts: TokenStream) -> TokenStream {
    todo!()
}
