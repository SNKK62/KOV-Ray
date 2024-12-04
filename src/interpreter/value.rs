use ray_tracer_rs::{material::MaterialEnum, texture::TextureEnum, vec3::Vec3};

#[derive(Clone)]
pub(super) enum Value {
    Num(f64),
    Str(String),
    Bool(bool),
    Vec3(f64, f64, f64),
    Material(MaterialEnum),
    Texture(TextureEnum),
}

impl Value {
    pub fn to_bool(&self) -> bool {
        match self {
            Value::Num(n) => *n != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::Bool(b) => *b,
            _ => panic!("Cannot convert to bool"),
        }
    }
}

pub struct ConfigValue {
    pub width: f64,
    pub height: f64,
    pub samples_per_pixel: f64,
    pub max_depth: f64,
    pub background: Vec3,
}

pub(crate) struct CameraConfigValue {
    pub(crate) lookfrom: Vec3,
    pub(crate) lookat: Vec3,
    pub(crate) up: Vec3,
    pub(crate) angle: f64,
    pub(crate) dist_to_focus: f64,
}
