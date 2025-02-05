use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PaletteError {
    #[error("parse error: {0}")]
    ParseError(String),
}

#[derive(Serialize, Debug, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Serialize, Debug, Default)]
pub struct Palette {
    pub colors: Vec<Color>,
}

impl Palette {
    pub fn from(lookup_table: impl Into<Vec<u8>>) -> Result<Palette, PaletteError> {
        let lookup_table = lookup_table.into();
        if lookup_table.len() > 256 * 3 {
            return Err(PaletteError::ParseError(format!(
                "expected palette of size ({}) got ({})",
                256 * 3,
                lookup_table.len()
            )));
        }
        let mut p: Palette = Palette {
            ..Default::default()
        };

        let table: Vec<_> = lookup_table.chunks(3).collect();

        for c in table {
            p.colors.push(Color {
                r: c[0],
                g: c[1],
                b: c[2],
                a: u8::MAX,
            })
        }
        Ok(p)
    }

    pub fn apply(&self, convert: &Vec<u8>, output: &mut Vec<u8>) -> Result<(), PaletteError> {
        for c in convert {
            let i = *c as usize;
            output.push(self.colors[i].r);
            output.push(self.colors[i].g);
            output.push(self.colors[i].b);
        }
        Ok(())
    }
}
