use quakeworld::bsp::{TextureMip, TextureParsed};
use quakeworld::texture::atlas::Atlas;
use quakeworld::texture::atlas::Statistics;
use std::error::Error;

#[test]
pub fn atlas_integration() -> Result<(), Box<dyn Error>> {
    let mut textures: Vec<_> = vec![];
    for n in 1..16 {
        let name = format!("texture_{}x{}", n, n);
        let mut data = vec![];
        for _ in 0..n * n {
            data.push(n as u8);
        }
        let tm = TextureMip {
            width: n,
            height: n,
            data,
        };
        let mut mip_levels = vec![];
        mip_levels.push(tm);
        let t = TextureParsed { name, mip_levels };
        textures.push(t);
    }

    let statistics = Statistics::gather(&textures, 0);
    let mut atlas = Atlas::new(
        statistics.minimum_box.width * 2,
        statistics.minimum_box.height * 2,
    );
    assert_eq!(statistics.textures.len(), textures.len());
    atlas.insert_textures(statistics.textures);

    let atlas_texture = atlas.generate_texture(&textures)?;

    assert_eq!(atlas_texture.textures.len(), textures.len());
    assert_eq!(atlas_texture.data.len() as u32, atlas_texture.size.area());

    for t in &atlas_texture.textures {
        for y in 0..t.size.height {
            let texture_index =
                t.position.x + y * atlas.size.width + t.position.y * atlas.size.width;
            for i in 0..t.size.width {
                let index = i + texture_index;
                assert_eq!(t.size.width as u8, atlas_texture.data[index as usize]);
            }
        }
    }

    Ok(())
}
