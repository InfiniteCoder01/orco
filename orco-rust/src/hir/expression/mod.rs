use super::{Context, ob};

pub mod block;
pub use block::Block;

pub mod literal;
pub use literal::Literal;

pub mod operator;
pub use operator::Operator;

pub mod symbol;
pub use symbol::Symbol;

#[derive(Clone, Debug)]
pub enum Expression {
    Literal(Literal),
    Symbol(Symbol),
    Operator(Operator),
    Block(Block),
}

impl Expression {
    pub fn parse(value: &syn::Expr, path: &crate::hir::Path) -> Self {
        match value {
            syn::Expr::Array(_expr_array) => todo!(),
            syn::Expr::Assign(_expr_assign) => todo!(),
            syn::Expr::Async(_expr_async) => todo!(),
            syn::Expr::Await(_expr_await) => todo!(),
            syn::Expr::Binary(_expr_binary) => todo!(),
            syn::Expr::Block(block) => Block::parse(&block.block, path).into(),
            syn::Expr::Break(_expr_break) => todo!(),
            syn::Expr::Call(call) => Operator::call(call, path).into(),
            syn::Expr::Cast(_expr_cast) => todo!(),
            syn::Expr::Closure(_expr_closure) => todo!(),
            syn::Expr::Const(_expr_const) => todo!(),
            syn::Expr::Continue(_expr_continue) => todo!(),
            syn::Expr::Field(_expr_field) => todo!(),
            syn::Expr::ForLoop(_expr_for_loop) => todo!(),
            syn::Expr::Group(_expr_group) => todo!(),
            syn::Expr::If(_expr_if) => todo!(),
            syn::Expr::Index(_expr_index) => todo!(),
            syn::Expr::Infer(_expr_infer) => todo!(),
            syn::Expr::Let(_expr_let) => todo!(),
            syn::Expr::Lit(literal) => Literal::parse(&literal.lit).into(),
            syn::Expr::Loop(_expr_loop) => todo!(),
            syn::Expr::Macro(_expr_macro) => todo!(),
            syn::Expr::Match(_expr_match) => todo!(),
            syn::Expr::MethodCall(_expr_method_call) => todo!(),
            syn::Expr::Paren(_expr_paren) => todo!(),
            syn::Expr::Path(path) => Symbol::parse(&path.path).into(),
            syn::Expr::Range(_expr_range) => todo!(),
            syn::Expr::RawAddr(_expr_raw_addr) => todo!(),
            syn::Expr::Reference(_expr_reference) => todo!(),
            syn::Expr::Repeat(_expr_repeat) => todo!(),
            syn::Expr::Return(_expr_return) => todo!(),
            syn::Expr::Struct(_expr_struct) => todo!(),
            syn::Expr::Try(_expr_try) => todo!(),
            syn::Expr::TryBlock(_expr_try_block) => todo!(),
            syn::Expr::Tuple(_expr_tuple) => todo!(),
            syn::Expr::Unary(_expr_unary) => todo!(),
            syn::Expr::Unsafe(_expr_unsafe) => todo!(),
            syn::Expr::Verbatim(_token_stream) => todo!(),
            syn::Expr::While(_expr_while) => todo!(),
            syn::Expr::Yield(_expr_yield) => todo!(),
            _ => todo!(),
        }
    }

    pub fn resolve(&mut self, ctx: &Context) {
        match self {
            Self::Literal(_literal) => (),
            Self::Symbol(symbol) => symbol.resolve(ctx),
            Self::Operator(operator) => operator.resolve(ctx),
            Self::Block(block) => block.resolve(ctx),
        }
    }

    pub fn build(&self, builder: &mut dyn ob::FunctionBuilder) -> ob::SSAValue {
        match self {
            Self::Literal(literal) => literal.build(builder),
            Self::Block(block) => block.build(builder),
            Self::Operator(call) => call.build(builder),
            Self::Symbol(symbol) => symbol.build(builder),
        }
    }
}

impl From<Literal> for Expression {
    fn from(value: Literal) -> Self {
        Self::Literal(value)
    }
}

impl From<Block> for Expression {
    fn from(value: Block) -> Self {
        Self::Block(value)
    }
}

impl From<Operator> for Expression {
    fn from(value: Operator) -> Self {
        Self::Operator(value)
    }
}

impl From<Symbol> for Expression {
    fn from(value: Symbol) -> Self {
        Self::Symbol(value)
    }
}

#[derive(Clone, Debug)]
pub struct Body {
    pub expression: Expression,
}

impl Body {
    pub fn new(expression: Expression) -> Self {
        Self { expression }
    }

    pub fn resolve(&mut self, ctx: &Context) {
        self.expression.resolve(ctx);
    }
}
