mod args;

use std::fs;

use clap::Parser;
use syn::{Item, Block, Stmt, Local, Expr, Pat, Member};
use syn::__private::ToTokens;

fn main() {
    let args = args::Args::parse();

    let contents = fs::read_to_string(args.file).unwrap();

    let ast = syn::parse_file(&contents).expect("unable to parse file");

    // println!("{:#?}", ast.items);
    for item in ast.items {
        parse_item(item);
    }
}

fn parse_item(item: Item) {
    match item {
        Item::Mod(i) => {
            println!("Adding {} as a module", i.ident);
        }
        Item::Fn(i) => {
            println!("Entering {} function", i.sig.ident);
            parse_block(i.block);
        }
        _ => {}
    }
}

fn parse_block(block: Box<Block>) {
    for stmt in block.stmts {
        parse_statement(stmt);
    }
}

fn parse_statement(stmt: Stmt) {
    match stmt {
        Stmt::Local(local) => println!("{}", parse_local(local)),
        Stmt::Expr(expr, _) => println!("{}", parse_expression(expr)),
        Stmt::Item(item) => parse_item(item),
        _ => {}
    }
}

fn parse_local(local: Local) -> String {
    if let Pat::Ident(pat) = local.pat {
        format!("Declared a local variable: {}", pat.ident)
    } else {
        String::new()
    }
}

fn parse_expression(expr: Expr) -> String {
    match expr {
        Expr::Path(e) => {
            if let Some(ident) = e.path.get_ident() {
                format!("{}", ident)
            } else {
                format!("Found a path: {:?}", e.path)
            }
        }
        Expr::Assign(e) => {
            format!(
                "Assigning to variable: {}\nThe value being assigned: {}",
                parse_expression(*e.left),
                parse_expression(*e.right)
            )
        }
        Expr::MethodCall(e) => {
            let mut output = format!(
                "Calling method {} on object: {}",
                e.method, parse_expression(*e.receiver)
            );
            for argument in e.args {
                output.push_str(&format!("\nWith argument: {}", parse_expression(argument)));
            }
            output
        }
        Expr::Call(e) => {
            let mut output = format!("Function call: {}", parse_expression(*e.func));
            for argument in e.args {
                output.push_str(&format!("\nWith argument: {}", parse_expression(argument)));
            }
            output
        }
        Expr::Binary(e) => {
            format!(
                "Binary operation: {} {} {}",
                parse_expression(*e.left),
                e.op.into_token_stream(),
                parse_expression(*e.right)
            )
        }
        Expr::Lit(e) => {
            format!("Literal: {:?}", e.lit)
        }
        Expr::Match(e) => {
            let mut output = format!("Match expression for: {}\n", parse_expression(*e.expr));
            for arm in e.arms {
                let arm_expr = parse_expression(*arm.body.clone());
                output.push_str(&format!("\nCase {} => {}", parse_pattern(arm.pat), arm_expr));
            }
            output
        }
        Expr::Block(e) => {
            parse_block(Box::from(e.block));
            String::new()
        }
        Expr::Macro(expr_macro) => {
            let macro_name = &expr_macro.mac.path.segments.last().unwrap().ident.to_string();
            let tokens: Vec<String> = expr_macro.mac.tokens.to_string().split_whitespace().map(|s| s.to_string()).collect();
            format!("Macro call to {} with tokens: {:?}", macro_name, tokens)
        }
        Expr::ForLoop(expr_for_loop) => {
            let pattern = parse_pattern(*expr_for_loop.pat.clone());
            let expression = parse_expression(*expr_for_loop.expr.clone());
            parse_block(Box::new(expr_for_loop.body.clone()));
            format!("For loop with pattern {} in expression {} with body", pattern, expression)
        }        
        _ => {
            format!("Other expression: {:?}", expr)
            // String::new()
        }
    }
}

fn parse_pattern(pat: Pat) -> String {
    match pat {
        Pat::Wild(_) => "_".to_string(),
        Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
        Pat::Tuple(tuple) => {
            let elems = tuple.elems.iter().map(|e| parse_pattern(e.clone())).collect::<Vec<String>>().join(", ");
            format!("({})", elems)
        },
        Pat::Struct(pat_struct) => {
            let fields = pat_struct.fields.iter().map(|f| {
                let member = match &f.member {
                    Member::Named(ident) => ident.to_string(),
                    Member::Unnamed(index) => index.index.to_string(),
                };
                let pat = parse_pattern(*f.pat.clone());
                match &f.colon_token {
                    Some(_) => format!("{}: {}", member, pat),
                    None => format!("{} @ {}", member, pat),
                }
            }).collect::<Vec<String>>().join(", ");
            format!("{} {{ {} }}", pat_struct.path.segments.iter().last().unwrap().ident, fields)
        }        
        Pat::Path(pat_path) => pat_path.path.get_ident().unwrap().to_string(),
        Pat::Range(pat_range) => format!("{}..{}", parse_expression(*pat_range.start.unwrap()), parse_expression(*pat_range.end.unwrap())),
        Pat::Reference(pat_ref) => format!("&{}", parse_pattern(*pat_ref.pat)),
        Pat::Or(pat_or) => pat_or.cases.iter().map(|c| parse_pattern(c.clone())).collect::<Vec<String>>().join(" | "),
        Pat::Lit(pat_lit) => format!("{:?}", pat_lit.lit),
        _ => String::new(),
    }
}