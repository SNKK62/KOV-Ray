use super::{Expression, Object, Span, AST};

#[derive(Debug, PartialEq, Clone)]
pub enum Statement<'src> {
    Expression(Expression<'src>),
    VarAssign {
        span: Span<'src>,
        name: Span<'src>,
        ex: Expression<'src>,
    },
    If {
        span: Span<'src>,
        cond: Box<Expression<'src>>,
        stmts: Box<AST<'src>>,
        else_stmts: Option<Box<AST<'src>>>,
    },
    While {
        span: Span<'src>,
        cond: Expression<'src>,
        stmts: AST<'src>,
    },
    Break,
    Continue,
    Object {
        span: Span<'src>,
        object: Object<'src>,
    },
    Camera {
        span: Span<'src>,
        lookfrom: Expression<'src>,
        lookat: Expression<'src>,
        angle: Expression<'src>,
    },
    Config {
        span: Span<'src>,
        width: Expression<'src>,
        height: Expression<'src>,
        samples_per_pixel: Expression<'src>,
        max_depth: Expression<'src>,
        sky_color: Option<Expression<'src>>,
    },
}

impl<'src> Statement<'src> {
    pub(super) fn span(&self) -> Option<Span<'src>> {
        use Statement::*;
        Some(match self {
            Expression(ex) => ex.span,
            VarAssign { span, .. } => *span,
            If { span, .. } => *span,
            While { span, .. } => *span,
            Object { span, .. } => *span,
            Camera { span, .. } => *span,
            Config { span, .. } => *span,
            Break | Continue => return None,
        })
    }
}
