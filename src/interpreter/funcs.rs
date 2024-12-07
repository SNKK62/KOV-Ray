use std::collections::HashMap;

use super::value::Value;

use rand::Rng;

pub type Functions<'src> = HashMap<String, FnDecl>;

fn noarg_fn(f: fn() -> f64) -> FnDecl {
    FnDecl::Native(NativeFn {
        code: Box::new(move |_| Ok(Value::Num(f()))),
    })
}

fn unary_fn(f: fn(f64) -> f64, name: String) -> FnDecl {
    FnDecl::Native(NativeFn {
        code: Box::new(move |args| {
            let arg = match args.iter().next().expect("function missing argument") {
                Value::Num(n) => n,
                _ => return Err(format!("\"{:?}\" has invalid argument type", name)),
            };
            Ok(Value::Num(f(*arg)))
        }),
    })
}

fn binary_fn(f: fn(f64, f64) -> f64, name: String) -> FnDecl {
    FnDecl::Native(NativeFn {
        code: Box::new(move |args| {
            let mut args = args.iter();
            let lhs = match args.next().expect("function missing argument") {
                Value::Num(n) => n,
                _ => return Err(format!("\"{:?}\" has invalid argument type", name)),
            };
            let rhs = match args.next().expect("function missing argument") {
                Value::Num(n) => n,
                _ => return Err(format!("\"{:?}\"Invalid argument type", name)),
            };
            Ok(Value::Num(f(*lhs, *rhs)))
        }),
    })
}

pub fn standard_functions<'src>() -> Functions<'src> {
    let mut funcs = Functions::new();
    funcs.insert("sqrt".to_string(), unary_fn(f64::sqrt, "sqrt".to_string()));
    funcs.insert("sin".to_string(), unary_fn(f64::sin, "sin".to_string()));
    funcs.insert("cos".to_string(), unary_fn(f64::cos, "cos".to_string()));
    funcs.insert("tan".to_string(), unary_fn(f64::tan, "tan".to_string()));
    funcs.insert("asin".to_string(), unary_fn(f64::asin, "asin".to_string()));
    funcs.insert("acos".to_string(), unary_fn(f64::acos, "acos".to_string()));
    funcs.insert("atan".to_string(), unary_fn(f64::atan, "atan".to_string()));
    funcs.insert(
        "atan2".to_string(),
        binary_fn(f64::atan2, "atan2".to_string()),
    );
    funcs.insert("pow".to_string(), binary_fn(f64::powf, "pow".to_string()));
    funcs.insert("exp".to_string(), unary_fn(f64::exp, "exp".to_string()));
    funcs.insert("log".to_string(), binary_fn(f64::log, "log".to_string()));
    funcs.insert(
        "log10".to_string(),
        unary_fn(f64::log10, "log10".to_string()),
    );
    funcs.insert(
        "rand".to_string(),
        noarg_fn(|| rand::thread_rng().gen_range(0.0..1.0)),
    );
    funcs.insert(
        "len".to_string(),
        FnDecl::Native(NativeFn {
            code: Box::new(|args| {
                let arg = match args.iter().next().expect("function missing argument") {
                    Value::Vec3(x, y, z) => (x * x + y * y + z * z).sqrt(),
                    _ => return Err("\"len\" has invalid argument type".to_string()),
                };
                Ok(Value::Num(arg))
            }),
        }),
    );
    funcs
}

pub enum FnDecl {
    Native(NativeFn),
}

type NativeFnCode = dyn Fn(&[Value]) -> Result<Value, String>;
pub struct NativeFn {
    pub code: Box<NativeFnCode>,
}
