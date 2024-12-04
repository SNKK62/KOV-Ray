mod expression;
mod funcs;
use funcs::standard_functions;
mod object;
mod statement;
use statement::eval_stmt;
mod value;
pub use value::ConfigValue;

use crate::ast::AST;

#[cfg(feature = "execution")]
use pg_indicator::{PGOutput, PGStyle, ProgressBar};
use rand::Rng;

use ray_tracer_rs::{
    camera::Camera,
    hittable::{BvhNode, HittableEnum},
    vec3::Color,
};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    thread,
    time::Instant,
};

type Variables = HashMap<String, value::Value>;

const COLOR_MAX: f64 = 255.0;

pub fn eval_ast(ast: &AST) -> (HittableEnum, ConfigValue, Camera) {
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
    // TODO: apply motion blur
    let world = HittableEnum::BvhNode(Box::new(BvhNode::new(&mut world, 0.0, 0.0)));

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

#[cfg(feature = "execution")]
pub fn interpret(ast: &AST) -> (Vec<u8>, u32, u32) {
    let (world, config, camera) = eval_ast(ast);
    let world = Arc::new(world);

    let width = config.width.round() as u32;
    let height = config.height.round() as u32;
    let samples_per_pixel = config.samples_per_pixel.round() as usize;
    let max_depth = config.max_depth.round() as usize;
    let background = config.background;

    let pg = Arc::new(RwLock::new(ProgressBar::new(
        (width * height) as usize,
        PGStyle::Fraction,
        PGOutput::Stdout,
    )));
    let camera = Arc::new(camera);

    // image buffer (1d flatten array of rgb values)
    let buffer = Arc::new(RwLock::new(vec![0; (width * height * 3) as usize]));
    let handles: Vec<_> = (0..height)
        .map(|j| {
            let buffer = Arc::clone(&buffer);
            let world = Arc::clone(&world);
            let camera = Arc::clone(&camera);
            let pg = Arc::clone(&pg);

            thread::spawn(move || {
                let mut rng = rand::thread_rng();
                for i in 0..width {
                    let mut pixel_color = Color::zero();
                    for _ in 0..samples_per_pixel {
                        let u = (i as f64 + rng.gen_range(0.0..1.0)) / (width - 1) as f64;
                        let v = (j as f64 + rng.gen_range(0.0..1.0)) / (height - 1) as f64;
                        let ray = camera.get_ray(u, v);
                        pixel_color += ray.color(&background, &world, max_depth);
                    }
                    let mut buf = buffer.write().unwrap();
                    let (r, g, b) = pixel_color.get_color(samples_per_pixel as i64);
                    buf[((height - j - 1) * width * 3 + i * 3) as usize] = r;
                    buf[((height - j - 1) * width * 3 + i * 3 + 1) as usize] = g;
                    buf[((height - j - 1) * width * 3 + i * 3 + 2) as usize] = b;
                    let mut pg = pg.write().unwrap();
                    pg.update();
                }
            })
        })
        .collect();

    let start = Instant::now();
    for handle in handles {
        handle.join().unwrap();
    }
    let duration = start.elapsed();
    println!("Time elapsed in expensive_function() is: {:?}", duration);
    (
        Arc::try_unwrap(buffer).unwrap().into_inner().unwrap(),
        width,
        height,
    )
}
