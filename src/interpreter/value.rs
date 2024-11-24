use ray_tracer_rs::{material::Material, texture::Texture};
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
