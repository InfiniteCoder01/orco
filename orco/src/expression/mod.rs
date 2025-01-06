/// See [Literal]
pub mod literal;
pub use literal::Literal;
/// Function
pub mod function;
pub use function::Function;
/// See [Call]
pub mod call;
pub use call::Call;

use soa_rs::Soars;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ExpressionKind {
    Literal,
    Function,
    Call,
}

type Index = u32;
pub struct LiteralIndex(Index);
pub struct FunctionIndex(Index);
pub struct ExpressionIndex(Index);

#[derive(Soars)]
struct InternalExpression {
    kind: ExpressionKind,
    value: Index,
}

#[repr(transparent)]
pub struct Expressions(soa_rs::Slice<InternalExpression>);

// -------------------------------------------  Immutable
pub enum ExpressionRef<'a> {
    Literal(LiteralIndex),
    Function(FunctionIndex),
    Call(&'a Call),
}

pub struct ExpressionsIter<'a> {
    expressions: &'a Expressions,
    index: Index,
}

impl<'a> Iterator for ExpressionsIter<'a> {
    type Item = ExpressionRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let expr = self.expressions.0.get(self.index as usize)?;
        Some(match expr.kind {
            ExpressionKind::Literal => ExpressionRef::Literal(LiteralIndex(*expr.value)),
            ExpressionKind::Function => ExpressionRef::Function(FunctionIndex(*expr.value)),
            ExpressionKind::Call => ExpressionRef::Call(unsafe {
                &*(&raw const *self
                    .expressions
                    .0
                    .idx(self.index as usize..(self.index + expr.value) as usize)
                    as *const _)
            }),
        })
    }
}

impl<'a> IntoIterator for &'a Expressions {
    type Item = ExpressionRef<'a>;

    type IntoIter = ExpressionsIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ExpressionsIter {
            expressions: self,
            index: 0,
        }
    }
}

// -------------------------------------------  Mutable
pub enum ExpressionMut<'a> {
    Literal(LiteralIndex),
    Function(FunctionIndex),
    Call(&'a mut Call),
}

pub struct ExpressionsIterMut<'a> {
    expressions: &'a mut Expressions,
    index: Index,
}

impl<'a> Iterator for ExpressionsIterMut<'a> {
    type Item = ExpressionMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let expr = self.expressions.0.get(self.index as usize)?;
        Some(match expr.kind {
            ExpressionKind::Literal => ExpressionMut::Literal(LiteralIndex(*expr.value)),
            ExpressionKind::Function => ExpressionMut::Function(FunctionIndex(*expr.value)),
            ExpressionKind::Call => ExpressionMut::Call(unsafe {
                &mut *(&raw mut *self
                    .expressions
                    .0
                    .idx_mut(self.index as usize..(self.index + expr.value) as usize)
                    as *mut _)
            }),
        })
    }
}

impl<'a> IntoIterator for &'a mut Expressions {
    type Item = ExpressionMut<'a>;

    type IntoIter = ExpressionsIterMut<'a>;

    fn into_iter(self) -> Self::IntoIter {
        ExpressionsIterMut {
            expressions: self,
            index: 0,
        }
    }
}

impl Expressions {
    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> <&mut Self as IntoIterator>::IntoIter {
        self.into_iter()
    }
}
