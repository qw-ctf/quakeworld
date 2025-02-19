use std::io::Write;

use crate::lmp::Palette;

pub fn from_palette_data(palette: &Palette, data: &Vec<u8>, width: u32, height: u32) -> Vec<u8> {
    let mut pngbuf = vec![];
    {
        let mut encoder = png::Encoder::new(std::io::Cursor::new(&mut pngbuf), width, height);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_source_gamma(png::ScaledFloat::from_scaled(45455)); // 1.0 / 2.2, scaled by 100000
        encoder.set_source_gamma(png::ScaledFloat::new(1.0 / 2.2)); // 1.0 / 2.2, unscaled, but rounded

        let mut writer = encoder.write_header().unwrap();

        let mut converted_data = vec![];
        let len_data = data.len();
        palette.apply(&data, &mut converted_data);
        writer.write_image_data(&converted_data).unwrap();

        let mut writer = writer.stream_writer().unwrap();

        writer.finish();
    }
    pngbuf
}
