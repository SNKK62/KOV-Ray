use super::Expression;

#[derive(Debug, PartialEq, Clone)]
pub struct CameraConfig<'src> {
    pub(crate) lookfrom: Expression<'src>,
    pub(crate) lookat: Expression<'src>,
    pub(crate) up: Option<Expression<'src>>,
    pub(crate) angle: Expression<'src>,
    pub(crate) dist_to_focus: Option<Expression<'src>>,
}
