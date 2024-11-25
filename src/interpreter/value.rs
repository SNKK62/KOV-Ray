use ray_tracer_rs::{material::Material, texture::Texture, vec3::Vec3};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone)]
pub(super) enum Value {
    Num(f64),
    Str(String),
    Bool(bool),
    Vec3(f64, f64, f64),
    Material(Rc<RefCell<dyn Material>>),
    Texture(Rc<dyn Texture>),
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

pub(crate) struct ConfigValue {
    pub(crate) width: f64,
    pub(crate) height: f64,
    pub(crate) samples_per_pixel: f64,
    pub(crate) max_depth: f64,
    pub(crate) background: Vec3,
}

pub(crate) struct CameraConfigValue {
    pub(crate) lookfrom: Vec3,
    pub(crate) lookat: Vec3,
    pub(crate) up: Vec3,
    pub(crate) angle: f64,
    pub(crate) dist_to_focus: f64,
}
