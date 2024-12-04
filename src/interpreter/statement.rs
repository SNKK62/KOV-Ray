use super::{
    expression::eval_expr,
    funcs::Functions,
    object::eval_object,
    value::{CameraConfigValue, ConfigValue, Value},
    Variables, COLOR_MAX,
};
use crate::ast::Statement;
use ray_tracer_rs::{
    hittable::HittableEnum,
    vec3::{Color, Vec3},
};

pub(super) fn eval_stmt<'a>(
    ast: &Statement<'a>,
    variables: &mut Variables,
    funcs: &Functions<'a>,
    world: &mut Vec<HittableEnum>,
    config: &mut Option<ConfigValue>,
    camera_config: &mut Option<CameraConfigValue>,
) {
    match ast {
        Statement::Expression(expr) => {
            let _ = eval_expr(expr, variables, funcs);
        }
        Statement::VarAssign { name, ex, .. } => {
            let value = eval_expr(ex, variables, funcs);
            variables.insert(name.to_string(), value);
        }
        Statement::If {
            cond,
            stmts,
            else_stmts,
            ..
        } => {
            if eval_expr(cond, variables, funcs).to_bool() {
                for stmt in stmts.iter() {
                    eval_stmt(stmt, variables, funcs, world, config, camera_config);
                }
            } else if let Some(ref else_stmts) = else_stmts {
                for stmt in else_stmts.iter() {
                    eval_stmt(stmt, variables, funcs, world, config, camera_config);
                }
            }
        }
        Statement::While { cond, stmts, .. } => {
            while eval_expr(cond, variables, funcs).to_bool() {
                for stmt in stmts.iter() {
                    match stmt {
                        Statement::Break => break,
                        Statement::Continue => continue,
                        _ => eval_stmt(stmt, variables, funcs, world, config, camera_config),
                    }
                }
            }
        }
        Statement::Break | Statement::Continue => {}
        Statement::Object { object, .. } => {
            eval_object(object, variables, funcs, world);
        }
        Statement::Config { config: c, .. } => {
            let width = match eval_expr(&c.width, variables, funcs) {
                Value::Num(n) => n,
                _ => panic!("Invalid width"),
            };
            let height = match eval_expr(&c.height, variables, funcs) {
                Value::Num(n) => n,
                _ => panic!("Invalid height"),
            };
            let samples_per_pixel = match eval_expr(&c.samples_per_pixel, variables, funcs) {
                Value::Num(n) => n,
                _ => panic!("Invalid samples_per_pixel"),
            };
            let max_depth = match c.max_depth.as_ref() {
                Some(expr) => match eval_expr(expr, variables, funcs) {
                    Value::Num(n) => n,
                    _ => panic!("Invalid max_depth"),
                },
                None => 100.0,
            };
            let background = match c.background.as_ref() {
                Some(expr) => match eval_expr(expr, variables, funcs) {
                    Value::Vec3(x, y, z) => Color::new(x, y, z) / COLOR_MAX,
                    _ => panic!("Invalid sky_color"),
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
            let lookfrom = match eval_expr(&c.lookfrom, variables, funcs) {
                Value::Vec3(x, y, z) => Vec3::new(x, y, z),
                _ => panic!("Invalid look_from"),
            };
            let lookat = match eval_expr(&c.lookat, variables, funcs) {
                Value::Vec3(x, y, z) => Vec3::new(x, y, z),
                _ => panic!("Invalid look_at"),
            };
            let up = match c.up.as_ref() {
                Some(expr) => match eval_expr(expr, variables, funcs) {
                    Value::Vec3(x, y, z) => Vec3::new(x, y, z),
                    _ => panic!("Invalid up"),
                },
                None => Vec3::new(0.0, 1.0, 0.0),
            };
            let angle = match eval_expr(&c.angle, variables, funcs) {
                Value::Num(n) => n,
                _ => panic!("Invalid angle"),
            };
            let dist_to_focus = match c.dist_to_focus.as_ref() {
                Some(expr) => match eval_expr(expr, variables, funcs) {
                    Value::Num(n) => n,
                    _ => panic!("Invalid dist_to_focus"),
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
    }
}
