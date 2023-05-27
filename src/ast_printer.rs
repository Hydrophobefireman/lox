// use crate::expr::{Binary, Expr, Grouping, Literal, Unary, Visitor};
// pub struct AstPrinter {}

// impl AstPrinter {
//     pub fn new() -> Self {
//         AstPrinter {}
//     }
//     fn parenthesize(&self, name: &str, exp: &[&Expr]) -> String {
//         let a = exp
//             .iter()
//             .map(|e| e.accept::<AstPrinter, String>(self))
//             .collect::<Vec<_>>()
//             .join(" ");
//         format!("({name} {a})")
//     }
//     pub fn print(&self, e: Expr) -> String {
//         e.accept(self)
//     }
// }

// impl Visitor<String> for AstPrinter {
//     fn Binary(&self, e: &Binary) -> String {
//         self.parenthesize(&e.operator.lexeme, &[&e.left, &e.right])
//     }

//     fn Grouping(&self, e: &Grouping) -> String {
//         self.parenthesize("group", &[&e.expression])
//     }

//     fn Literal(&self, e: &Literal) -> String {
//         match &e.value {
//             crate::tokens::token::LiteralType::String(s) => s.clone(),
//             crate::tokens::token::LiteralType::Float(n) => n.to_string(),
//             crate::tokens::token::LiteralType::True => "true".into(),
//             crate::tokens::token::LiteralType::False => "false".into(),
//             crate::tokens::token::LiteralType::Nil => "nil".into(),
//             crate::tokens::token::LiteralType::None => "(?)".into(),
//         }
//     }

//     fn Unary(&self, e: &Unary) -> String {
//         self.parenthesize(&e.operator.lexeme, &[&e.right])
//     }
// }
