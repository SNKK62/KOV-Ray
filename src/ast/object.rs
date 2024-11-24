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
pub struct AffineProperties<'src> {
    pub translate: Option<Expression<'src>>, // vec3
    pub rotate: Vec<Rotate<'src>>,
}

impl Default for AffineProperties<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'src> AffineProperties<'src> {
    pub fn new() -> Self {
        Self {
            translate: None,
            rotate: Vec::new(),
        }
    }

    /// Set the translation vector.
    /// axis should be 0(X), 1(Y), or 2(Z).
    pub fn push_rotate(&mut self, axis: usize, expr: Expression<'src>) {
        if axis > 2 {
            panic!("Invalid axis: {}", axis);
        }
        let axis = match axis {
            0 => RotateAxis::X,
            1 => RotateAxis::Y,
            2 => RotateAxis::Z,
            _ => unreachable!(),
        };
        self.rotate.push(Rotate { axis, expr });
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Object<'src> {
    Objects {
        objects: Vec<Object<'src>>,
        affine: AffineProperties<'src>,
    },
    Sphere {
        center: Expression<'src>,
        radius: Expression<'src>,
        material: Expression<'src>, // Expression::Material
        affine: AffineProperties<'src>,
    },
    Box {
        vertex: (Expression<'src>, Expression<'src>),
        material: Expression<'src>, // Expression::Material
        affine: AffineProperties<'src>,
    },
    Plane {
        vertex: (Expression<'src>, Expression<'src>),
        material: Expression<'src>, // Expression::Material
        affine: AffineProperties<'src>,
    },
}
