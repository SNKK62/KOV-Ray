use super::{expression::eval_expr, funcs::Functions, value::Value, EvalError, Variables};
use crate::ast::{
    object::{AffineProperties, RotateAxis},
    Object,
};
use ray_tracer_rs::{
    hittable::{
        BvhNode, Cuboid, HittableEnum, RotateX, RotateY, RotateZ, Sphere, Translation, XYRect,
        XZRect, YZRect,
    },
    vec3::Vec3,
};

use std::boxed::Box;

pub(super) fn eval_object<'a>(
    object: &'a Object,
    variables: &mut Variables,
    funcs: &Functions,
    world: &mut Vec<HittableEnum>,
) -> Result<(), EvalError<'a>> {
    let (mut obj, affine): (HittableEnum, &Vec<AffineProperties>) = match object {
        Object::Sphere {
            center,
            radius,
            material,
            affine,
        } => {
            let center_val = eval_expr(center, variables, funcs)?;
            let radius = eval_expr(radius, variables, funcs)?;
            let (center, radius) = match (center_val, radius) {
                (Value::Vec3(x, y, z), Value::Num(num)) => (Vec3::new(x, y, z), num),
                _ => {
                    return Err(EvalError {
                        span: Some(center.span),
                        message: "Invalid center for Sphere".to_string(),
                    });
                }
            };
            let material_val = eval_expr(material, variables, funcs)?;
            let material = match material_val {
                Value::Material(material) => material,
                _ => {
                    return Err(EvalError {
                        span: Some(material.span),
                        message: "Invalid material for Sphere".to_string(),
                    });
                }
            };
            (
                HittableEnum::Sphere(Sphere::new(&center, radius, material)),
                affine,
            )
        }
        Object::Box {
            vertex,
            material,
            affine,
        } => {
            let vertex1 = eval_expr(&vertex.0, variables, funcs)?;
            let vertex2 = eval_expr(&vertex.1, variables, funcs)?;
            let (vertex1, vertex2) = match (vertex1, vertex2) {
                (Value::Vec3(x1, y1, z1), Value::Vec3(x2, y2, z2)) => {
                    if (x1 == x2 || y1 == y2 || z1 == z2) {
                        return Err(EvalError {
                            span: Some(vertex.0.span),
                            message: "Box vertexes should be completely different".to_string(),
                        });
                    }
                    (Vec3::new(x1, y1, z1), Vec3::new(x2, y2, z2))
                }
                _ => {
                    return Err(EvalError {
                        span: Some(vertex.0.span),
                        message: "Invalid vertexesfor Box".to_string(),
                    });
                }
            };
            let material_val = eval_expr(material, variables, funcs)?;
            let material = match material_val {
                Value::Material(material) => material,
                _ => {
                    return Err(EvalError {
                        span: Some(material.span),
                        message: "Invalid material for Box".to_string(),
                    });
                }
            };
            (
                HittableEnum::Cuboid(Cuboid::new(&vertex1, &vertex2, material)),
                affine,
            )
        }
        Object::Plane {
            vertex,
            material,
            affine,
        } => {
            let (vertex1, vertex2) = match (
                eval_expr(&vertex.0, variables, funcs)?,
                eval_expr(&vertex.1, variables, funcs)?,
            ) {
                (Value::Vec3(x1, y1, z1), Value::Vec3(x2, y2, z2)) => {
                    (Vec3::new(x1, y1, z1), Vec3::new(x2, y2, z2))
                }
                _ => {
                    return Err(EvalError {
                        span: Some(vertex.0.span),
                        message: "Invalid vertex for Plane".to_string(),
                    });
                }
            };
            let material = match eval_expr(material, variables, funcs)? {
                Value::Material(material) => material,
                _ => {
                    return Err(EvalError {
                        span: Some(material.span),
                        message: "Invalid material for Plane".to_string(),
                    });
                }
            };
            let is_x_same = vertex1.x() == vertex2.x();
            let is_y_same = vertex1.y() == vertex2.y();
            let is_z_same = vertex1.z() == vertex2.z();
            let rect: HittableEnum = if is_x_same && !is_y_same && !is_z_same {
                HittableEnum::YZRect(YZRect::new(
                    vertex1.y(),
                    vertex2.y(),
                    vertex1.z(),
                    vertex2.z(),
                    vertex1.x(),
                    material,
                ))
            } else if is_y_same && !is_x_same && !is_z_same {
                HittableEnum::XZRect(XZRect::new(
                    vertex1.x(),
                    vertex2.x(),
                    vertex1.z(),
                    vertex2.z(),
                    vertex1.y(),
                    material,
                ))
            } else if is_z_same && !is_x_same && !is_y_same {
                HittableEnum::XYRect(XYRect::new(
                    vertex1.x(),
                    vertex2.x(),
                    vertex1.y(),
                    vertex2.y(),
                    vertex1.z(),
                    material,
                ))
            } else {
                return Err(EvalError {
                    span: Some(vertex.0.span),
                    message: "Invalid vertex for Plane".to_string(),
                });
            };
            (rect, affine)
        }
        Object::Objects { objects, affine } => {
            let mut objs = Vec::new();
            for obj in objects.iter() {
                eval_object(obj, variables, funcs, &mut objs)?;
            }
            // TODO: apply motion blur
            (
                HittableEnum::BvhNode(Box::new(BvhNode::new(&mut objs, 0.0, 0.0))),
                affine,
            )
        }
    };
    for af in affine.iter() {
        match af {
            AffineProperties::Translation(expr) => {
                let offset = match eval_expr(expr, variables, funcs)? {
                    Value::Vec3(x, y, z) => Vec3::new(x, y, z),
                    _ => {
                        return Err(EvalError {
                            span: Some(expr.span),
                            message: "Invalid arguments for Translation".to_string(),
                        })
                    }
                };
                obj = HittableEnum::Translation(Translation::new(obj, offset));
            }
            AffineProperties::Rotate(rotate) => {
                let angle = match eval_expr(&rotate.expr, variables, funcs)? {
                    Value::Num(num) => num,
                    _ => {
                        return Err(EvalError {
                            span: Some(rotate.expr.span),
                            message: "Invalid arguments for Rotate".to_string(),
                        })
                    }
                };
                obj = match rotate.axis {
                    RotateAxis::X => HittableEnum::RotateX(Box::new(RotateX::new(obj, angle))),
                    RotateAxis::Y => HittableEnum::RotateY(Box::new(RotateY::new(obj, angle))),
                    RotateAxis::Z => HittableEnum::RotateZ(Box::new(RotateZ::new(obj, angle))),
                };
            }
        }
    }
    world.push(obj);
    Ok(())
}
