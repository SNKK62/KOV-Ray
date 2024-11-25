pub mod expression;
pub use expression::{ExprEnum, Expression};
pub mod statement;
pub use statement::Statement;
pub mod object;
pub use object::Object;
pub mod material;
pub use material::Material;
pub mod texture;
pub use texture::Texture;
pub mod config;
pub use config::Config;
pub mod camera;
pub use camera::CameraConfig;

use nom_locate::LocatedSpan;
pub type Span<'a> = LocatedSpan<&'a str>;

pub type AST<'src> = Vec<Statement<'src>>;

trait GetSpan<'a> {
    fn span(&self) -> Span<'a>;
}

impl<'a> GetSpan<'a> for AST<'a> {
    fn span(&self) -> Span<'a> {
        self.iter().find_map(|stmt| stmt.span()).unwrap()
    }
}
