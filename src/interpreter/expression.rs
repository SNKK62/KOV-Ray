use super::{
    funcs::{FnDecl, Functions},
    value::Value,
    Variables, COLOR_MAX,
};
use crate::ast::{
    material::Material as MaterialAST, texture::Texture as TextureAST, ExprEnum, Expression,
};
use ray_tracer_rs::{
    material::{Dielectric, DiffuseLight, Lambertian, MaterialEnum, Metal},
    texture::{Checker, NoiseTexture, SolidColor, TextureEnum},
    vec3::Color,
};

pub(super) fn eval_expr(ast: &Expression, variables: &mut Variables, funcs: &Functions) -> Value {
    match &ast.expr {
        ExprEnum::Ident(ident) => {
            let ident = ident.fragment();
            if *ident == "PI" {
                return Value::Num(std::f64::consts::PI);
            }
            let val = variables
                .get(*ident)
                .unwrap_or_else(|| panic!("variable {} not found", *ident));
            val.clone()
        }
        ExprEnum::NumLiteral(n) => Value::Num(*n),
        ExprEnum::StrLiteral(s) => Value::Str(s.clone()),
        ExprEnum::FnInvoke(name, args) => {
            let name = name.fragment();
            let func = funcs
                .get(*name)
                .unwrap_or_else(|| panic!("function {} not found", name));
            let args = args
                .iter()
                .map(|arg| {
                    let val = eval_expr(arg, variables, funcs);
                    match val {
                        Value::Num(n) => n,
                        _ => panic!("Invalid argument type"),
                    }
                })
                .collect::<Vec<_>>();
            // TODO: handle non-native functions
            match func {
                FnDecl::Native(native) => Value::Num(native.code.as_ref()(&args)),
            }
        }
        ExprEnum::Add(a, b) => {
            let lhs = eval_expr(a, variables, funcs);
            let rhs = eval_expr(b, variables, funcs);
            match (lhs, rhs) {
                (Value::Num(lhs), Value::Num(rhs)) => Value::Num(lhs + rhs),
                _ => panic!("Invalid operands for addition"),
            }
        }
        ExprEnum::Sub(a, b) => {
            let lhs = eval_expr(a, variables, funcs);
            let rhs = eval_expr(b, variables, funcs);
            match (lhs, rhs) {
                (Value::Num(lhs), Value::Num(rhs)) => Value::Num(lhs - rhs),
                _ => panic!("Invalid operands for subtraction"),
            }
        }
        ExprEnum::Mul(a, b) => {
            let lhs = eval_expr(a, variables, funcs);
            let rhs = eval_expr(b, variables, funcs);
            match (lhs, rhs) {
                (Value::Num(lhs), Value::Num(rhs)) => Value::Num(lhs * rhs),
                _ => panic!("Invalid operands for multiplication"),
            }
        }
        ExprEnum::Div(a, b) => {
            let lhs = eval_expr(a, variables, funcs);
            let rhs = eval_expr(b, variables, funcs);
            match (lhs, rhs) {
                (Value::Num(lhs), Value::Num(rhs)) => Value::Num(lhs / rhs),
                _ => panic!("Invalid operands for division"),
            }
        }
        ExprEnum::And(a, b) => Value::Bool(
            eval_expr(a, variables, funcs).to_bool() && eval_expr(b, variables, funcs).to_bool(),
        ),
        ExprEnum::Or(a, b) => Value::Bool(
            eval_expr(a, variables, funcs).to_bool() || eval_expr(b, variables, funcs).to_bool(),
        ),
        ExprEnum::Gt(a, b) => {
            let lhs = eval_expr(a, variables, funcs);
            let rhs = eval_expr(b, variables, funcs);
            match (lhs, rhs) {
                (Value::Num(lhs), Value::Num(rhs)) => Value::Bool(lhs > rhs),
                _ => panic!("Invalid operands for greater than"),
            }
        }
        ExprEnum::Lt(a, b) => {
            let lhs = eval_expr(a, variables, funcs);
            let rhs = eval_expr(b, variables, funcs);
            match (lhs, rhs) {
                (Value::Num(lhs), Value::Num(rhs)) => Value::Bool(lhs < rhs),
                _ => panic!("Invalid operands for less than"),
            }
        }
        ExprEnum::Eq(a, b) => {
            let lhs = eval_expr(a, variables, funcs);
            let rhs = eval_expr(b, variables, funcs);
            match (lhs, rhs) {
                (Value::Num(lhs), Value::Num(rhs)) => Value::Bool(lhs == rhs),
                _ => panic!("Invalid operands for equality"),
            }
        }
        ExprEnum::Neq(a, b) => {
            let lhs = eval_expr(a, variables, funcs);
            let rhs = eval_expr(b, variables, funcs);
            match (lhs, rhs) {
                (Value::Num(lhs), Value::Num(rhs)) => Value::Bool(lhs != rhs),
                _ => panic!("Invalid operands for inequality"),
            }
        }
        ExprEnum::Not(a) => {
            let val = eval_expr(a, variables, funcs).to_bool();
            Value::Bool(!val)
        }
        ExprEnum::Vec3(x, y, z) => {
            let x = eval_expr(x, variables, funcs);
            let y = eval_expr(y, variables, funcs);
            let z = eval_expr(z, variables, funcs);
            match (x, y, z) {
                (Value::Num(x), Value::Num(y), Value::Num(z)) => Value::Vec3(x, y, z),
                _ => panic!("Invalid operands for Vec3"),
            }
        }
        ExprEnum::Material(mat) => match mat.as_ref() {
            MaterialAST::Lambertian { texture } => {
                let texture = eval_expr(texture, variables, funcs);
                match texture {
                    Value::Texture(texture) => {
                        Value::Material(MaterialEnum::Lambertian(Lambertian::new(&texture)))
                    }
                    _ => panic!("Invalid texture type"),
                }
            }
            MaterialAST::Metal { color, fuzz } => {
                let color = eval_expr(color, variables, funcs);
                let fuzz = eval_expr(fuzz, variables, funcs);
                match (color, fuzz) {
                    (Value::Vec3(r, g, b), Value::Num(fuzz)) => Value::Material(
                        MaterialEnum::Metal(Metal::new(&(Color::new(r, g, b) / COLOR_MAX), fuzz)),
                    ),
                    _ => panic!("Invalid color or fuzz type"),
                }
            }
            MaterialAST::Dielectric { reflection_index } => {
                let reflection_index = eval_expr(reflection_index, variables, funcs);
                match reflection_index {
                    Value::Num(reflection_index) => {
                        Value::Material(MaterialEnum::Dielectric(Dielectric::new(reflection_index)))
                    }
                    _ => panic!("Invalid reflection index type"),
                }
            }
            MaterialAST::Light { color, intensity } => {
                let color = eval_expr(color, variables, funcs);
                let intensity = eval_expr(intensity, variables, funcs);
                match (color, intensity) {
                    (Value::Vec3(r, g, b), Value::Num(intensity)) => Value::Material(
                        MaterialEnum::DiffuseLight(DiffuseLight::new(&TextureEnum::SolidColor(
                            SolidColor::new(Color::new(r, g, b) / COLOR_MAX * intensity),
                        ))),
                    ),
                    _ => panic!("Invalid color or intensity type"),
                }
            }
        },
        ExprEnum::Texture(tex) => match tex.as_ref() {
            TextureAST::SolidColor(color) => {
                let color = eval_expr(color, variables, funcs);
                match color {
                    Value::Vec3(r, g, b) => Value::Texture(TextureEnum::SolidColor(
                        SolidColor::new(Color::new(r, g, b) / COLOR_MAX),
                    )),
                    _ => panic!("Invalid color type"),
                }
            }
            TextureAST::Checker(add, even) => {
                let odd = eval_expr(add, variables, funcs);
                let even = eval_expr(even, variables, funcs);
                match (odd, even) {
                    (Value::Texture(odd), Value::Texture(even)) => {
                        Value::Texture(TextureEnum::Checker(Checker::new(odd, even)))
                    }
                    _ => panic!("Invalid checker type"),
                }
            }
            TextureAST::Perlin(scale) => {
                let scale = eval_expr(scale, variables, funcs);
                match scale {
                    Value::Num(scale) => {
                        Value::Texture(TextureEnum::NoiseTexture(NoiseTexture::new(scale)))
                    }
                    _ => panic!("Invalid scale type"),
                }
            }
        },
    }
}
