use super::Expression;

#[derive(Debug, PartialEq, Clone)]
pub enum Object<'src> {
    Objects {
        objects: Vec<Object<'src>>,
        translate: Option<Expression<'src>>,
        rotate: Option<Expression<'src>>,
    },
    Sphere {
        center: Expression<'src>,
        radius: Expression<'src>,
        material: Expression<'src>, // Expression::Material
        translate: Option<Expression<'src>>,
        rotate: Option<Expression<'src>>,
    },
    Box {
        vertex: (Expression<'src>, Expression<'src>),
        material: Expression<'src>, // Expression::Material
        translate: Option<Expression<'src>>,
        rotate: Option<Expression<'src>>,
    },
    Plane {
        vertex: (Expression<'src>, Expression<'src>),
        material: Expression<'src>, // Expression::Material
        translate: Option<Expression<'src>>,
        rotate: Option<Expression<'src>>,
    },
}
