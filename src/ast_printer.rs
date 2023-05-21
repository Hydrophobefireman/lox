// use crate::expr::{Binary, Expr, Grouping, Literal, Unary, Visitor};
// pub struct AstPrinter {}

// impl AstPrinter {
//     fn unwrap_value(&self, value: &Box<dyn std::any::Any>) -> Result<String, ()> {
//         if let Some(num) = value.downcast_ref::<f32>() {
//             Ok(format!("{}", num))
//         } else if let Some(string) = value.downcast_ref::<String>() {
//             Ok(string.clone())
//         } else if let Some(str) = value.downcast_ref::<&str>() {
//             Ok((*str).into())
//         } else {
//             Err(())
//         }
//     }

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
//             None => "nil".into(),
//             Some(val) => format!("{:?}", self.unwrap_value(val).unwrap_or("err".into())),
//         }
//     }

//     fn Unary(&self, e: &Unary) -> String {
//         self.parenthesize(&e.operator.lexeme, &[&e.right])
//     }
// }
