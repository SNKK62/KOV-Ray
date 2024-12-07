use super::{expression::eval_expr, funcs::Functions, value::Value, Variables};
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

pub(super) fn eval_object(
    object: &Object,
    variables: &mut Variables,
    funcs: &Functions,
    world: &mut Vec<HittableEnum>,
) -> Result<(), String> {
    let (mut obj, affine): (HittableEnum, &Vec<AffineProperties>) = match object {
        Object::Sphere {
            center,
            radius,
            material,
            affine,
        } => {
            let center = eval_expr(center, variables, funcs)?;
            let radius = eval_expr(radius, variables, funcs)?;
            let (center, radius) = match (center, radius) {
                (Value::Vec3(x, y, z), Value::Num(num)) => (Vec3::new(x, y, z), num),
                _ => return Err("Invalid arguments for Sphere".to_string()),
            };
            let material = eval_expr(material, variables, funcs)?;
            let material = match material {
                Value::Material(material) => material,
                _ => return Err("Invalid arguments for Sphere".to_string()),
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
                    (Vec3::new(x1, y1, z1), Vec3::new(x2, y2, z2))
                }
                _ => return Err("Invalid arguments for Box".to_string()),
            };
            let material = eval_expr(material, variables, funcs)?;
            let material = match material {
                Value::Material(material) => material,
                _ => return Err("Invalid arguments for Box".to_string()),
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
                _ => return Err("Invalid arguments for Plane".to_string()),
            };
            let material = match eval_expr(material, variables, funcs)? {
                Value::Material(material) => material,
                _ => return Err("Invalid arguments for Plane".to_string()),
            };
            let rect: HittableEnum = if vertex1.x() == vertex2.x() {
                HittableEnum::YZRect(YZRect::new(
                    vertex1.y(),
                    vertex2.y(),
                    vertex1.z(),
                    vertex2.z(),
                    vertex1.x(),
                    material,
                ))
            } else if vertex1.y() == vertex2.y() {
                HittableEnum::XZRect(XZRect::new(
                    vertex1.x(),
                    vertex2.x(),
                    vertex1.z(),
                    vertex2.z(),
                    vertex1.y(),
                    material,
                ))
            } else if vertex1.z() == vertex2.z() {
                HittableEnum::XYRect(XYRect::new(
                    vertex1.x(),
                    vertex2.x(),
                    vertex1.y(),
                    vertex2.y(),
                    vertex1.z(),
                    material,
                ))
            } else {
                return Err("Invalid vertex for Plane".to_string());
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
                    _ => return Err("Invalid arguments for Sphere".to_string()),
                };
                obj = HittableEnum::Translation(Translation::new(obj, offset));
            }
            AffineProperties::Rotate(rotate) => {
                let angle = match eval_expr(&rotate.expr, variables, funcs)? {
                    Value::Num(num) => num,
                    _ => return Err("Invalid arguments for Rotate".to_string()),
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
