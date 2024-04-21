use logos::Logos;
use orco::ir::expression::Constant;

/// Token (number, word, operator, comment, etc.)
#[derive(Logos, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[logos(skip r"[ \t\n\f]+", error = Error)]
pub enum Token {
    // TODO: XID
    /// Identifier
    #[regex("[_a-zA-Z][_0-9a-zA-Z]*", |lex| lex.slice().to_owned())]
    #[regex("r#[_a-zA-Z][_0-9a-zA-Z]*", |lex| lex.slice()[2..].to_owned())]
    Ident(String),
    /// Operator
    #[token("(", |_| Operator::LParen)]
    #[token(")", |_| Operator::RParen)]
    #[token("[", |_| Operator::LBracket)]
    #[token("]", |_| Operator::RBracket)]
    #[token("{", |_| Operator::LBrace)]
    #[token("}", |_| Operator::RBrace)]
    #[token(",", |_| Operator::Comma)]
    #[token(":", |_| Operator::Colon)]
    #[token(";", |_| Operator::Semicolon)]
    #[token("->", |_| Operator::Arrow)]
    #[token("+", |_| Operator::Plus)]
    #[token("-", |_| Operator::Minus)]
    #[token("*", |_| Operator::Star)]
    #[token("/", |_| Operator::Slash)]
    #[token("%", |_| Operator::Percent)]
    Operator(Operator),
    /// Constant
    #[regex("[0-9][0-9_]*", |lex| parse_unsigned(lex.slice(), "", 10))]
    #[regex("-[0-9][0-9_]*", |lex| parse_signed(lex.slice(), "", 10))]
    #[regex("0b[0-1][0-1_]*", |lex| parse_unsigned(lex.slice(), "0b", 2))]
    #[regex("-0b[0-1][0-1_]*", |lex| parse_signed(lex.slice(), "0b", 2))]
    #[regex("0o[0-7][0-7_]*", |lex| parse_unsigned(lex.slice(), "0o", 8))]
    #[regex("-0o[0-7][0-7_]*", |lex| parse_signed(lex.slice(), "0o", 8))]
    #[regex("0x[0-9a-fA-F][0-9a-fA-F_]*", |lex| parse_unsigned(lex.slice(), "0x", 16))]
    #[regex("-0x[0-9a-fA-F][0-9a-fA-F_]*", |lex| parse_signed(lex.slice(), "0x", 16))]
    Constant(Constant),
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Ident(ident) => write!(f, "word '{}'", ident),
            Token::Operator(operator) => write!(f, "operator '{}'", operator),
            Token::Constant(value) => write!(f, "literal '{}'", value),
        }
    }
}

/// An error, that can occur during lexing process
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Error {
    /// Invalid (unknown) token
    #[default]
    InvalidToken,
    /// Integer out of bounds
    IntegerOutOfBounds(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidToken => write!(f, "Invalid token"),
            Error::IntegerOutOfBounds(s) => write!(f, "Integer out of bounds: {}", s),
        }
    }
}

fn parse_unsigned(slice: &str, prefix: &str, radix: u32) -> Result<Constant, Error> {
    u128::from_str_radix(&slice.strip_prefix(prefix).unwrap().replace('_', ""), radix)
        .map(|value| Constant::UnsignedInteger { value, size: None })
        .map_err(|_| Error::IntegerOutOfBounds(slice.to_owned()))
}

fn parse_signed(slice: &str, prefix: &str, radix: u32) -> Result<Constant, Error> {
    i128::from_str_radix(&slice.replace(prefix, "").replace('_', ""), radix)
        .map(|value| Constant::SignedInteger { value, size: None })
        .map_err(|_| Error::IntegerOutOfBounds(slice.to_owned()))
}

/// Operator (slash, comma, parens, +=, etc.)
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Operator {
    /// (
    LParen,
    /// )
    RParen,
    /// [
    LBracket,
    /// ]
    RBracket,
    /// {
    LBrace,
    /// }
    RBrace,
    /// ,
    Comma,
    /// :
    Colon,
    /// ;
    Semicolon,
    /// ->
    Arrow,
    /// +
    Plus,
    /// -
    Minus,
    /// *
    Star,
    /// /
    Slash,
    /// %
    Percent,
}

impl std::fmt::Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operator::LParen => write!(f, "("),
            Operator::RParen => write!(f, ")"),
            Operator::LBracket => write!(f, "["),
            Operator::RBracket => write!(f, "]"),
            Operator::LBrace => write!(f, "{{"),
            Operator::RBrace => write!(f, "}}"),
            Operator::Comma => write!(f, ","),
            Operator::Colon => write!(f, ":"),
            Operator::Semicolon => write!(f, ";"),
            Operator::Arrow => write!(f, "->"),
            Operator::Plus => write!(f, "+"),
            Operator::Minus => write!(f, "-"),
            Operator::Star => write!(f, "*"),
            Operator::Slash => write!(f, "/"),
            Operator::Percent => write!(f, "%"),
        }
    }
}

/// Parser, holds lexer, current token and error reporter. Used throughout parsing process
pub struct Parser<'source> {
    lexer: logos::Lexer<'source, Token>,
    peek: Option<Token>,
}

impl<'source> Parser<'source> {
    /// Create new parser from a source string
    pub fn new(source: &'source str) -> Self {
        Self {
            lexer: Token::lexer(source),
            peek: None,
        }
    }

    fn fill(&mut self) {
        if self.peek.is_none() {
            for token in self.lexer.by_ref() {
                match token {
                    Ok(token) => {
                        self.peek = Some(token);
                        break;
                    }
                    Err(err) => todo!("Error handling: {}", err),
                }
            }
        }
    }

    /// Get the current token
    pub fn peek(&mut self) -> Option<&Token> {
        self.fill();
        self.peek.as_ref()
    }

    /// Get the span for the current token
    pub fn span(&mut self) -> logos::Span {
        self.fill();
        self.lexer.span()
    }

    /// Check if the current token is EOF
    pub fn eof(&mut self) -> bool {
        self.fill();
        self.peek().is_none()
    }
}

impl Iterator for Parser<'_> {
    type Item = Token;

    /// Get the current token and advance to the next one
    fn next(&mut self) -> Option<Token> {
        self.fill();
        self.peek.take()
    }
}

impl<'source> Parser<'source> {
    /// Match a keyword, consume if matched
    pub fn match_keyword(&mut self, keyword: &str) -> bool {
        self.fill();
        if let Some(Token::Ident(ident)) = self.peek() {
            if ident == keyword {
                self.peek.take();
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Match an operator, consume if matched
    pub fn match_opertor(&mut self, operator: Operator) -> bool {
        self.fill();
        if self.peek == Some(Token::Operator(operator)) {
            self.peek.take();
            true
        } else {
            false
        }
    }

    /// Match an identifier, consume if matched
    pub fn match_ident(&mut self) -> Option<String> {
        self.fill();
        let peek = self.peek.take();
        if let Some(Token::Ident(ident)) = peek {
            Some(ident)
        } else {
            self.peek = peek;
            None
        }
    }

    /// Match a constant, consume if matched
    pub fn match_constant(&mut self) -> Option<Constant> {
        self.fill();
        let peek = self.peek.take();
        if let Some(Token::Constant(value)) = peek {
            Some(value)
        } else {
            self.peek = peek;
            None
        }
    }

    /// Report an error in form of "Expected {what}, got {current token}"
    pub fn expected_error(&mut self, what: &str) {
        if let Some(token) = self.peek() {
            todo!("Error: Expected {}, got {}", what, token);
        } else {
            todo!("Error: Expected {}", what);
        }
    }

    /// Expect an identifier to follow, if it is, consume and return it, else report an error
    /// "Expected {what}"
    pub fn expect_ident(&mut self, what: &str) -> Option<String> {
        self.fill();
        let peek = self.peek.take();
        if let Some(Token::Ident(ident)) = peek {
            Some(ident)
        } else {
            self.peek = peek;
            self.expected_error(what);
            None
        }
    }

    /// Expect an operator to follow, if it is, consume it, else report an error
    pub fn expect_operator(&mut self, operator: Operator) {
        self.fill();
        if self.peek == Some(Token::Operator(operator)) {
            self.peek.take();
        } else {
            self.expected_error(&format!("'{}'", operator));
        }
    }
}
