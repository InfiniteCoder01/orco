pub mod block;
pub use block::Block;

pub mod literal;
pub use literal::Literal;

#[derive(Clone, Debug)]
pub enum Expression {
    Literal(Literal),
    Block(Block),
}

impl Expression {
    pub fn parse(value: &syn::Expr, path: &super::Path) -> Self {
        match value {
            syn::Expr::Array(expr_array) => todo!(),
            syn::Expr::Assign(expr_assign) => todo!(),
            syn::Expr::Async(expr_async) => todo!(),
            syn::Expr::Await(expr_await) => todo!(),
            syn::Expr::Binary(expr_binary) => todo!(),
            syn::Expr::Block(block) => Block::parse(&block.block, path).into(),
            syn::Expr::Break(expr_break) => todo!(),
            syn::Expr::Call(expr_call) => todo!(),
            syn::Expr::Cast(expr_cast) => todo!(),
            syn::Expr::Closure(expr_closure) => todo!(),
            syn::Expr::Const(expr_const) => todo!(),
            syn::Expr::Continue(expr_continue) => todo!(),
            syn::Expr::Field(expr_field) => todo!(),
            syn::Expr::ForLoop(expr_for_loop) => todo!(),
            syn::Expr::Group(expr_group) => todo!(),
            syn::Expr::If(expr_if) => todo!(),
            syn::Expr::Index(expr_index) => todo!(),
            syn::Expr::Infer(expr_infer) => todo!(),
            syn::Expr::Let(expr_let) => todo!(),
            syn::Expr::Lit(literal) => Literal::from(literal.lit.clone()).into(),
            syn::Expr::Loop(expr_loop) => todo!(),
            syn::Expr::Macro(expr_macro) => todo!(),
            syn::Expr::Match(expr_match) => todo!(),
            syn::Expr::MethodCall(expr_method_call) => todo!(),
            syn::Expr::Paren(expr_paren) => todo!(),
            syn::Expr::Path(expr_path) => todo!(),
            syn::Expr::Range(expr_range) => todo!(),
            syn::Expr::RawAddr(expr_raw_addr) => todo!(),
            syn::Expr::Reference(expr_reference) => todo!(),
            syn::Expr::Repeat(expr_repeat) => todo!(),
            syn::Expr::Return(expr_return) => todo!(),
            syn::Expr::Struct(expr_struct) => todo!(),
            syn::Expr::Try(expr_try) => todo!(),
            syn::Expr::TryBlock(expr_try_block) => todo!(),
            syn::Expr::Tuple(expr_tuple) => todo!(),
            syn::Expr::Unary(expr_unary) => todo!(),
            syn::Expr::Unsafe(expr_unsafe) => todo!(),
            syn::Expr::Verbatim(token_stream) => todo!(),
            syn::Expr::While(expr_while) => todo!(),
            syn::Expr::Yield(expr_yield) => todo!(),
            _ => todo!(),
        }
    }

    pub fn build(
        &self,
        builder: &mut crate::backend::FunctionBuilder,
    ) -> Vec<crate::backend::cl::Value> {
        match self {
            Self::Literal(literal) => vec![literal.build(builder)],
            Self::Block(block) => {
                for statement in &block.statements {
                    statement.build(builder);
                }
                block
                    .tail
                    .as_ref()
                    .map_or_else(Vec::new, |expr| expr.build(builder))
            }
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

// impl syn::spanned::Spanned for Expression {
//     fn span(&self) -> proc_macro2::Span {
//         match self {
//             Expression::Literal(literal) => todo!(),
//             Expression::Block(block) => todo!(),
//         }
//     }
// }

#[derive(Clone, Debug)]
pub struct Body {
    pub expression: Expression,
}

impl Body {
    pub fn new(expression: Expression) -> Self {
        Self { expression }
    }
}
