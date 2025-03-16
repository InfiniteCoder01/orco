use crate::Context;

pub mod block;
pub use block::Block;

pub mod literal;
pub use literal::Literal;

pub mod operator;
pub use operator::Operator;

#[derive(Clone, Debug)]
pub enum Expression {
    Literal(Literal),
    Operator(Operator),
    Block(Block),
}

impl Expression {
    pub fn parse(ctx: &mut Context, value: &syn::Expr) -> Self {
        match value {
            syn::Expr::Array(_expr_array) => todo!(),
            syn::Expr::Assign(_expr_assign) => todo!(),
            syn::Expr::Async(_expr_async) => todo!(),
            syn::Expr::Await(_expr_await) => todo!(),
            syn::Expr::Binary(_expr_binary) => todo!(),
            syn::Expr::Block(block) => Block::parse(ctx, &block.block).into(),
            syn::Expr::Break(_expr_break) => todo!(),
            syn::Expr::Call(call) => Operator::call(ctx, call).into(),
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
            syn::Expr::Lit(literal) => Literal::parse(ctx, &literal.lit).into(),
            syn::Expr::Loop(_expr_loop) => todo!(),
            syn::Expr::Macro(_expr_macro) => todo!(),
            syn::Expr::Match(_expr_match) => todo!(),
            syn::Expr::MethodCall(_expr_method_call) => todo!(),
            syn::Expr::Paren(_expr_paren) => todo!(),
            syn::Expr::Path(_expr_path) => todo!(),
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

    pub fn build(
        &self,
        builder: &mut crate::backend::FunctionBuilder,
    ) -> Vec<crate::backend::cl::Value> {
        match self {
            Self::Literal(literal) => vec![literal.build(builder)],
            Self::Block(block) => block.build(builder),

            Self::Operator(call) => call.build(builder),
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

#[derive(Clone, Debug)]
pub struct Body {
    pub expression: Expression,
}

impl Body {
    pub fn new(expression: Expression) -> Self {
        Self { expression }
    }

    pub fn resolve(&self, ctx: &Context) {
        //
    }
}
