use super::Expression;

#[derive(Debug, PartialEq, Clone)]
pub enum Texture<'src> {
    SolidColor(Expression<'src>),
    Checker(Expression<'src>, Expression<'src>),
    Perlin(Expression<'src>), // scale: f64
                              // TODO: ImageTexture
}
