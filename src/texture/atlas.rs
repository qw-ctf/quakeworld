use std::{
    cmp::Ordering,
    collections::HashMap,
    fmt::Display,
    ops::{Add, Index},
};

use crate::bsp::{TextureMip, TextureParsed};

use super::error::{Error, Result};

#[derive(Debug)]
pub struct StatisticsMinMax<T> {
    pub min: T,
    pub max: T,
}

impl Default for StatisticsMinMax<u32> {
    fn default() -> Self {
        Self {
            min: u32::max_value(),
            max: u32::min_value(),
        }
    }
}

impl<T: std::cmp::PartialOrd + Clone> StatisticsMinMax<T> {
    fn apply(&mut self, value: T) {
        if value < self.min {
            self.min = value.clone();
        }
        if value > self.max {
            self.max = value.clone();
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct TextureBox {
    pub width: u32,
    pub height: u32,
    pub x: u32,
    pub y: u32,
    pub index: usize,
}

#[derive(Debug, Default)]
pub struct Statistics {
    pub width: StatisticsMinMax<u32>,
    pub height: StatisticsMinMax<u32>,
    pub textures: Vec<AtlasTextureBox>,
    pub minimum_box: TextureBox,
}

impl Statistics {
    pub fn gather(textures: &Vec<TextureParsed>, index: usize) -> Self {
        let mut s = Statistics::default();
        for (i, texture) in textures.iter().enumerate() {
            let t = &texture.mip_levels[index];
            s.width.apply(t.width);
            s.height.apply(t.height);
            s.textures.push(AtlasTextureBox {
                size: AtlasSize {
                    width: t.width,
                    height: t.height,
                },
                position: AtlasPosition { x: 0, y: 0 },
                index: i,
            })
        }
        let mut width_pow = s.width.max - 1;
        let mut height_pow = s.height.max - 1;
        for x in 0..5 {
            width_pow |= width_pow >> 2 ^ x;
            height_pow |= height_pow >> 2 ^ x;
        }
        width_pow += 1;
        height_pow += 1;
        s.minimum_box.width = width_pow;
        s.minimum_box.height = height_pow;
        s.textures = sort_by_size(s.textures.clone());
        s
    }
}

fn sort_by_size(textures: Vec<AtlasTextureBox>) -> Vec<AtlasTextureBox> {
    let mut textures_sorted: Vec<AtlasTextureBox> = textures.clone();

    textures_sorted.sort_by(|a, b| {
        let a_area = a.size.width * a.size.height;
        let b_area = b.size.width * b.size.height;
        // a_area.cmp(&b_area)
        b_area.cmp(&a_area)
    });
    textures_sorted
}

fn insert_texture(
    data: &mut Vec<u8>,
    texture: &TextureMip,
    stride: u32,
    x: u32,
    y: u32,
) -> Result<()> {
    for h in 0..texture.height {
        // index into atlas
        let mut index = stride * (y + h) + x;
        for i in 0..texture.width {
            // index into atlas data
            let ii = (i + index) as usize;
            // index into texture data
            let iii = (h * texture.width + i) as usize;
            if data.len() <= ii {
                return Err(Error::AtlasIndexAtlasTexture(data.len(), ii));
            }
            if texture.data.len() <= iii {
                return Err(Error::AtlasIndexTexture(texture.data.len(), iii));
            }
            data[ii] = texture.data[iii];
        }
    }
    Ok(())
}

#[derive(Debug, Default, Clone)]
pub struct AtlasPosition {
    pub x: u32,
    pub y: u32,
}

impl Display for AtlasPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(x:{} y:{})", self.x, self.y)
    }
}

impl Add for AtlasPosition {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct AtlasSize {
    pub width: u32,
    pub height: u32,
}

impl AtlasSize {
    pub fn area(&self) -> u32 {
        self.width * self.height
    }
}
impl Display for AtlasSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(w:{} h:{})", self.width, self.height)
    }
}

#[derive(Debug, Default, Clone)]
pub struct AtlasBox {
    pub position: AtlasPosition,
    pub size: AtlasSize,
}

impl Display for AtlasBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(size:{} position:{})", self.size, self.position)
    }
}

impl Ord for AtlasBox {
    fn cmp(&self, other: &Self) -> Ordering {
        self.size.width.cmp(&other.size.width)
    }
}

impl PartialOrd for AtlasBox {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.size.width.partial_cmp(&other.size.width)
    }
}

impl PartialEq for AtlasBox {
    fn eq(&self, other: &Self) -> bool {
        self.size.width.eq(&other.size.width)
    }
}

impl Eq for AtlasBox {}

impl AtlasBox {
    pub fn carve_out(&mut self, texture: &AtlasTextureBox) -> Option<(AtlasBox, AtlasBox)> {
        if self.size.width < texture.size.width {
            return None;
        };
        if self.size.height < texture.size.height {
            return None;
        };
        // box under the texture
        let mut b1_position = self.position.clone();
        b1_position.y += texture.size.height;

        let mut b1_size = self.size.clone();
        b1_size.width = texture.size.width;
        b1_size.height -= texture.size.height;

        // box to the right texture
        let mut b2_position = self.position.clone();
        b2_position.x += texture.size.width;

        let mut b2_size = self.size.clone();
        b2_size.width -= texture.size.width;
        b2_size.height = self.size.height;

        let b1 = AtlasBox {
            position: b1_position,
            size: b1_size,
        };
        let b2 = AtlasBox {
            position: b2_position,
            size: b2_size,
        };

        Some((b1, b2))
    }

    pub fn insert(&mut self, texture: &AtlasTextureBox) -> Option<(AtlasBox, AtlasBox)> {
        if self.size.width < texture.size.width {
            return None;
        };
        if self.size.height < texture.size.height {
            return None;
        };

        self.carve_out(texture)
    }
}

#[derive(Debug, Default, Clone)]
pub struct AtlasTextureBox {
    pub position: AtlasPosition, // this is relative to the tile
    pub size: AtlasSize,
    pub index: usize,
}

#[derive(Debug, Default, Clone)]
pub struct AtlasTile {
    pub size: AtlasSize,
    pub boxes: Vec<AtlasBox>,
    pub textures: Vec<AtlasTextureBox>,
    pub full: bool,
}

impl AtlasTile {
    fn new(size: AtlasSize) -> Self {
        return AtlasTile {
            size,
            ..Default::default()
        };
    }

    fn insert(&mut self, texture: &AtlasTextureBox) -> bool {
        if self.full {
            return false;
        }
        if self.boxes.len() == 0 {
            let b1 = AtlasBox {
                position: AtlasPosition {
                    x: texture.size.width,
                    y: 0,
                },
                size: AtlasSize {
                    width: self.size.width - texture.size.width,
                    height: self.size.height,
                },
            };
            let b2 = AtlasBox {
                position: AtlasPosition {
                    x: 0,
                    y: texture.size.height,
                },
                size: AtlasSize {
                    width: texture.size.width,
                    height: self.size.height - texture.size.height,
                },
            };
            let new_area = texture.size.area() + b1.size.area() + b2.size.area();

            self.boxes.push(b1);
            self.boxes.push(b2);
            let mut t = texture.clone();
            t.position.x = 0;
            t.position.y = 0;
            self.textures.push(t);
            return true;
        } else {
            let mut remove_index: usize = 0;
            let mut found = false;
            let mut boxes: Option<(AtlasBox, AtlasBox)> = None;
            for (i, abox) in self.boxes.iter_mut().enumerate() {
                match abox.insert(texture) {
                    Some(a) => {
                        boxes = Some(a);
                        remove_index = i;
                        self.textures.push(AtlasTextureBox {
                            position: abox.position.clone(),
                            size: texture.size.clone(),
                            index: texture.index.clone(),
                        });
                        break;
                    }
                    None => {}
                }
            }
            match boxes {
                Some((a, b)) => {
                    let _ = self.boxes.remove(remove_index);
                    self.boxes.push(a);
                    self.boxes.push(b);
                    self.boxes = self
                        .boxes
                        .clone()
                        .into_iter()
                        .filter(|i| i.size.area() > 0)
                        .collect();
                    self.boxes.sort();
                    if self.boxes.len() == 0 {
                        self.full = true;
                    }

                    return true;
                }
                None => return false,
            }
        }
    }
}

#[derive(Debug)]
pub struct UV {
    pub u: f32,
    pub v: f32,
}

#[derive(Debug)]
pub struct AtlasUV {
    pub min: UV,
    pub max: UV,
}

impl AtlasUV {
    pub fn min_max_as_array(&self) -> [f32; 4] {
        [self.min.u, self.min.v, self.max.u, self.max.v]
    }
    pub fn scale_uv(&self, uv: [f32; 2]) -> [f32; 2] {
        [
            self.min.u + (self.max.u - self.min.u) * uv[0],
            self.min.v + (self.max.v - self.min.v) * uv[1],
        ]
    }
}

impl Index<usize> for UV {
    type Output = f32;
    fn index(&self, index: usize) -> &f32 {
        match index {
            0 => &self.u,
            1 => &self.v,
            n => panic!("Invalid UV index: {}", n),
        }
    }
}

impl UV {
    fn as_array(self) -> [f32; 2] {
        [self.u, self.v]
    }
}

#[derive(Debug)]
pub struct AtlasTextureParsed {
    pub original_index: usize,
    pub name: String,
    pub position: AtlasPosition,
    pub size: AtlasSize,
    pub uv: AtlasUV,
}

#[derive(Debug)]
pub struct AtlasTexture {
    pub data: Vec<u8>,
    pub textures: Vec<AtlasTextureParsed>,
    pub map: HashMap<String, usize>,
    pub size: AtlasSize,
}

#[derive(Debug)]
pub struct Atlas {
    pub tile_size: AtlasSize,
    pub size: AtlasSize,
    pub tiles: Vec<AtlasTile>,
    pub debug_boxes: bool,
    pub mip_level: usize,
}

static DEBUGCOLORS: &[u8; 10] = &[128, 108, 160, 176, 192, 208, 224, 84, 100, 108];

impl Atlas {
    pub fn new(width: u32, height: u32) -> Self {
        let tile_size = AtlasSize { width, height };
        let t = AtlasTile::new(tile_size.clone());
        Self {
            tile_size,
            tiles: vec![t],
            size: AtlasSize {
                ..Default::default()
            },
            debug_boxes: false,
            mip_level: 0,
        }
    }

    pub fn insert_textures(&mut self, textures: Vec<AtlasTextureBox>) {
        for (it, texture) in textures.into_iter().enumerate() {
            let mut insert_new_tile = true;
            for tile in &mut self.tiles {
                if tile.insert(&texture) {
                    insert_new_tile = false;
                    break;
                }
            }
            if insert_new_tile {
                let mut tile = AtlasTile::new(self.tile_size.clone());
                tile.insert(&texture);
                self.tiles.push(tile);
            }
        }
    }

    pub fn generate_texture(&mut self, textures: &Vec<TextureParsed>) -> Result<AtlasTexture> {
        let mut atlas_texture = AtlasTexture {
            data: vec![],
            textures: vec![],
            map: HashMap::new(),
            size: AtlasSize {
                width: 0,
                height: 0,
            },
        };

        let tile_count = self.tiles.len();
        self.size = AtlasSize {
            width: self.tile_size.width * self.tiles.len() as u32,
            height: self.tile_size.height,
        };
        let mut data: Vec<u8> = Vec::with_capacity((self.size.area()) as usize);
        for _ in 0..self.size.area() {
            data.push(0);
        }

        let stride = self.size.width;
        let mut i = 0;
        for x in 0..tile_count as u32 {
            for y in 0..1 {
                if i >= self.tiles.len() as u32 {
                    break;
                }
                let tile = &self.tiles[i as usize];
                let tile_position = AtlasPosition {
                    x: x * self.tile_size.width,
                    y: y * self.tile_size.height,
                };
                for (c, texture) in tile.textures.iter().enumerate() {
                    let mut gd = Vec::with_capacity(texture.size.area() as usize);
                    for _ in 0..texture.size.area() {
                        gd.push(c as u8);
                    }
                    let real_texture = &textures[texture.index].mip_levels[self.mip_level];
                    let ft = TextureMip {
                        width: real_texture.width,
                        height: real_texture.height,
                        data: gd,
                    };

                    // let real_texture = ft;
                    let texture_pos = texture.position.clone() + tile_position.clone();
                    insert_texture(
                        &mut data, //&ft,
                        &real_texture,
                        stride,
                        texture_pos.x,
                        texture_pos.y,
                    )?;
                    let rt = &textures[texture.index];
                    let min_u = texture_pos.x as f32 / self.size.width as f32;
                    let min_v = texture_pos.y as f32 / self.size.height as f32;

                    let max_u =
                        (texture_pos.x as f32 + texture.size.width as f32) / self.size.width as f32;
                    let max_v = (texture_pos.y as f32 + texture.size.height as f32)
                        / self.size.height as f32;

                    let uv = AtlasUV {
                        min: UV { u: min_u, v: min_v },
                        max: UV { u: max_u, v: max_v },
                    };
                    let new_texture = AtlasTextureParsed {
                        original_index: texture.index,
                        name: rt.name.clone(),
                        position: texture_pos,
                        size: AtlasSize {
                            width: real_texture.width,
                            height: real_texture.height,
                        },
                        uv,
                    };
                    let pos = atlas_texture.textures.len();
                    atlas_texture.textures.push(new_texture);
                    let _ = atlas_texture.map.insert(rt.name.clone(), pos);
                }

                if self.debug_boxes {
                    for (color_count, tile_box) in tile.boxes.iter().enumerate() {
                        let box_size = tile_box.size.clone();
                        let c = DEBUGCOLORS[color_count % 10];

                        let mut gd = Vec::with_capacity((box_size.area()) as usize);
                        for _ in 0..box_size.area() {
                            gd.push(c as u8);
                        }
                        let ft = TextureMip {
                            width: box_size.width,
                            height: box_size.height,
                            data: gd,
                        };

                        let box_position = tile_box.position.clone() + tile_position.clone();
                        insert_texture(&mut data, &ft, stride, box_position.x, box_position.y)?;
                    }
                }
                i = i + 1;
            }
        }
        atlas_texture.data = data;
        atlas_texture.size = self.size.clone();
        atlas_texture.textures.sort_by(|a, b| {
            if a.original_index < b.original_index {
                return Ordering::Less;
            } else if a.original_index < b.original_index {
                return Ordering::Greater;
            }
            Ordering::Equal
        });
        Ok(atlas_texture)
    }
}
