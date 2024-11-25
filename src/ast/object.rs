use super::Expression;

#[derive(Debug, PartialEq, Clone)]
pub enum RotateAxis {
    X,
    Y,
    Z,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Rotate<'src> {
    pub axis: RotateAxis,
    pub expr: Expression<'src>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AffineProperties<'src> {
    Translation(Expression<'src>), // vec3
    Rotate(Rotate<'src>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Object<'src> {
    Objects {
        objects: Vec<Object<'src>>,
        affine: Vec<AffineProperties<'src>>,
    },
    Sphere {
        center: Expression<'src>,
        radius: Expression<'src>,
        material: Expression<'src>, // Expression::Material
        affine: Vec<AffineProperties<'src>>,
    },
    Box {
        vertex: (Expression<'src>, Expression<'src>),
        material: Expression<'src>, // Expression::Material
        affine: Vec<AffineProperties<'src>>,
    },
    Plane {
        vertex: (Expression<'src>, Expression<'src>),
        material: Expression<'src>, // Expression::Material
        affine: Vec<AffineProperties<'src>>,
    },
}
