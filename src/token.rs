use crate::token_type::TokenType;
#[derive(Debug)]
pub struct Token {
    ty: TokenType,
    lexeme: String,
    literal: Option<Box<dyn std::any::Any>>,
    line: usize,
}

impl Token {
    pub fn new(
        ty: TokenType,
        lexeme: String,
        literal: Option<Box<dyn std::any::Any>>,
        line: usize,
    ) -> Self {
        Token {
            ty,
            lexeme,
            literal,
            line,
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {} {:?} {}",
            self.ty, self.lexeme, self.literal, self.line
        )
    }
}
