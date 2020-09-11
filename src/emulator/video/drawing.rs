#![allow(overflowing_literals)]

use crate::emulator::storage::Storage;
use crate::emulator::video::Eye;
use anyhow::Result;

const BACKGROUND_MAP_MEMORY: usize = 0x00020000;
const WORLD_ATTRIBUTE_MEMORY: usize = 0x0003d800;
const OBJECT_ATTRIBUTE_MEMORY: usize = 0x0003e000;
const CHARACTER_TABLE: usize = 0x00078000;

const SPT0: usize = 0x0005f848;

const GPLT0: usize = 0x0005f860;

const JPLT0: usize = 0x0005f868;

const BKCOL: usize = 0x0005f870;

// World attribute flags
const LON: i16 = 0x8000;
const RON: i16 = 0x4000;
const BGM: i16 = 0x3000;
const SCX: i16 = 0x0c00;
const SCY: i16 = 0x0300;
const END_FLAG: i16 = 0x0040;
const BG_MAP_BASE: i16 = 0x000f;

// Object attribute flags
const JX: i16 = 0x01ff;
const JLON: i16 = 0x8000;
const JRON: i16 = 0x4000;
const JP: i16 = 0x001f;
const JY: i16 = 0x00ff;
const JHFLP: i16 = 0x2000;
const JVFLP: i16 = 0x1000;
const JCA: i16 = 0x07ff;

pub struct DrawingProcess {
    buffer: [[i16; 384]; 28],
    object_world: usize,
}

impl DrawingProcess {
    pub fn new() -> DrawingProcess {
        DrawingProcess {
            buffer: [[0; 384]; 28],
            object_world: 3,
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
        self.object_world = 3;
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
        let header = storage.read_halfword(world_address);
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
        log::debug!("World {}: 0x{:04x}", world, header);
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
                let cell_data = storage.read_halfword(bgmap_address + (cell_index * 2));
                let palette_index = (cell_data >> 14) & 0x0003;
                let flip_horizontally = (cell_data & 0x2000) != 0;
                let flip_vertically = (cell_data & 0x1000) != 0;
                let char_index = cell_data & 0x03ff;

                // figure out which pixel in the character we're rendering
                let mut char_x = (source_x + column) % 8;
                let mut char_y = (source_y + row) % 8;
                if flip_horizontally {
                    char_x = 7 - char_x;
                }
                if flip_vertically {
                    char_y = 7 - char_y;
                }
                let pixel = self.get_char_pixel(storage, char_index, char_x, char_y);
                if pixel == 0 {
                    continue;
                }
                let color = self.get_palette_color(storage, GPLT0, palette_index, pixel);

                self.draw_pixel(dest_x + column, dest_y + row, color);
            }
        }

        return Ok(false);
    }

    fn draw_object_world(&mut self, storage: &Storage, eye: Eye) {
        let end_register = SPT0 + (self.object_world * 2);
        let mut obj_index = storage.read_halfword(end_register) as usize & 0x03ff;

        let target_obj_index: usize;
        if self.object_world == 0 {
            self.object_world = 3;
            target_obj_index = 1023;
        } else {
            self.object_world -= 1;
            let start_register = SPT0 + (self.object_world * 2);
            target_obj_index = storage.read_halfword(start_register) as usize & 0x03ff;
        }

        while obj_index != target_obj_index {
            let obj_address = OBJECT_ATTRIBUTE_MEMORY + (obj_index * 8);
            self.draw_object(storage, eye, obj_address);

            obj_index = if obj_index == 0 { 1023 } else { obj_index - 1 };
        }
    }

    fn draw_object(&mut self, storage: &Storage, eye: Eye, obj_address: usize) {
        let visible = match eye {
            Eye::Left => (storage.read_halfword(obj_address + 2) & JLON) != 0,
            Eye::Right => (storage.read_halfword(obj_address + 2) & JRON) != 0,
        };
        if !visible {
            return;
        }

        let jx = storage.read_halfword(obj_address) & JX;
        let jp = storage.read_halfword(obj_address + 2) & JP;
        // apply parallax to the x coordinate
        let jx = match eye {
            Eye::Left => jx - jp,
            Eye::Right => jx + jp,
        };

        let jy = storage.read_halfword(obj_address + 4) & JY;
        // JY is effectively the lower 8 bits of an i16, so figure out the sign from the range
        // if it's > 224, it's supposed to be negative
        let jy = if jy > 224 {
            jy.wrapping_shl(8).wrapping_shr(8)
        } else {
            jy
        };

        let jplts = (storage.read_halfword(obj_address + 6) >> 14) & 0x3;
        let flip_horizontal = (storage.read_halfword(obj_address + 6) & JHFLP) != 0;
        let flip_vertical = (storage.read_halfword(obj_address + 6) & JVFLP) != 0;
        let jca = storage.read_halfword(obj_address + 6) & JCA;

        for x in 0..8 {
            for y in 0..8 {
                let char_x = if flip_horizontal { 7 - x } else { x };
                let char_y = if flip_vertical { 7 - y } else { y };
                let pixel = self.get_char_pixel(storage, jca, char_x, char_y);
                if pixel == 0 {
                    continue;
                }

                let color = self.get_palette_color(storage, JPLT0, jplts, pixel);
                self.draw_pixel(jx + x, jy + y, color);
            }
        }
    }

    fn get_char_pixel(&self, storage: &Storage, index: i16, x: i16, y: i16) -> i16 {
        let char_row =
            storage.read_halfword(CHARACTER_TABLE + (index as usize * 16) + (y as usize * 2));
        let pixel = (char_row >> (x * 2)) & 0x3;
        pixel
    }

    fn get_palette_color(&self, storage: &Storage, base: usize, index: i16, pixel: i16) -> i16 {
        let palette = storage.read_halfword(base + (index as usize * 2));
        let color = (palette >> (pixel * 2)) & 0x03;
        color
    }

    fn draw_pixel(&mut self, column: i16, row: i16, color: i16) {
        if column < 0 || row < 0 || column >= 384 || row >= 224 {
            return;
        }
        let row_index = row as usize / 8;
        let current_value = &mut self.buffer[row_index][column as usize];
        let row_offset = row % 8;
        *current_value &= !(0b11 << (row_offset * 2));
        *current_value |= color << (row_offset * 2);
    }
}
