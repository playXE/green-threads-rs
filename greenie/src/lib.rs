extern crate proc_macro;

extern crate syn;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::*;
#[macro_use]
extern crate quote;
use syn::fold::Fold;
#[proc_macro_attribute]
pub fn greenify(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input: ItemFn = parse_macro_input!(input as ItemFn);
    let new_stmt: Stmt = parse(TokenStream::from(quote::quote!(green_yield!();))).unwrap();
    let f = GreenFolder::new(new_stmt).fold_item_fn(input);
    TokenStream::from(f.to_token_stream())
}

struct GreenFolder {
    stmt: Stmt,
}

impl GreenFolder {
    pub fn new(s: Stmt) -> GreenFolder {
        Self { stmt: s }
    }

    fn gen_block(&self, old_block: &Block) -> Block {
        /*let mut new_stmts = old_block.stmts.clone();
        new_stmts.insert(0, self.stmt.clone());
        Block {
            stmts: new_stmts,
            brace_token: old_block.brace_token.clone(),
        };*/
        parse_quote! {
            {
                green_yield!();
                #old_block
            }
        }
    }
}

impl syn::fold::Fold for GreenFolder {
    fn fold_expr(&mut self, mut expr: Expr) -> Expr {
        let new_expr = match expr {
            Expr::Loop(ExprLoop {
                body,
                attrs,
                label,
                loop_token,
            }) => Expr::Loop(ExprLoop {
                body: self.gen_block(&body),
                attrs,
                label,
                loop_token,
            }),
            Expr::ForLoop(ExprForLoop { ref mut body, .. }) => {
                *body = self.gen_block(&body);
                expr
            }
            Expr::While(ExprWhile { ref mut body, .. }) => {
                *body = self.gen_block(&body);
                expr
            }
            _ => syn::fold::fold_expr(self, expr),
        };
        new_expr
    }
    fn fold_block(&mut self, i: Block) -> Block {
        self.gen_block(&i)
    }

    fn fold_item_fn(&mut self, mut i: ItemFn) -> ItemFn {
        let mut stmts = vec![];
        for stmt in i.block.stmts.iter() {
            stmts.push(syn::fold::fold_stmt(self, stmt.clone()));
        }
        i.block.stmts = stmts;
        i.block = Box::new(self.gen_block(&*i.block));
        i
    }
}
