use image::RgbImage;
use kov_ray::{interpreter::interpret, parser};

fn main() {
    let args = kov_ray::parse_args().unwrap_or_else(|| std::process::exit(1));
    let source_file_name = args.source.unwrap();
    let source = std::fs::read_to_string(&source_file_name).unwrap_or_else(|e| {
        eprintln!("Failed to read file {}: {}", source_file_name, e);
        std::process::exit(1);
    });
    let output = args.output;
    let ast = parser::parse(&source).unwrap_or_else(|e| {
        eprintln!("Failed to parse file {}:\n {}", source_file_name, e);
        std::process::exit(1)
    });
    if args.show_ast {
        println!("{:#?}", ast);
    }
    let (image_buffer, width, height) = interpret(&ast, true);
    let img = RgbImage::from_raw(width, height, image_buffer).expect("incorrect image buffer size");

    img.save(output).expect("failed to save image");
}
