use super::Expression;

#[derive(Debug, PartialEq, Clone)]
pub enum Texture<'src> {
    SolidColor(Expression<'src>),
    Checker(Expression<'src>, Expression<'src>), // Texture, Texture
    Perlin(Expression<'src>),                    // scale: f64
                                                 // TODO: ImageTexture
}
