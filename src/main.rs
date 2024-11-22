use kov_ray::parser;
use std::io::Write;

fn main() {
    let args = kov_ray::parse_args().unwrap_or_else(|| std::process::exit(1));
    let source_file_name = args.source.unwrap();
    let source = std::fs::read_to_string(&source_file_name).unwrap_or_else(|e| {
        eprintln!("Failed to read file {}: {}", source_file_name, e);
        std::process::exit(1);
    });
    let output = args.output;
    let ast = parser::statements_finish(parser::Span::new(&source)).unwrap_or_else(|e| {
        eprintln!("Failed to parse file {}:\n {}", source_file_name, e);
        std::process::exit(1)
    });
    if args.show_ast {
        println!("{:#?}", ast);
    }
    let mut output_file = std::fs::File::create(output.clone()).unwrap_or_else(|e| {
        eprintln!("Failed to create file {}: {}", output, e);
        std::process::exit(1);
    });
    let output_ast = format!("{:#?}", ast);
    output_file
        .write_all(output_ast.as_bytes())
        .unwrap_or_else(|e| {
            eprintln!("Failed to write to file {}: {}", output, e);
            std::process::exit(1);
        });
}