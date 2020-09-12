#![allow(overflowing_literals)]

use crate::emulator::memory::Memory;
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
const OVERPLANE_FLAG: i16 = 0x0080;
const END_FLAG: i16 = 0x0040;
const BG_MAP_BASE: i16 = 0x000f;

// Object attribute flags
const JX: i16 = 0x03ff;
const JLON: i16 = 0x8000;
const JRON: i16 = 0x4000;
const JP: i16 = 0x03ff;
const JY: i16 = 0x00ff;
const JHFLP: i16 = 0x2000;
const JVFLP: i16 = 0x1000;
const JCA: i16 = 0x07ff;

fn modulus(a: i16, b: i16) -> i16 {
    ((a % b) + b) % b
}

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
    pub fn draw_eye(&mut self, memory: &mut Memory, eye: Eye, buf_address: usize) -> Result<()> {
        // Clear both frames to BKCOL
        let bkcol = memory.read_halfword(BKCOL) & 0x03;
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
            let done = self.draw_world(memory, eye, world)?;
            if done {
                break;
            }
        }

        for (row_offset, row) in self.buffer.iter().enumerate() {
            for (column_offset, column) in row.iter().enumerate() {
                let address = buf_address + (column_offset * 64) + (row_offset * 2);
                memory.write_halfword(address, *column);
            }
        }

        Ok(())
    }

    fn draw_world(&mut self, memory: &Memory, eye: Eye, world: usize) -> Result<bool> {
        let world_address = WORLD_ATTRIBUTE_MEMORY + (world * 32);
        let header = memory.read_halfword(world_address);
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
        let bgm = (header & BGM) >> 12;
        if bgm == 3 {
            self.draw_object_world(memory, eye);
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

        let overplane = memory.read_halfword(world_address + 20);
        let background = Background::parse(header, overplane)?;

        let dest_x = memory.read_halfword(world_address + 2);
        let dest_parallax_x = memory.read_halfword(world_address + 4);
        let dest_y = memory.read_halfword(world_address + 6);
        let source_x = memory.read_halfword(world_address + 8);
        let source_parallax_x = memory.read_halfword(world_address + 10);
        let source_y = memory.read_halfword(world_address + 12);
        let width = memory.read_halfword(world_address + 14) + 1;
        let height = i16::max(memory.read_halfword(world_address + 16) + 1, 8);

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
                // figure out which cell in this background map is being read
                let bg_x = source_x + column;
                let bg_y = source_y + row;
                let cell_address = background.get_cell_address(bg_x, bg_y);

                // load that cell data
                let cell_data = memory.read_halfword(cell_address);
                let palette_index = (cell_data >> 14) & 0x0003;
                let flip_horizontally = (cell_data & 0x2000) != 0;
                let flip_vertically = (cell_data & 0x1000) != 0;
                let char_index = cell_data & 0x07ff;

                // figure out which pixel in the character we're rendering
                let mut char_x = modulus(bg_x, 8);
                let mut char_y = modulus(bg_y, 8);
                if flip_horizontally {
                    char_x = 7 - char_x;
                }
                if flip_vertically {
                    char_y = 7 - char_y;
                }
                let pixel = self.get_char_pixel(memory, char_index, char_x, char_y);
                if pixel == 0 {
                    continue;
                }
                let color = self.get_palette_color(memory, GPLT0, palette_index, pixel);

                self.draw_pixel(dest_x + column, dest_y + row, color);
            }
        }

        return Ok(false);
    }

    fn draw_object_world(&mut self, memory: &Memory, eye: Eye) {
        let end_register = SPT0 + (self.object_world * 2);
        let mut obj_index = memory.read_halfword(end_register) as usize & 0x03ff;

        let target_obj_index: usize;
        if self.object_world == 0 {
            self.object_world = 3;
            target_obj_index = 1023;
        } else {
            self.object_world -= 1;
            let start_register = SPT0 + (self.object_world * 2);
            target_obj_index = memory.read_halfword(start_register) as usize & 0x03ff;
        }

        while obj_index != target_obj_index {
            let obj_address = OBJECT_ATTRIBUTE_MEMORY + (obj_index * 8);
            self.draw_object(memory, eye, obj_address);

            obj_index = if obj_index == 0 { 1023 } else { obj_index - 1 };
        }
    }

    fn draw_object(&mut self, memory: &Memory, eye: Eye, obj_address: usize) {
        let visible = match eye {
            Eye::Left => (memory.read_halfword(obj_address + 2) & JLON) != 0,
            Eye::Right => (memory.read_halfword(obj_address + 2) & JRON) != 0,
        };
        if !visible {
            return;
        }

        let jx = memory.read_halfword(obj_address) & JX;
        let jp = (memory.read_halfword(obj_address + 2) & JP)
            .wrapping_shl(6)
            .wrapping_shr(6);
        // apply parallax to the x coordinate
        let jx = match eye {
            Eye::Left => jx - jp,
            Eye::Right => jx + jp,
        };

        let jy = memory.read_halfword(obj_address + 4) & JY;
        // JY is effectively the lower 8 bits of an i16, so figure out the sign from the range
        // if it's > 224, it's supposed to be negative
        let jy = if jy > 224 {
            jy.wrapping_shl(8).wrapping_shr(8)
        } else {
            jy
        };

        let jplts = (memory.read_halfword(obj_address + 6) >> 14) & 0x3;
        let flip_horizontal = (memory.read_halfword(obj_address + 6) & JHFLP) != 0;
        let flip_vertical = (memory.read_halfword(obj_address + 6) & JVFLP) != 0;
        let jca = memory.read_halfword(obj_address + 6) & JCA;

        for x in 0..8 {
            for y in 0..8 {
                let char_x = if flip_horizontal { 7 - x } else { x };
                let char_y = if flip_vertical { 7 - y } else { y };
                let pixel = self.get_char_pixel(memory, jca, char_x, char_y);
                if pixel == 0 {
                    continue;
                }

                let color = self.get_palette_color(memory, JPLT0, jplts, pixel);
                self.draw_pixel(jx + x, jy + y, color);
            }
        }
    }

    fn get_char_pixel(&self, memory: &Memory, index: i16, x: i16, y: i16) -> i16 {
        let char_row =
            memory.read_halfword(CHARACTER_TABLE + (index as usize * 16) + (y as usize * 2));
        let pixel = (char_row >> (x * 2)) & 0x3;
        pixel
    }

    fn get_palette_color(&self, memory: &Memory, base: usize, index: i16, pixel: i16) -> i16 {
        let palette = memory.read_halfword(base + (index as usize * 2));
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

struct Background {
    pub mode: i16,
    pub bgmap_width: i16,
    pub bgmap_height: i16,
    pub bgmap_base: usize,
    pub overplane_cell: Option<usize>,
}
impl Background {
    pub fn parse(header: i16, overplane: i16) -> Result<Background> {
        let bgm = (header & BGM) >> 12;
        if bgm != 0 {
            // TODO: support other modes
            return Err(anyhow::anyhow!("Unsupported BGM {}!", bgm));
        }
        let scx = (header & SCX) >> 10;
        let scy = (header & SCY) >> 8;
        let bgmap_width = 2i16.pow(scx as u32);
        let bgmap_height = 2i16.pow(scy as u32);
        if bgmap_width != 1 || bgmap_height != 1 {
            // TODO: support multiple background maps
            return Err(anyhow::anyhow!(
                "Too many background maps ({}x{})!",
                bgmap_width,
                bgmap_height
            ));
        }
        let bgmap_base = (header & BG_MAP_BASE) as usize;
        let has_overplane = (header & OVERPLANE_FLAG) != 0;
        let overplane_cell = if has_overplane {
            Some(overplane as usize)
        } else {
            None
        };

        Ok(Background {
            mode: bgm,
            bgmap_width,
            bgmap_height,
            bgmap_base,
            overplane_cell,
        })
    }

    pub fn get_cell_address(&self, x: i16, y: i16) -> usize {
        // for now, assume everything is background map 0
        let bg_width = self.bgmap_width * 512;
        let bg_height = self.bgmap_height * 512;

        let mut bg_x = x;
        if bg_x < 0 || bg_x >= bg_width {
            bg_x = match self.overplane_cell {
                Some(index) => return BACKGROUND_MAP_MEMORY + (index * 2),
                None => modulus(bg_x, bg_width),
            };
        }
        let mut bg_y = y;
        if bg_y < 0 || bg_y >= bg_height {
            bg_y = match self.overplane_cell {
                Some(index) => return BACKGROUND_MAP_MEMORY + (index * 2),
                None => modulus(bg_y, bg_height),
            };
        }

        let row = bg_y as usize / 8;
        let column = bg_x as usize / 8;
        let index = row * 64 + column;
        BACKGROUND_MAP_MEMORY + (self.bgmap_base * 8192) + (index * 2)
    }
}
