use image::io::Reader as ImageReader;
use image::GenericImageView;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let image_dir = "static/pixel-art";
    let output_file_path = Path::new(&out_dir).join("pixel_art.rs");
    let mut output_file = File::create(output_file_path).unwrap();

    writeln!(&mut output_file, "pub mod pixel_art {{").unwrap();

    for entry in fs::read_dir(image_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("png") {
            let file_stem = path.file_stem().and_then(|stem| stem.to_str()).unwrap();
            let img = ImageReader::open(&path).unwrap().decode().unwrap();
            let (width, height) = img.dimensions();

            if width != 8 || height != 8 {
                panic!("Image dimensions must be 8x8");
            }

            let pixels = img.to_rgb8().into_raw();

            writeln!(
                &mut output_file,
                "    pub const {}: [[u8; 3]; 64] = [",
                file_stem.to_uppercase()
            )
            .unwrap();

            for i in 0..64 {
                let index = i * 3;
                let r = pixels[index];
                let g = pixels[index + 1];
                let b = pixels[index + 2];
                writeln!(&mut output_file, "        [{}, {}, {}],", r, g, b).unwrap();
            }

            writeln!(&mut output_file, "    ];").unwrap();
        }
    }

    writeln!(&mut output_file, "}}").unwrap();
}
