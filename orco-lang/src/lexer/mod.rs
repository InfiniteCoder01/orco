use logos::Logos;
use orco::diagnostics::*;
use orco::ir::expression::Constant;

pub mod unescape;

/// Wrapper for [`orco::Src`], that implements [`logos::Source`]
pub struct Source(pub orco::Src);

impl std::ops::Deref for Source {
    type Target = orco::Src;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl logos::Source for Source {
    type Slice<'a> = &'a str;

    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    fn read<'a, Chunk>(&'a self, offset: usize) -> Option<Chunk>
    where
        Chunk: logos::source::Chunk<'a>,
    {
        self.0.read(offset)
    }

    #[inline]
    unsafe fn read_unchecked<'a, Chunk>(&'a self, offset: usize) -> Chunk
    where
        Chunk: logos::source::Chunk<'a>,
    {
        self.0.read_unchecked(offset)
    }

    #[inline]
    fn slice(&self, range: std::ops::Range<usize>) -> Option<Self::Slice<'_>> {
        self.0.slice(range)
    }

    #[inline]
    unsafe fn slice_unchecked(&self, range: std::ops::Range<usize>) -> Self::Slice<'_> {
        debug_assert!(
            range.start <= self.len() && range.end <= self.len(),
            "Reading out of bounds {:?} for {}!",
            range,
            self.len()
        );

        self.0.get_unchecked(range)
    }

    #[inline]
    fn is_boundary(&self, index: usize) -> bool {
        self.0.is_boundary(index)
    }
}

/// Token (number, word, operator, comment, etc.)
#[derive(Logos, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[logos(source = Source, error = Error)]
#[logos(skip r"[ \t\n\f]+")]
#[logos(skip r"//.*")]
#[logos(skip r"/[*]([^*]|([*][^/]))*[*]+/")]
pub enum Token {
    // TODO: XID
    /// Identifier
    #[regex("[_a-zA-Z][_0-9a-zA-Z]*", |lex| Span(lex.source().0.clone(), lex.span()))]
    #[regex("r#[_a-zA-Z][_0-9a-zA-Z]*", |lex| Span(lex.source().0.clone(), lex.span().start + 2..lex.span().end))]
    Ident(Span),
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
    #[regex("c\"([^\"]|\\\\.)*\"", |lex| parse_cstring(lex.slice()))]
    #[regex("c#\"([^\"]|\"[^#])*\"#", |lex| Constant::CString(lex.slice().as_bytes().to_vec()))]
    Constant(Constant),
    /// Error
    Error,
}

fn parse_unsigned(slice: &str, prefix: &str, radix: u32) -> Result<Constant, Error> {
    u128::from_str_radix(&slice.strip_prefix(prefix).unwrap().replace('_', ""), radix)
        .map(|value| Constant::Integer {
            value,
            r#type: orco::ir::Type::IntegerWildcard,
        })
        .map_err(|_| Error::IntegerOutOfBounds)
}

fn parse_cstring(slice: &str) -> Result<Constant, Error> {
    let mut bytes = unescape::unescape(&slice[2..slice.len() - 1], 2)?;
    bytes.push(0);
    Ok(Constant::CString(bytes))
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Ident(ident) => write!(f, "word '{}'", ident),
            Token::Operator(operator) => write!(f, "operator '{}'", operator),
            Token::Constant(value) => write!(f, "literal '{}'", value),
            Token::Error => write!(f, "<error>"),
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
    pub fn new(source: &'a Source, reporter: &'a mut R) -> Self {
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
                        let mut span = Span((**self.lexer.source()).clone(), self.lexer.span());
                        let mut colors = ColorGenerator::new();
                        let report = orco::diagnostics::Report::build(
                            orco::diagnostics::ReportKind::Error,
                            self.lexer.source().0.clone(),
                            span.1.start,
                        );
                        let report = match err {
                            Error::InvalidToken => report
                                .with_code("OL0")
                                .with_message("Invalid token")
                                .with_label(
                                    Label::new(span)
                                        .with_message("Invalid token")
                                        .with_color(colors.next()),
                                ),
                            Error::IntegerOutOfBounds => report
                                .with_code("OL1")
                                .with_message("Integer out of bounds")
                                .with_label(
                                    Label::new(span)
                                        .with_message("This constant")
                                        .with_color(colors.next()),
                                ),
                            Error::InvalidEscapeCode(offset, expected, got) => {
                                span.1 = span.1.start + offset..span.1.start + offset + 1;
                                report
                                    .with_code("OL2")
                                    .with_message("Invalid escape code")
                                    .with_label(
                                        Label::new(span)
                                            .with_message(format!(
                                                "Expected {}, got '{}'",
                                                expected, got
                                            ))
                                            .with_color(colors.next()),
                                    )
                            }
                            Error::InvalidUnicodeCodepoint(offset, codepoint) => {
                                span.1 = span.1.start + offset..span.1.start + offset + 1;
                                report
                                    .with_code("OL3")
                                    .with_message("Invalid unicode codepoint")
                                    .with_label(
                                        Label::new(span)
                                            .with_message(format!(
                                                "Invalid unicode codepoint: 0x{:x}",
                                                codepoint
                                            ))
                                            .with_color(colors.next()),
                                    )
                            }
                        }
                        .finish();
                        self.reporter.report(report);
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
        Span((**self.lexer.source()).clone(), self.lexer.span())
    }

    /// Check if the current token is EOF
    pub fn eof(&mut self) -> bool {
        self.fill();
        self.peek().is_none()
    }

    /// Get the span from the start to the end of the current token
    pub fn span_from(&mut self, start: usize) -> Span {
        Span((**self.lexer.source()).clone(), start..self.span().1.start)
    }

    /// Wrap an object in [`orco::Spanned`], starting at start, ending at the current position
    pub fn wrap_span<T>(&mut self, object: T, start: usize) -> Spanned<T> {
        Spanned {
            inner: object,
            span: self.span_from(start),
        }
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
            if ident.as_ref() == keyword {
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
            Some(ident)
        } else {
            self.peek = peek;
            None
        }
    }

    /// Match a constant, consume if matched
    pub fn match_constant(&mut self) -> Option<Spanned<Constant>> {
        self.fill();
        let span = self.span();
        let peek = self.peek.take();
        if let Some(Token::Constant(value)) = peek {
            Some(Spanned { inner: value, span })
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
            format!("Error: Expected {}", what)
        };
        let mut colors = ColorGenerator::new();
        let report = Report::build(
            ReportKind::Error,
            self.lexer.source().0.clone(),
            self.span().1.start,
        )
        .with_message(message)
        .with_label(
            Label::new(self.span())
                .with_message("Here")
                .with_color(colors.next()),
        )
        .finish();
        self.reporter.report(report);
    }

    /// Expect an identifier to follow, if it is, consume and return it, else report an error
    /// "Expected {what}"
    pub fn expect_ident(&mut self, what: &str) -> Option<Span> {
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
