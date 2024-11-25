mod expression;
mod funcs;
use funcs::standard_functions;
mod object;
mod statement;
use statement::eval_stmt;
mod value;
use value::ConfigValue;

use crate::ast::AST;

use pg_indicator::{PGOutput, PGStyle, ProgressBar};
use rand::Rng;

use ray_tracer_rs::{
    camera::Camera,
    hittable::{BvhNode, Hittable},
    vec3::Color,
};
use std::{collections::HashMap, rc::Rc};

type Variables = HashMap<String, value::Value>;

const COLOR_MAX: f64 = 255.0;

fn eval_ast(ast: &AST) -> (Vec<Rc<dyn Hittable>>, ConfigValue, Camera) {
    let mut world = Vec::new();
    let mut config = None;
    let mut camera_config = None;
    let mut variables = Variables::new();
    let funcs = standard_functions();
    for stmt in ast.iter() {
        eval_stmt(
            stmt,
            &mut variables,
            &funcs,
            &mut world,
            &mut config,
            &mut camera_config,
        );
    }
    if config.is_none() {
        panic!("Config not found");
    }
    if camera_config.is_none() {
        panic!("Camera is not found");
    }

    let config = config.unwrap();
    let camera_config = camera_config.unwrap();

    let camera = Camera::new(
        camera_config.lookfrom,
        camera_config.lookat,
        camera_config.up,
        camera_config.angle,
        config.width / config.height,
        0.0,
        camera_config.dist_to_focus,
        0.0,
        1.0,
    );
    (world, config, camera)
}

fn clamp(x: f64, min: f64, max: f64) -> f64 {
    if x < min {
        return min;
    }
    if x > max {
        return max;
    }
    x
}

fn get_rgb(color: &Color, samples_per_pixel: usize) -> (u8, u8, u8) {
    let scale = 1.0 / samples_per_pixel as f64;
    let min = 0.0;
    let max = 0.999;
    // gamma correction
    let r = (255.999 * clamp((color.x() * scale).sqrt(), min, max)).round() as u8;
    let g = (255.999 * clamp((color.y() * scale).sqrt(), min, max)).round() as u8;
    let b = (255.999 * clamp((color.z() * scale).sqrt(), min, max)).round() as u8;
    (r, g, b)
}

pub fn interpret(ast: &AST, show_progress: bool) -> (Vec<u8>, u32, u32) {
    let (mut world, config, camera) = eval_ast(ast);
    // TODO: apply motion blur
    let world = BvhNode::new(&mut world, 0.0, 0.0);

    let width = config.width.round() as u32;
    let height = config.height.round() as u32;
    let samples_per_pixel = config.samples_per_pixel.round() as usize;
    let max_depth = config.max_depth.round() as usize;
    let background = config.background;

    // image buffer (1d flatten array of rgb values)
    let mut buffer: Vec<u8> = Vec::new();

    let mut pg = ProgressBar::new(
        (width * height) as usize,
        PGStyle::Fraction,
        PGOutput::Stdout,
    );

    for j in (0..height).rev() {
        for i in 0..width {
            let mut pixel_color = Color::zero();
            for _ in 0..samples_per_pixel {
                let u = (i as f64 + rand::thread_rng().gen_range(0.0..1.0)) / (width - 1) as f64;
                let v = (j as f64 + rand::thread_rng().gen_range(0.0..1.0)) / (height - 1) as f64;

                let ray = camera.get_ray(u, v);
                pixel_color += ray.color(&background, &world, max_depth);
            }
            if show_progress {
                pg.update();
            }
            let (r, g, b) = get_rgb(&pixel_color, samples_per_pixel);
            buffer.extend_from_slice(&[r, g, b]);
        }
    }

    (buffer, width, height)
}
