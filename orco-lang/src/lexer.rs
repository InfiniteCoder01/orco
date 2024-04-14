use logos::{Logos, SpannedIter};

/// An error, that can occur during lexing process
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Error {
    /// Invalid (unknown) token
    #[default]
    InvalidToken,
}

/// Token (number, word, operator, comment, etc.)
#[derive(Logos, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[logos(skip r"[ \t\n\f]+", error = Error)]
pub enum Token {
    // TODO: XID
    /// Identifier
    #[regex("[_a-zA-Z][_0-9a-zA-Z]*", |lex| lex.slice().to_owned())]
    Ident(String),
    /// Operator
    #[token("(", |_| Operator::LParen)]
    #[token(")", |_| Operator::RParen)]
    Operator(Operator),
}

/// Operator (slash, comma, parens, +=, etc.)
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Operator {
    /// (
    LParen,
    /// )
    RParen,
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
