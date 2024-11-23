use super::Expression;

#[derive(Debug, PartialEq, Clone)]
pub enum Material<'src> {
    Lambertian {
        texture: Expression<'src>,
    },
    Metal {
        color: Expression<'src>,
        fuzz: Expression<'src>,
    },
    Dielectric {
        reflection_index: Expression<'src>,
    },
    Light {
        color: Expression<'src>,
        intensity: Expression<'src>,
    },
}
