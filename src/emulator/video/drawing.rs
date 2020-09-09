use crate::emulator::storage::Storage;
use crate::emulator::video::Eye;
use anyhow::Result;

const BACKGROUND_MAP_MEMORY: usize = 0x00020000;
const WORLD_ATTRIBUTE_MEMORY: usize = 0x0003d800;
const CHARACTER_TABLE: usize = 0x00078000;
const GPLT0: usize = 0x0005f860;

const BKCOL: usize = 0x0005f870;

// World attribute flags
const LON: u16 = 0x8000;
const RON: u16 = 0x4000;
const BGM: u16 = 0x3000;
const SCX: u16 = 0x0c00;
const SCY: u16 = 0x0300;
const END_FLAG: u16 = 0x0040;
const BG_MAP_BASE: u16 = 0x000f;

pub struct DrawingProcess {
    buffer: [[i16; 384]; 28],
}

impl DrawingProcess {
    pub fn new() -> DrawingProcess {
        DrawingProcess {
            buffer: [[0; 384]; 28],
        }
    }

    // Draws the contents of the given eye to the screen
    pub fn draw_eye(&mut self, storage: &mut Storage, eye: Eye, buf_address: usize) -> Result<()> {
        // Clear both frames to BKCOL
        let bkcol = storage.read_halfword(BKCOL) & 0x03;
        let fill = (0..16)
            .step_by(2)
            .map(|shift| bkcol << shift)
            .fold(0, |a, b| a | b);
        for row in self.buffer.iter_mut() {
            for column in row.iter_mut() {
                *column = fill;
            }
        }

        // Draw rows in the buffer one-at-a-time
        for world in (0..32).rev() {
            let done = self.draw_world(storage, eye, world)?;
            if done {
                break;
            }
        }

        for (row_offset, row) in self.buffer.iter().enumerate() {
            for (column_offset, column) in row.iter().enumerate() {
                let address = buf_address + (column_offset * 64) + (row_offset * 2);
                storage.write_halfword(address, *column);
            }
        }

        Ok(())
    }

    fn draw_world(&mut self, storage: &Storage, eye: Eye, world: usize) -> Result<bool> {
        let world_address = WORLD_ATTRIBUTE_MEMORY + (world * 32);
        let header = storage.read_halfword(world_address) as u16;
        if (header & END_FLAG) != 0 {
            // END flag set, we're done rendering
            return Ok(true);
        }
        let left_enabled = (header & LON) != 0;
        let right_enabled = (header & RON) != 0;
        if !left_enabled && !right_enabled {
            // This world is blank, move on to the next
            return Ok(false);
        }
        let eye_enabled = match eye {
            Eye::Left => left_enabled,
            Eye::Right => right_enabled,
        };
        if !eye_enabled {
            // This eye is blank
            return Ok(false);
        }

        let bgm = (header & BGM) >> 12;
        if bgm == 3 {
            self.draw_object_world(storage, eye);
            return Ok(false);
        }
        if bgm != 0 {
            // TODO: support other modes
            return Err(anyhow::anyhow!("Unsupported BGM {}!", bgm));
        }

        let scx = (header & SCX) >> 10;
        let scy = (header & SCY) >> 8;
        let bgmap_width = 2u16.pow(scx as u32);
        let bgmap_height = 2u16.pow(scy as u32);
        if bgmap_width != 1 || bgmap_height != 1 {
            // TODO: support multiple background maps
            return Err(anyhow::anyhow!(
                "Too many background maps ({}x{})!",
                bgmap_width,
                bgmap_height
            ));
        }

        let bgmap = (header & BG_MAP_BASE) as usize;
        let bgmap_address = BACKGROUND_MAP_MEMORY + (bgmap * 8192);

        let dest_x = storage.read_halfword(world_address + 2);
        let dest_parallax_x = storage.read_halfword(world_address + 4);
        let dest_y = storage.read_halfword(world_address + 6);
        let source_x = storage.read_halfword(world_address + 8);
        let source_parallax_x = storage.read_halfword(world_address + 10);
        let source_y = storage.read_halfword(world_address + 12);
        let width = storage.read_halfword(world_address + 14) + 1;
        let height = i16::max(storage.read_halfword(world_address + 16) + 1, 8);

        // Apply parallax based on which eye this is
        let dest_x = match eye {
            Eye::Left => dest_x - dest_parallax_x,
            Eye::Right => dest_x + dest_parallax_x,
        };
        let source_x = match eye {
            Eye::Left => source_x - source_parallax_x,
            Eye::Right => source_x + source_parallax_x,
        };

        for column in 0..width {
            for row in 0..height {
                // for now, assume everything is background map 0

                // figure out which cell in this background map is being read
                let cell_row = (source_y + row) as usize / 8;
                let cell_column = (source_x + column) as usize / 8;
                let cell_index = cell_row * 64 + cell_column;

                // load that cell data
                let cell_data = storage.read_halfword(bgmap_address + (cell_index * 2)) as usize;
                let palette_index = (cell_data & 0xc000) >> 14;
                let flip_horizontally = (cell_data & 0x2000) != 0;
                let flip_vertically = (cell_data & 0x1000) != 0;
                let character_index = cell_data & 0x03ff;

                // figure out which pixel in the character we're rendering
                let mut character_x = source_x + column % 8;
                let mut character_y = (source_y + row) as usize % 8;
                if flip_horizontally {
                    character_x = 7 - character_x;
                }
                if flip_vertically {
                    character_y = 7 - character_y;
                }

                // get the value of that pixel
                let character_row = storage
                    .read_halfword(CHARACTER_TABLE + (character_index * 16) + (character_y * 2));
                let character_pixel = (character_row >> (character_x * 2)) & 0x03;
                if character_pixel == 0 {
                    continue;
                }

                // translate that through the palette
                let palette = storage.read_halfword(GPLT0 + (palette_index * 2));
                let pixel_value = (palette >> (character_pixel * 2)) & 0x03;

                // we finally have the pixel, now just write it to the right slot in the buffer
                // every 2 bytes contain 8 pixels, make sure we only update the 2 bits we're drawing now
                let column_index = (dest_x + column) as usize;
                let row_index = (dest_y + row) as usize / 8;
                let current_value = &mut self.buffer[row_index][column_index];
                let row_offset = (dest_y + row) % 8;
                *current_value &= !(0b11 << (row_offset * 2));
                *current_value |= pixel_value << (row_offset * 2);
            }
        }

        return Ok(false);
    }

    fn draw_object_world(&self, _storage: &Storage, _eye: Eye) {
        // TODO
    }
}
