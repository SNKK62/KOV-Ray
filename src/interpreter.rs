mod expression;
mod funcs;
mod value;

use std::collections::HashMap;

type Variables = HashMap<String, value::Value>;

const COLOR_MAX: f64 = 255.0;
