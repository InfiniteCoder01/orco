use logos::Logos;
use orco::diagnostics::*;
use orco::ir::expression::Constant;

pub mod unescape;

/// Token (number, word, operator, comment, etc.)
#[derive(Logos, Clone, Debug, PartialEq)]
#[logos(source = orco::Src, error = Error)]
#[logos(skip r"[ \t\n\f]+")]
#[logos(skip r"//.*")]
#[logos(skip r"/[*]([^*]|([*][^/]))*[*]+/")]
pub enum Token {
    // TODO: XID
    /// Identifier
    #[regex("[_a-zA-Z][_0-9a-zA-Z]*", |lex| ParsedIdent::new(Span(lex.source().clone(), lex.span()), false))]
    #[regex("r#[_a-zA-Z][_0-9a-zA-Z]*", |lex| ParsedIdent::new(Span(lex.source().clone(), lex.span().start + 2..lex.span().end), true))]
    Ident(ParsedIdent),
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
    #[token("=", |_| Operator::Equal)]
    #[token("==", |_| Operator::EqualEqual)]
    #[token("!=", |_| Operator::NotEqual)]
    #[token("<", |_| Operator::Lt)]
    #[token("<=", |_| Operator::LtEq)]
    #[token(">", |_| Operator::Gt)]
    #[token(">=", |_| Operator::GtEq)]
    Operator(Operator),
    /// Constant
    #[regex("[0-9][0-9_]*", |lex| parse_unsigned(lex.slice(), "", 10))]
    #[regex("0b[0-1][0-1_]*", |lex| parse_unsigned(lex.slice(), "0b", 2))]
    #[regex("0o[0-7][0-7_]*", |lex| parse_unsigned(lex.slice(), "0o", 8))]
    #[regex("0x[0-9a-fA-F][0-9a-fA-F_]*", |lex| parse_unsigned(lex.slice(), "0x", 16))]
    #[regex(r"[0-9]*[.][0-9]+", |lex| parse_float(lex.slice()))]
    #[regex("c\"([^\"]|\\\\.)*\"", |lex| parse_cstring(lex.slice()))]
    #[regex("c#\"([^\"]|\"[^#])*\"#", |lex| Constant::CString(lex.slice().as_bytes().to_vec(), Box::new(())))]
    Literal(Constant),
    /// Error
    Error,
}

fn parse_unsigned(slice: &str, prefix: &str, radix: u32) -> Result<Constant, Error> {
    u128::from_str_radix(&slice.strip_prefix(prefix).unwrap().replace('_', ""), radix)
        .map(|value| Constant::Integer {
            value,
            r#type: orco::ir::Type::IntegerWildcard,
            metadata: Box::new(()),
        })
        .map_err(|_| Error::IntegerOutOfBounds)
}

fn parse_float(slice: &str) -> Result<Constant, Error> {
    fast_float::parse(slice)
        .map(|value| Constant::Float {
            value,
            r#type: orco::ir::Type::FloatWildcard,
            metadata: Box::new(()),
        })
        .map_err(|_| Error::InvalidFloat)
}

fn parse_cstring(slice: &str) -> Result<Constant, Error> {
    let mut bytes = unescape::unescape(&slice[2..slice.len() - 1], 2)?;
    bytes.push(0);
    Ok(Constant::CString(bytes, Box::new(())))
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Ident(ident) => write!(f, "word '{}'", ident),
            Token::Operator(operator) => write!(f, "operator '{}'", operator),
            Token::Literal(value) => write!(f, "literal '{}'", value),
            Token::Error => write!(f, "<error>"),
        }
    }
}

/// Parsed identifier
#[derive(Clone, Debug, PartialEq)]
pub struct ParsedIdent {
	/// The identifier itself
    pub ident: Span,
	/// Was this identifier raw?
    pub raw: bool,
}

impl ParsedIdent {
	/// Create a new parsed identifier
    pub fn new(ident: Span, raw: bool) -> Self {
        Self { ident, raw }
    }
}

impl std::fmt::Display for ParsedIdent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.raw {
        	write!(f, "r#{}", self.ident)
        } else {
             write!(f, "{}", self.ident)
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
    IntegerOutOfBounds,
    /// Invalid float
    InvalidFloat,
    /// Invalid escape code
    InvalidEscapeCode(usize, &'static str, char),
    /// Invalid unicode codepoint
    InvalidUnicodeCodepoint(usize, u32),
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
    /// =
    Equal,
    /// ==
    EqualEqual,
    /// !=
    NotEqual,
    /// <
    Lt,
    /// <=
    LtEq,
    /// >
    Gt,
    /// >=
    GtEq,
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
            Operator::Equal => write!(f, "="),
            Operator::EqualEqual => write!(f, "=="),
            Operator::NotEqual => write!(f, "!="),
            Operator::Lt => write!(f, "<"),
            Operator::LtEq => write!(f, "<="),
            Operator::Gt => write!(f, ">"),
            Operator::GtEq => write!(f, ">="),
        }
    }
}

/// Parser, holds lexer, current token and error reporter. Used throughout parsing process
pub struct Parser<'a, R: ErrorReporter + ?Sized> {
    /// Error reporter
    pub reporter: &'a mut R,
    lexer: logos::Lexer<'a, Token>,
    peek: Option<Token>,
}

impl<'a, R: ErrorReporter + ?Sized> Parser<'a, R> {
    /// Create new parser from a source string
    pub fn new(source: &'a Src, reporter: &'a mut R) -> Self {
        Self {
            reporter,
            lexer: Token::lexer(source),
            peek: None,
        }
    }

    fn fill(&mut self) {
        if self.peek.is_none() {
            if let Some(token) = self.lexer.next() {
                self.peek = Some(match token {
                    Ok(token) => token,
                    Err(err) => {
                        let mut span = Span(self.lexer.source().clone(), self.lexer.span());
                        let message = match err {
                            Error::InvalidToken => "Invalid token".to_owned(),
                            Error::IntegerOutOfBounds => {
                                "Integer constant out of bounds".to_owned()
                            }
                            Error::InvalidFloat => "Invalid floating point constant".to_owned(),
                            Error::InvalidEscapeCode(offset, expected, got) => {
                                span.1 = span.1.start + offset..span.1.start + offset + 1;
                                format!("Invalid escape code: Expected {}, got '{}'", expected, got)
                            }
                            Error::InvalidUnicodeCodepoint(offset, codepoint) => {
                                span.1 = span.1.start + offset..span.1.start + offset + 1;
                                format!("Invalid unicode codepoint: 0x{:x}", codepoint)
                            }
                        };
                        self.reporter.report(
                            Report::build(ReportKind::Error)
                                .with_message(message)
                                .with_label(
                                    Label::new(span)
                                        .with_message("Here")
                                        .with_color(colors::Label),
                                )
                                .finish(),
                        );
                        Token::Error
                    }
                });
            }
        }
    }

    /// Get the current token
    pub fn peek(&mut self) -> Option<&Token> {
        self.fill();
        self.peek.as_ref()
    }

    /// Get the span for the current token
    pub fn span(&mut self) -> Span {
        self.fill();
        Span(self.lexer.source().clone(), self.lexer.span())
    }

    /// Check if the current token is EOF
    pub fn eof(&mut self) -> bool {
        self.fill();
        self.peek().is_none()
    }

    /// Get the span from the start to the end of the current token
    pub fn span_from(&mut self, start: usize) -> Span {
        Span(self.lexer.source().clone(), start..self.lexer.span().end)
    }

    /// Wrap an object in [`orco::Spanned`], starting at start, ending at the current position
    pub fn wrap_span<T>(&mut self, object: T, start: usize) -> Spanned<T> {
        Spanned::new(object, self.span_from(start))
    }

    /// Get a zero-length span at the current position
    pub fn point_span(&mut self) -> Span {
        let start = self.span().1.start;
        self.span_from(start)
    }

    /// Wrap an object in [`orco::Spanned`], span will be a zero-length point at the current
    /// position
    pub fn wrap_point<T>(&mut self, object: T) -> Spanned<T> {
        let start = self.span().1.start;
        self.wrap_span(object, start)
    }
}

impl<R: ErrorReporter + ?Sized> Iterator for Parser<'_, R> {
    type Item = Token;

    /// Get the current token and advance to the next one
    fn next(&mut self) -> Option<Token> {
        self.fill();
        self.peek.take()
    }
}

impl<'source, R: ErrorReporter + ?Sized> Parser<'source, R> {
    /// Match a keyword, consume if matched
    pub fn match_keyword(&mut self, keyword: &str) -> bool {
        self.fill();
        if let Some(Token::Ident(ident)) = self.peek() {
            if !ident.raw && ident.ident.as_ref() == keyword {
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
    pub fn match_operator(&mut self, operator: Operator) -> bool {
        self.fill();
        if self.peek == Some(Token::Operator(operator)) {
            self.peek.take();
            true
        } else {
            false
        }
    }

    /// Match an identifier, consume if matched
    pub fn match_ident(&mut self) -> Option<Span> {
        self.fill();
        let peek = self.peek.take();
        if let Some(Token::Ident(ident)) = peek {
            Some(ident.ident)
        } else {
            self.peek = peek;
            None
        }
    }

    /// Match a literal, consume if matched
    pub fn match_literal(&mut self) -> Option<Spanned<Constant>> {
        self.fill();
        let span = self.span();
        let peek = self.peek.take();
        if let Some(Token::Literal(value)) = peek {
            Some(Spanned::new(value, span))
        } else {
            self.peek = peek;
            None
        }
    }

    /// Match an error, consume if matched
    pub fn match_error(&mut self) -> Option<Span> {
        self.fill();
        let span = self.span();
        if self.peek == Some(Token::Error) {
            self.peek.take();
            Some(span)
        } else {
            None
        }
    }

    /// Report an error in form of "Expected {what}, got {current token}"
    pub fn expected_error(&mut self, what: &str) {
        let message = if let Some(token) = self.peek() {
            format!("Expected {}, got {}", what, token)
        } else {
            format!("Expected {}", what)
        };
        let span = self.span();
        self.reporter.report(
            Report::build(ReportKind::Error)
                .with_message(message)
                .with_label(
                    Label::new(span)
                        .with_message("Here")
                        .with_color(Color::Blue),
                )
                .finish(),
        );
    }

    /// Expect an identifier to follow, if it is, consume and return it, else report an error
    /// "Expected {what}"
    pub fn expect_ident(&mut self, what: &str) -> Option<Span> {
        self.fill();
        let peek = self.peek.take();
        if let Some(Token::Ident(ident)) = peek {
            Some(ident.ident)
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
