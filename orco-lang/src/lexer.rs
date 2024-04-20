use logos::{Logos, SpannedIter};
use orco::ir::expression::Constant;

/// Token (number, word, operator, comment, etc.)
#[derive(Logos, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[logos(skip r"[ \t\n\f]+", error = Error)]
pub enum Token {
    /// Keyword
    #[token("fn", |_| Keyword::Fn)]
    #[token("extern", |_| Keyword::Extern)]
    #[token("return", |_| Keyword::Return)]
    Keyword(Keyword),
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

/// Keyword
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Keyword {
    /// fn
    Fn,
    /// extern
    Extern,
    /// return
    Return,
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
}

// * Lexer
/// Spanned token
pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

/// Lexer (iterator over tokens)
pub struct Lexer<'input> {
    token_stream: SpannedIter<'input, Token>,
}

impl<'input> Lexer<'input> {
    /// Create a new lexer for the given input.
    /// Based on [`logos::SpannedIter`], which I think it does tokenization lazily
    pub fn new(input: &'input str) -> Self {
        Self {
            token_stream: Token::lexer(input).spanned(),
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Token, usize, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.token_stream
            .next()
            .map(|(token, span)| Ok((span.start, token?, span.end)))
    }
}
