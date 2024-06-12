use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use image::io::Reader as ImageReader;
use image::GenericImageView;

fn main() {
    // Specify the directory where your PNG images are located
    let out_dir = env::var("OUT_DIR").unwrap();
    //let out_dir = "static";
    let image_dir = "static/pixel-art";
    let output_file_path = Path::new(&out_dir).join("images.rs");

    // Create the output file where the generated code will be written
    let mut output_file = File::create(output_file_path).unwrap();

    // Write the preamble for the Rust source file
    writeln!(&mut output_file, "pub mod images {{").unwrap();

    // Iterate over each PNG file in the image directory
    for entry in fs::read_dir(image_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("png") {
            let file_stem = path.file_stem().and_then(|stem| stem.to_str()).unwrap();
            let img = ImageReader::open(&path).unwrap().decode().unwrap();

            let (width, height) = img.dimensions();
            let pixels = img.to_rgba8().into_raw();

            // Write the image data to the Rust source file as a constant array
            writeln!(
                &mut output_file,
                "    pub const {}: [u8; {}] = {:?};",
                file_stem.to_uppercase(),
                pixels.len(),
                pixels
            )
            .unwrap();
            writeln!(
                &mut output_file,
                "    pub const {}_WIDTH: u32 = {};",
                file_stem.to_uppercase(),
                width
            )
            .unwrap();
            writeln!(
                &mut output_file,
                "    pub const {}_HEIGHT: u32 = {};",
                file_stem.to_uppercase(),
                height
            )
            .unwrap();
        }
    }

    // Close the module in the Rust source file
    writeln!(&mut output_file, "}}").unwrap();
}
