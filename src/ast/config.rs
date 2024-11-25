use super::Expression;

#[derive(Debug, PartialEq, Clone)]
pub struct Config<'src> {
    pub(crate) width: Expression<'src>,
    pub(crate) height: Expression<'src>,
    pub(crate) samples_per_pixel: Expression<'src>,
    pub(crate) max_depth: Option<Expression<'src>>,
    pub(crate) sky_color: Option<Expression<'src>>,
}
