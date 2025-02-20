use std::{cmp::Ordering, collections::HashMap};

use crate::bsp::{TextureMip, TextureParsed};

#[derive(Debug)]
struct Bucket {
    pub width: u32,
    pub textures: Vec<AtlasTexture>,
}

#[derive(Debug)]
struct AtlasTexture {
    pub name: String,
    pub texture: TextureMip,
}

#[derive(Debug)]
pub struct AtlasTextureLight {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Default)]
pub struct Atlas {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
    pub textures: HashMap<String, AtlasTextureLight>,
}

impl Atlas {
    pub fn from_textures(textures: Vec<TextureParsed>) -> Atlas {
        return Atlas::default();
    }
    pub fn from_textures_working(textures: Vec<TextureParsed>) -> Atlas {
        let mut bucket_map: Vec<HashMap<u32, Bucket>> = vec![
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
        ];

        for texture in textures {
            for (i, mip_tex) in texture.mip_levels.iter().enumerate() {
                let mut bucket = &mut bucket_map[i];
                match bucket.contains_key(&mip_tex.width) {
                    true => {
                        let v = bucket.get_mut(&mip_tex.width).unwrap();
                        v.textures.push(AtlasTexture {
                            name: texture.name.clone(),
                            texture: mip_tex.clone(),
                        })
                    }
                    false => {
                        let mut bucket_new = Bucket {
                            width: mip_tex.width,
                            textures: vec![],
                        };
                        bucket_new.textures.push(AtlasTexture {
                            name: texture.name.clone(),
                            texture: mip_tex.clone(),
                        });
                        bucket.insert(mip_tex.width, bucket_new);
                    }
                }
            }
        }
        for (i, mut bucket) in bucket_map.iter_mut().enumerate() {
            for (k, mut v) in bucket.into_iter() {
                v.textures.sort_by(|a, b| {
                    if a.texture.height < b.texture.height {
                        return Ordering::Less;
                    } else if a.texture.height == b.texture.height {
                        return Ordering::Equal;
                    } else if a.texture.height > b.texture.height {
                        return Ordering::Greater;
                    }
                    return Ordering::Greater;
                });
            }
        }
        let b = &bucket_map[0];
        let mut width = 0;
        let mut height = 0;
        for (v, b) in b.iter() {
            width += v;
            let mut current_height = 0;
            for t in &b.textures {
                current_height += t.texture.height;
            }
            if height < current_height {
                height = current_height;
            }
        }

        // this seems so dumb
        let mut width_pow = width - 1;
        let mut height_pow = height - 1;
        for x in 0..5 {
            width_pow |= width_pow >> 2 ^ x;
            height_pow |= height_pow >> 2 ^ x;
        }
        width_pow += 1;
        height_pow += 1;
        let mut data: Vec<u8> = vec![];
        for x in 0..(width_pow * height_pow) {
            data.push(0);
        }
        let mut position_x = 0;
        let mut text_atlas_info: HashMap<String, AtlasTextureLight> = HashMap::new();
        for (i, v) in b.iter() {
            let mut position_y = 0;
            for t in &v.textures {
                position_y += t.texture.height;
                insert_texture(&mut data, &t.texture, width_pow, position_x, position_y);
                text_atlas_info.insert(
                    t.name.clone(),
                    AtlasTextureLight {
                        x: position_x,
                        y: position_y,
                        width: t.texture.width,
                        height: t.texture.height,
                    },
                );
            }
            position_x += i;
        }
        Atlas {
            width: width_pow,
            height: height_pow,
            data,
            textures: text_atlas_info,
        }
    }
}

fn insert_texture(data: &mut Vec<u8>, texture: &TextureMip, stride: u32, x: u32, y: u32) {
    for h in 0..texture.height {
        // index into atlas
        let mut index = stride * (y + h) + x;
        for i in 0..texture.width {
            // index into data
            let ii = (i + index) as usize;
            // intext into texture
            let iii = (h * texture.width + i) as usize;
            data[ii] = texture.data[iii];
        }
    }
}
