use super::{
    expression::eval_expr,
    funcs::Functions,
    object::eval_object,
    value::{CameraConfigValue, ConfigValue, Value},
    EvalError, Variables, COLOR_MAX,
};
use crate::ast::Statement;
use ray_tracer_rs::{
    hittable::HittableEnum,
    vec3::{Color, Vec3},
};

pub(super) fn eval_stmt<'a>(
    ast: &'a Statement<'a>,
    variables: &mut Variables,
    funcs: &Functions<'a>,
    world: &mut Vec<HittableEnum>,
    config: &mut Option<ConfigValue>,
    camera_config: &mut Option<CameraConfigValue>,
) -> Result<(), EvalError<'a>> {
    match ast {
        Statement::Expression(expr) => {
            let _ = eval_expr(expr, variables, funcs)?;
        }
        Statement::VarAssign { name, ex, .. } => {
            let value = eval_expr(ex, variables, funcs)?;
            variables.insert(name.to_string(), value);
        }
        Statement::If {
            cond,
            stmts,
            else_stmts,
            ..
        } => {
            let cond_val = eval_expr(cond, variables, funcs)?;
            let cond_bool = cond_val.to_bool();
            if let Err(e) = cond_bool {
                return Err(EvalError {
                    span: Some(cond.span),
                    message: e,
                });
            }
            if cond_bool.unwrap() {
                for stmt in stmts.iter() {
                    eval_stmt(stmt, variables, funcs, world, config, camera_config)?;
                }
            } else if let Some(ref else_stmts) = else_stmts {
                for stmt in else_stmts.iter() {
                    eval_stmt(stmt, variables, funcs, world, config, camera_config)?;
                }
            }
        }
        Statement::While { cond, stmts, .. } => loop {
            let cond_val = eval_expr(cond, variables, funcs)?;
            let cond_bool = cond_val.to_bool();
            if let Err(e) = cond_bool {
                return Err(EvalError {
                    span: Some(cond.span),
                    message: e,
                });
            }
            if !cond_bool.unwrap() {
                break;
            }
            for stmt in stmts.iter() {
                match stmt {
                    Statement::Break => break,
                    Statement::Continue => continue,
                    _ => eval_stmt(stmt, variables, funcs, world, config, camera_config)?,
                }
            }
        },
        Statement::Break | Statement::Continue => {}
        Statement::Object { object, .. } => {
            eval_object(object, variables, funcs, world)?;
        }
        Statement::Config { config: c, .. } => {
            let width = match eval_expr(&c.width, variables, funcs)? {
                Value::Num(n) => n,
                _ => {
                    return Err(EvalError {
                        span: Some(c.width.span),
                        message: "Invalid width".to_string(),
                    })
                }
            };
            let height = match eval_expr(&c.height, variables, funcs)? {
                Value::Num(n) => n,
                _ => {
                    return Err(EvalError {
                        span: Some(c.height.span),
                        message: "Invalid height".to_string(),
                    })
                }
            };
            let samples_per_pixel = match eval_expr(&c.samples_per_pixel, variables, funcs)? {
                Value::Num(n) => n,
                _ => {
                    return Err(EvalError {
                        span: Some(c.samples_per_pixel.span),
                        message: "Invalid samples_per_pixel".to_string(),
                    })
                }
            };
            let max_depth = match c.max_depth.as_ref() {
                Some(expr) => match eval_expr(expr, variables, funcs)? {
                    Value::Num(n) => n,
                    _ => {
                        return Err(EvalError {
                            span: Some(c.max_depth.as_ref().unwrap().span),
                            message: "Invalid max_depth".to_string(),
                        })
                    }
                },
                None => 100.0,
            };
            let background = match c.background.as_ref() {
                Some(expr) => match eval_expr(expr, variables, funcs)? {
                    Value::Vec3(x, y, z) => Color::new(x, y, z) / COLOR_MAX,
                    _ => {
                        return Err(EvalError {
                            span: Some(c.background.as_ref().unwrap().span),
                            message: "Invalid sky_color".to_string(),
                        })
                    }
                },
                None => Color::zero(),
            };
            *config = Some(ConfigValue {
                width,
                height,
                samples_per_pixel,
                max_depth,
                background,
            });
        }
        Statement::Camera { config: c, .. } => {
            let lookfrom = match eval_expr(&c.lookfrom, variables, funcs)? {
                Value::Vec3(x, y, z) => Vec3::new(x, y, z),
                _ => {
                    return Err(EvalError {
                        span: Some(c.lookfrom.span),
                        message: "Invalid look_from".to_string(),
                    })
                }
            };
            let lookat = match eval_expr(&c.lookat, variables, funcs)? {
                Value::Vec3(x, y, z) => Vec3::new(x, y, z),
                _ => {
                    return Err(EvalError {
                        span: Some(c.lookat.span),
                        message: "Invalid look_at".to_string(),
                    })
                }
            };
            let up = match c.up.as_ref() {
                Some(expr) => match eval_expr(expr, variables, funcs)? {
                    Value::Vec3(x, y, z) => Vec3::new(x, y, z),
                    _ => {
                        return Err(EvalError {
                            span: Some(c.up.as_ref().unwrap().span),
                            message: "Invalid up".to_string(),
                        })
                    }
                },
                None => Vec3::new(0.0, 1.0, 0.0),
            };
            let angle = match eval_expr(&c.angle, variables, funcs)? {
                Value::Num(n) => n,
                _ => {
                    return Err(EvalError {
                        span: Some(c.angle.span),
                        message: "Invalid angle".to_string(),
                    })
                }
            };
            let dist_to_focus = match c.dist_to_focus.as_ref() {
                Some(expr) => match eval_expr(expr, variables, funcs)? {
                    Value::Num(n) => n,
                    _ => {
                        return Err(EvalError {
                            span: Some(c.dist_to_focus.as_ref().unwrap().span),
                            message: "Invalid dist_to_focus".to_string(),
                        })
                    }
                },
                None => 10.0,
            };
            *camera_config = Some(CameraConfigValue {
                lookfrom,
                lookat,
                up,
                angle,
                dist_to_focus,
            })
        }
    };
    Ok(())
}
