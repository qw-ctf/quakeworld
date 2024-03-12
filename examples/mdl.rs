
use std::error::Error;
use std::fs::File;
use std::env;

use std::path::Path;
use std::io::{BufWriter, Read};

use quakeworld::mdl::Mdl;
use quakeworld::lmp::Palette;

fn parse_file(filename: String) -> Result<bool, Box<dyn Error>> {
    // read the file into a buffer
    let file = match File::open(filename) {
        Ok(file) => file,
        Err(err) => {
            return Err(Box::new(err))
        }
    };
    let mdl = Mdl::load(file)?;

    let mut file = match File::open("data/palette.lmp") {
        Ok(file) => file,
        Err(err) => {
            return Err(Box::new(err))
        }
    };
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    println!("{}", buf.len());
    let palette = Palette::from(buf)?;

    println!("we work?");
// For reading and opening files

    let path = Path::new(r"./image.png");
    let file = File::create(path).unwrap();
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, mdl.skin_width, mdl.skin_height * mdl.skins.len() as u32); // Width is 2 pixels and height is 1.
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    //encoder.set_source_gamma(png::ScaledFloat::from_scaled(45455)); // 1.0 / 2.2, scaled by 100000
    //encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2));     // 1.0 / 2.2, unscaled, but rounded
    /*let source_chromaticities = png::SourceChromaticities::new(     // Using unscaled instantiation here
        (0.31270, 0.32900),
        (0.64000, 0.33000),
        (0.30000, 0.60000),
        (0.15000, 0.06000)
    );
    encoder.set_source_chromaticities(source_chromaticities);
    */
    let mut writer = encoder.write_header().unwrap();

    for s in mdl.skins {
        let mut buffer = Vec::new();
        palette.apply(s.data, &mut buffer)?;
        writer.write_image_data(&buffer)?;
    }
    return Ok(true)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("need to supply a mdl");
        return
    }
    let filename = &args[1];
    match parse_file(filename.to_string()) {
        Ok(..) => {
            println!("{} parsed.", filename);
        }
        Err(err) => {
            eprintln!("error in file {}: {}", filename, err);
        }
    }

}
