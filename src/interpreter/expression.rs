use super::{
    funcs::{FnDecl, Functions},
    value::Value,
    EvalError, Variables, COLOR_MAX,
};
use crate::ast::{
    material::Material as MaterialAST, texture::Texture as TextureAST, ExprEnum, Expression,
};
use ray_tracer_rs::{
    material::{Dielectric, DiffuseLight, Lambertian, MaterialEnum, Metal},
    texture::{Checker, NoiseTexture, SolidColor, TextureEnum},
    vec3::Color,
};

pub(super) fn eval_expr<'a>(
    ast: &'a Expression,
    variables: &mut Variables,
    funcs: &Functions,
) -> Result<Value, EvalError<'a>> {
    let val = match &ast.expr {
        ExprEnum::Ident(ident) => {
            let _ident = ident.fragment();
            if *_ident == "PI" {
                return Ok(Value::Num(std::f64::consts::PI));
            }
            let val = variables.get(*_ident);

            if val.is_none() {
                return Err(EvalError {
                    span: Some(*ident),
                    message: format!("variable \"{}\" not found", *ident),
                });
            }
            val.unwrap().clone()
        }
        ExprEnum::NumLiteral(n) => Value::Num(*n),
        ExprEnum::StrLiteral(s) => Value::Str(s.clone()),
        ExprEnum::FnInvoke(name, args) => {
            let _name = name.fragment();
            let func = funcs.get(*_name);
            if func.is_none() {
                return Err(EvalError {
                    span: Some(*name),
                    message: format!("function \"{}\" not found", name),
                });
            }
            let args = args
                .iter()
                .map(|arg| eval_expr(arg, variables, funcs))
                .collect::<Vec<_>>();
            for arg in &args {
                if let Err(e) = arg {
                    return Err(e.clone());
                }
            }
            let args = args.into_iter().map(|arg| arg.unwrap()).collect::<Vec<_>>();
            // TODO: handle non-native functions
            match func.unwrap() {
                FnDecl::Native(native) => {
                    let ans = native.code.as_ref()(&args);
                    if let Err(e) = ans {
                        return Err(EvalError {
                            span: Some(*name),
                            message: e,
                        });
                    }
                    ans.unwrap()
                }
            }
        }
        ExprEnum::Add(a, b) => {
            let lhs = eval_expr(a, variables, funcs)?;
            let rhs = eval_expr(b, variables, funcs)?;
            match (lhs, rhs) {
                (Value::Num(lhs), Value::Num(rhs)) => Value::Num(lhs + rhs),
                (Value::Vec3(x1, y1, z1), Value::Vec3(x2, y2, z2)) => {
                    Value::Vec3(x1 + x2, y1 + y2, z1 + z2)
                }
                _ => {
                    return Err(EvalError {
                        span: Some(a.span),
                        message: "Invalid operands for +".to_string(),
                    })
                }
            }
        }
        ExprEnum::Sub(a, b) => {
            let lhs = eval_expr(a, variables, funcs)?;
            let rhs = eval_expr(b, variables, funcs)?;
            match (lhs, rhs) {
                (Value::Num(lhs), Value::Num(rhs)) => Value::Num(lhs - rhs),
                (Value::Vec3(x1, y1, z1), Value::Vec3(x2, y2, z2)) => {
                    Value::Vec3(x1 - x2, y1 - y2, z1 - z2)
                }
                _ => {
                    return Err(EvalError {
                        span: Some(a.span),
                        message: "Invalid operands for -".to_string(),
                    })
                }
            }
        }
        ExprEnum::Mul(a, b) => {
            let lhs = eval_expr(a, variables, funcs)?;
            let rhs = eval_expr(b, variables, funcs)?;
            match (lhs, rhs) {
                (Value::Num(lhs), Value::Num(rhs)) => Value::Num(lhs * rhs),
                (Value::Num(lhs), Value::Vec3(x, y, z)) => Value::Vec3(lhs * x, lhs * y, lhs * z),
                (Value::Vec3(x, y, z), Value::Num(rhs)) => Value::Vec3(rhs * x, rhs * y, rhs * z),
                (Value::Vec3(x1, y1, z1), Value::Vec3(x2, y2, z2)) => {
                    Value::Vec3(x1 * x2, y1 * y2, z1 * z2)
                }
                _ => {
                    return Err(EvalError {
                        span: Some(a.span),
                        message: "Invalid operands for *".to_string(),
                    })
                }
            }
        }
        ExprEnum::Div(a, b) => {
            let lhs = eval_expr(a, variables, funcs)?;
            let rhs = eval_expr(b, variables, funcs)?;
            match (lhs, rhs) {
                (Value::Num(lhs), Value::Num(rhs)) => Value::Num(lhs / rhs),
                (Value::Vec3(x, y, z), Value::Num(rhs)) => Value::Vec3(x / rhs, y / rhs, z / rhs),
                _ => {
                    return Err(EvalError {
                        span: Some(a.span),
                        message: "Invalid operands for /".to_string(),
                    })
                }
            }
        }
        ExprEnum::And(a, b) => {
            let lhs = eval_expr(a, variables, funcs)?;
            let rhs = eval_expr(b, variables, funcs)?;
            let lhs_bool = lhs.to_bool();
            if let Err(e) = lhs_bool {
                return Err(EvalError {
                    span: Some(a.span),
                    message: e,
                });
            }
            let rhs_bool = rhs.to_bool();
            if let Err(e) = rhs_bool {
                return Err(EvalError {
                    span: Some(b.span),
                    message: e,
                });
            }
            Value::Bool(lhs_bool.unwrap() && rhs_bool.unwrap())
        }
        ExprEnum::Or(a, b) => {
            let lhs = eval_expr(a, variables, funcs)?;
            let rhs = eval_expr(b, variables, funcs)?;
            let lhs_bool = lhs.to_bool();
            if let Err(e) = lhs_bool {
                return Err(EvalError {
                    span: Some(a.span),
                    message: e,
                });
            }
            let rhs_bool = rhs.to_bool();
            if let Err(e) = rhs_bool {
                return Err(EvalError {
                    span: Some(b.span),
                    message: e,
                });
            }
            Value::Bool(lhs_bool.unwrap() || rhs_bool.unwrap())
        }
        ExprEnum::Gt(a, b) => {
            let lhs = eval_expr(a, variables, funcs)?;
            let rhs = eval_expr(b, variables, funcs)?;
            match (lhs, rhs) {
                (Value::Num(lhs), Value::Num(rhs)) => Value::Bool(lhs > rhs),
                _ => {
                    return Err(EvalError {
                        span: Some(a.span),
                        message: "Invalid operands for >".to_string(),
                    })
                }
            }
        }
        ExprEnum::Ge(a, b) => {
            let lhs = eval_expr(a, variables, funcs)?;
            let rhs = eval_expr(b, variables, funcs)?;
            match (lhs, rhs) {
                (Value::Num(lhs), Value::Num(rhs)) => Value::Bool(lhs >= rhs),
                _ => {
                    return Err(EvalError {
                        span: Some(a.span),
                        message: "Invalid operands for >=".to_string(),
                    })
                }
            }
        }
        ExprEnum::Lt(a, b) => {
            let lhs = eval_expr(a, variables, funcs)?;
            let rhs = eval_expr(b, variables, funcs)?;
            match (lhs, rhs) {
                (Value::Num(lhs), Value::Num(rhs)) => Value::Bool(lhs < rhs),
                _ => {
                    return Err(EvalError {
                        span: Some(a.span),
                        message: "Invalid operands for <".to_string(),
                    })
                }
            }
        }
        ExprEnum::Le(a, b) => {
            let lhs = eval_expr(a, variables, funcs)?;
            let rhs = eval_expr(b, variables, funcs)?;
            match (lhs, rhs) {
                (Value::Num(lhs), Value::Num(rhs)) => Value::Bool(lhs <= rhs),
                _ => {
                    return Err(EvalError {
                        span: Some(a.span),
                        message: "Invalid operands for <=".to_string(),
                    })
                }
            }
        }
        ExprEnum::Eq(a, b) => {
            let lhs = eval_expr(a, variables, funcs)?;
            let rhs = eval_expr(b, variables, funcs)?;
            match (lhs, rhs) {
                (Value::Num(lhs), Value::Num(rhs)) => Value::Bool(lhs == rhs),
                _ => {
                    return Err(EvalError {
                        span: Some(a.span),
                        message: "Invalid operands for ==".to_string(),
                    })
                }
            }
        }
        ExprEnum::Neq(a, b) => {
            let lhs = eval_expr(a, variables, funcs)?;
            let rhs = eval_expr(b, variables, funcs)?;
            match (lhs, rhs) {
                (Value::Num(lhs), Value::Num(rhs)) => Value::Bool(lhs != rhs),
                _ => {
                    return Err(EvalError {
                        span: Some(a.span),
                        message: "Invalid operands for !=".to_string(),
                    })
                }
            }
        }
        ExprEnum::Not(a) => {
            let val = eval_expr(a, variables, funcs)?.to_bool();
            if let Err(e) = val {
                return Err(EvalError {
                    span: Some(a.span),
                    message: e,
                });
            }
            Value::Bool(!val.unwrap())
        }
        ExprEnum::Vec3(x, y, z) => {
            let x_val = eval_expr(x, variables, funcs)?;
            let y_val = eval_expr(y, variables, funcs)?;
            let z_val = eval_expr(z, variables, funcs)?;
            match (x_val, y_val, z_val) {
                (Value::Num(x), Value::Num(y), Value::Num(z)) => Value::Vec3(x, y, z),
                _ => {
                    return Err(EvalError {
                        span: Some(x.span),
                        message: "Invalid member for Vec3".to_string(),
                    })
                }
            }
        }
        ExprEnum::Material(mat) => match mat.as_ref() {
            MaterialAST::Lambertian { texture } => {
                let texture_val = eval_expr(texture, variables, funcs)?;
                match texture_val {
                    Value::Texture(texture) => {
                        Value::Material(MaterialEnum::Lambertian(Lambertian::new(&texture)))
                    }
                    _ => {
                        return Err(EvalError {
                            span: Some(texture.span),
                            message: "Invalid texture type".to_string(),
                        })
                    }
                }
            }
            MaterialAST::Metal { color, fuzz } => {
                let color_val = eval_expr(color, variables, funcs)?;
                let fuzz = eval_expr(fuzz, variables, funcs)?;
                match (color_val, fuzz) {
                    (Value::Vec3(r, g, b), Value::Num(fuzz)) => Value::Material(
                        MaterialEnum::Metal(Metal::new(&(Color::new(r, g, b) / COLOR_MAX), fuzz)),
                    ),
                    _ => {
                        return Err(EvalError {
                            span: Some(color.span),
                            message: "Invalid texture type".to_string(),
                        })
                    }
                }
            }
            MaterialAST::Dielectric { reflection_index } => {
                let reflection_index_val = eval_expr(reflection_index, variables, funcs)?;
                match reflection_index_val {
                    Value::Num(reflection_index) => {
                        Value::Material(MaterialEnum::Dielectric(Dielectric::new(reflection_index)))
                    }
                    _ => {
                        return Err(EvalError {
                            span: Some(reflection_index.span),
                            message: "Invalid reflection index type".to_string(),
                        })
                    }
                }
            }
            MaterialAST::Light { color, intensity } => {
                let color_val = eval_expr(color, variables, funcs)?;
                let intensity = eval_expr(intensity, variables, funcs)?;
                match (color_val, intensity) {
                    (Value::Vec3(r, g, b), Value::Num(intensity)) => Value::Material(
                        MaterialEnum::DiffuseLight(DiffuseLight::new(&TextureEnum::SolidColor(
                            SolidColor::new(Color::new(r, g, b) / COLOR_MAX * intensity),
                        ))),
                    ),
                    _ => {
                        return Err(EvalError {
                            span: Some(color.span),
                            message: "Invalid color or intensity type".to_string(),
                        })
                    }
                }
            }
        },
        ExprEnum::Texture(tex) => match tex.as_ref() {
            TextureAST::SolidColor(color) => {
                let color_val = eval_expr(color, variables, funcs)?;
                match color_val {
                    Value::Vec3(r, g, b) => Value::Texture(TextureEnum::SolidColor(
                        SolidColor::new(Color::new(r, g, b) / COLOR_MAX),
                    )),
                    _ => {
                        return Err(EvalError {
                            span: Some(color.span),
                            message: "Invalid color type".to_string(),
                        })
                    }
                }
            }
            TextureAST::Checker(odd, even) => {
                let odd_val = eval_expr(odd, variables, funcs)?;
                let even = eval_expr(even, variables, funcs)?;
                match (odd_val, even) {
                    (Value::Texture(odd), Value::Texture(even)) => {
                        Value::Texture(TextureEnum::Checker(Checker::new(odd, even)))
                    }
                    _ => {
                        return Err(EvalError {
                            span: Some(odd.span),
                            message: "Invalid checker type".to_string(),
                        })
                    }
                }
            }
            TextureAST::Perlin(scale) => {
                let scale_val = eval_expr(scale, variables, funcs)?;
                match scale_val {
                    Value::Num(scale) => {
                        Value::Texture(TextureEnum::NoiseTexture(NoiseTexture::new(scale)))
                    }
                    _ => {
                        return Err(EvalError {
                            span: Some(scale.span),
                            message: "Invalid scale type".to_string(),
                        })
                    }
                }
            }
        },
    };
    Ok(val)
}
