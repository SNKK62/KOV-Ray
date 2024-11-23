pub mod ast;
pub mod parser;

// ref: https://github.com/msakuta/ruscal/blob/ed869ab38ba0608b75ec63040bcc06eb8a6fc5d7/src/lib.rs
// use std::{collections::HashMap, sync::atomic::AtomicBool};
use std::sync::atomic::AtomicBool;

pub struct Args {
    pub source: Option<String>,
    pub output: String,
    pub show_ast: bool,
}

impl Args {
    pub fn new() -> Self {
        Self {
            source: None,
            output: "".to_string(),
            show_ast: false,
        }
    }
}

impl Default for Args {
    fn default() -> Self {
        Self::new()
    }
}

pub static DEBUG: AtomicBool = AtomicBool::new(false);

pub fn parse_args() -> Option<Args> {
    let mut source = None;
    let mut output = None;
    let mut show_ast = false;
    let mut show_help = false;
    let mut args_is_empty = true;

    let mut args = std::env::args();
    let exe = args.next();
    let mut next_arg = args.next();
    while let Some(arg) = next_arg {
        match &arg as &str {
            "-h" => show_help = true,
            "-o" => output = args.next(),
            "-a" => show_ast = true,
            _ => {
                if source.is_none() {
                    source = Some(arg);
                } else {
                    println!("More than 1 file name is specified");
                    return None;
                }
            }
        }
        args_is_empty = false;
        next_arg = args.next();
    }

    if show_help || args_is_empty {
        let options = r#"    -o       file  Specify output file
    -a       Show parsed AST
        }"#;
        println!(
            r#"Usage: {exe} [options] [source.txt]

Options:
{options}
    -a       Show AST
    -h       Display help
"#,
            exe = exe.unwrap(),
            options = options
        );
        return None;
    }

    Some(Args {
        source,
        output: output.unwrap_or_else(|| "ast.txt".to_string()),
        show_ast,
    })
}

#[macro_export]
macro_rules! dprintln {
    ($fmt:literal) => {
        #[cfg(not(target_arch = "wasm32"))]
        if ::ruscal::DEBUG.load(std::sync::atomic::Ordering::Relaxed) {
            println!($fmt);
        }
    };
    ($fmt:literal, $($args:expr),*) => {
        #[cfg(not(target_arch = "wasm32"))]
        if ::ruscal::DEBUG.load(std::sync::atomic::Ordering::Relaxed) {
            println!($fmt, $($args),*);
        }
    };
}
