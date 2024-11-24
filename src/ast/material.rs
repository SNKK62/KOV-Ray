use super::Expression;

#[derive(Debug, PartialEq, Clone)]
pub enum Material<'src> {
    Lambertian {
        texture: Expression<'src>,
    },
    Metal {
        color: Expression<'src>,
        fuzz: Expression<'src>, // f64
    },
    Dielectric {
        reflection_index: Expression<'src>, // f64
    },
    Light {
        color: Expression<'src>,
        intensity: Expression<'src>,
    },
}
