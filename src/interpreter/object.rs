use super::{expression::eval_expr, funcs::Functions, value::Value, Variables};
use crate::ast::{
    object::{AffineProperties, RotateAxis},
    Object,
};
use ray_tracer_rs::{
    hittable::{
        BvhNode, Cuboid, Hittable, RotateX, RotateY, RotateZ, Sphere, Translation, XYRect, XZRect,
        YZRect,
    },
    vec3::Vec3,
};
use std::sync::Arc;

pub(super) fn eval_object(
    object: &Object,
    variables: &mut Variables,
    funcs: &Functions,
    world: &mut Vec<Arc<dyn Hittable>>,
) {
    let (mut obj, affine): (Arc<dyn Hittable>, &Vec<AffineProperties>) = match object {
        Object::Sphere {
            center,
            radius,
            material,
            affine,
        } => {
            let (center, radius) = match (
                eval_expr(center, variables, funcs),
                eval_expr(radius, variables, funcs),
            ) {
                (Value::Vec3(x, y, z), Value::Num(num)) => (Vec3::new(x, y, z), num),
                _ => panic!("Invalid arguments for Sphere"),
            };
            let material = match eval_expr(material, variables, funcs) {
                Value::Material(material) => material,
                _ => panic!("Invalid arguments for Sphere"),
            };
            (Arc::new(Sphere::new(&center, radius, material)), affine)
        }
        Object::Box {
            vertex,
            material,
            affine,
        } => {
            let (vertex1, vertex2) = match (
                eval_expr(&vertex.0, variables, funcs),
                eval_expr(&vertex.1, variables, funcs),
            ) {
                (Value::Vec3(x1, y1, z1), Value::Vec3(x2, y2, z2)) => {
                    (Vec3::new(x1, y1, z1), Vec3::new(x2, y2, z2))
                }
                _ => panic!("Invalid arguments for Box"),
            };
            let material = match eval_expr(material, variables, funcs) {
                Value::Material(material) => material,
                _ => panic!("Invalid arguments for Box"),
            };
            (Arc::new(Cuboid::new(&vertex1, &vertex2, material)), affine)
        }
        Object::Plane {
            vertex,
            material,
            affine,
        } => {
            let (vertex1, vertex2) = match (
                eval_expr(&vertex.0, variables, funcs),
                eval_expr(&vertex.1, variables, funcs),
            ) {
                (Value::Vec3(x1, y1, z1), Value::Vec3(x2, y2, z2)) => {
                    (Vec3::new(x1, y1, z1), Vec3::new(x2, y2, z2))
                }
                _ => panic!("Invalid arguments for Plane"),
            };
            let material = match eval_expr(material, variables, funcs) {
                Value::Material(material) => material,
                _ => panic!("Invalid arguments for Plane"),
            };
            let rect: Arc<dyn Hittable> = if vertex1.x() == vertex2.x() {
                Arc::new(YZRect::new(
                    vertex1.y(),
                    vertex2.y(),
                    vertex1.z(),
                    vertex2.z(),
                    vertex1.x(),
                    material,
                ))
            } else if vertex1.y() == vertex2.y() {
                Arc::new(XZRect::new(
                    vertex1.x(),
                    vertex2.x(),
                    vertex1.z(),
                    vertex2.z(),
                    vertex1.y(),
                    material,
                ))
            } else if vertex1.z() == vertex2.z() {
                Arc::new(XYRect::new(
                    vertex1.x(),
                    vertex2.x(),
                    vertex1.y(),
                    vertex2.y(),
                    vertex1.z(),
                    material,
                ))
            } else {
                panic!("Invalid vertex for Plane")
            };
            (rect, affine)
        }
        Object::Objects { objects, affine } => {
            let mut objs = Vec::new();
            for obj in objects.iter() {
                eval_object(obj, variables, funcs, &mut objs);
            }
            // TODO: apply motion blur
            (Arc::new(BvhNode::new(&mut objs, 0.0, 0.0)), affine)
        }
    };
    for af in affine.iter() {
        match af {
            AffineProperties::Translation(expr) => {
                let offset = match eval_expr(expr, variables, funcs) {
                    Value::Vec3(x, y, z) => Vec3::new(x, y, z),
                    _ => panic!("Invalid arguments for Sphere"),
                };
                obj = Arc::new(Translation::new(obj, offset));
            }
            AffineProperties::Rotate(rotate) => {
                let angle = match eval_expr(&rotate.expr, variables, funcs) {
                    Value::Num(num) => num,
                    _ => panic!("Invalid arguments for Rotate"),
                };
                obj = match rotate.axis {
                    RotateAxis::X => Arc::new(RotateX::new(obj, angle)),
                    RotateAxis::Y => Arc::new(RotateY::new(obj, angle)),
                    RotateAxis::Z => Arc::new(RotateZ::new(obj, angle)),
                };
            }
        }
    }
    world.push(obj);
}
