use crate::emulator::memory::Memory;
use crate::emulator::video::Eye;
use std::cell::RefMut;

const BACKGROUND_MAP_MEMORY: usize = 0x00020000;
const WORLD_ATTRIBUTE_MEMORY: usize = 0x0003d800;
const OBJECT_ATTRIBUTE_MEMORY: usize = 0x0003e000;
const CHARACTER_TABLE: usize = 0x00078000;

const SPT0: usize = 0x0005f848;

const GPLT0: usize = 0x0005f860;

const JPLT0: usize = 0x0005f868;

const BKCOL: usize = 0x0005f870;

// World attribute flags
const LON: u16 = 0x8000;
const RON: u16 = 0x4000;
const BGM: u16 = 0x3000;
const SCX: u16 = 0x0c00;
const SCY: u16 = 0x0300;
const OVERPLANE_FLAG: u16 = 0x0080;
const END_FLAG: u16 = 0x0040;
const BG_MAP_BASE: u16 = 0x000f;

// Object attribute flags
const JX: u16 = 0x03ff;
const JLON: u16 = 0x8000;
const JRON: u16 = 0x4000;
const JP: u16 = 0x03ff;
const JY: u16 = 0x00ff;
const JHFLP: u16 = 0x2000;
const JVFLP: u16 = 0x1000;
const JCA: u16 = 0x07ff;

fn modulus(a: i16, b: i16) -> u16 {
    if b.count_ones() == 1 {
        (a & (b - 1)) as u16
    } else {
        a.rem_euclid(b) as u16
    }
}

pub struct DrawingProcess {
    buffer: [[u16; 384]; 28],
    object_world: usize,

    last_char: u16,
    last_char_index: u16,
    last_char_row: u16,

    last_cell_address: usize,
    cell_palette_index: u16,
    cell_flip_horizontally: bool,
    cell_flip_vertically: bool,
    cell_char_index: u16,
}

impl DrawingProcess {
    pub fn new() -> DrawingProcess {
        DrawingProcess {
            buffer: [[0; 384]; 28],
            object_world: 3,
            last_char: 0,
            last_char_index: u16::MAX,
            last_char_row: u16::MAX,

            last_cell_address: usize::MAX,
            cell_palette_index: u16::MAX,
            cell_flip_horizontally: false,
            cell_flip_vertically: false,
            cell_char_index: u16::MAX,
        }
    }

    // Draws the contents of the given eye to the screen
    pub fn draw_eye(&mut self, memory: &mut RefMut<Memory>, eye: Eye, buf_address: usize) {
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

        self.last_char_index = u16::MAX;
        self.last_cell_address = usize::MAX;

        // Draw rows in the buffer one-at-a-time
        self.object_world = 3;
        for world in (0..32).rev() {
            let done = self.draw_world(&memory, eye, world);
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
    }

    fn draw_world(&mut self, memory: &RefMut<Memory>, eye: Eye, world: usize) -> bool {
        let world_address = WORLD_ATTRIBUTE_MEMORY + (world * 32);
        let header = memory.read_halfword(world_address);
        if (header & END_FLAG) != 0 {
            // END flag set, we're done rendering
            return true;
        }
        let left_enabled = (header & LON) != 0;
        let right_enabled = (header & RON) != 0;
        if !left_enabled && !right_enabled {
            // This world is blank, move on to the next
            return false;
        }
        let bgm = (header & BGM) >> 12;
        if bgm == 3 {
            self.draw_object_world(memory, eye);
            return false;
        }

        let eye_enabled = match eye {
            Eye::Left => left_enabled,
            Eye::Right => right_enabled,
        };
        if !eye_enabled {
            // This eye is blank
            return false;
        }

        let background = Background::parse(memory, world_address);

        let dest_x = memory.read_halfword(world_address + 2) as i16;
        let dest_parallax_x = memory.read_halfword(world_address + 4) as i16;
        let dest_y = memory.read_halfword(world_address + 6) as i16;
        let width = memory.read_halfword(world_address + 14) as i16 + 1;
        let height = i16::max(memory.read_halfword(world_address + 16) as i16 + 1, 8);

        // Apply parallax based on which eye this is
        let dest_x = match eye {
            Eye::Left => dest_x - dest_parallax_x,
            Eye::Right => dest_x + dest_parallax_x,
        };

        for row in 0..height {
            for column in 0..width {
                // figure out which cell in this background map is being read
                let (bg_x, bg_y) = background.get_coords(eye, column, row);
                let cell_address = background.get_cell_address(bg_x, bg_y);

                // load that cell data
                if self.last_cell_address != cell_address {
                    let cell_data = memory.read_halfword(cell_address);
                    self.last_cell_address = cell_address;
                    self.cell_palette_index = (cell_data >> 14) & 0x0003;
                    self.cell_flip_horizontally = (cell_data & 0x2000) != 0;
                    self.cell_flip_vertically = (cell_data & 0x1000) != 0;
                    self.cell_char_index = cell_data & 0x07ff;
                }

                // figure out which pixel in the character we're rendering
                let mut char_x = modulus(bg_x, 8);
                let mut char_y = modulus(bg_y, 8);
                if self.cell_flip_horizontally {
                    char_x = 7 - char_x;
                }
                if self.cell_flip_vertically {
                    char_y = 7 - char_y;
                }
                let pixel = self.get_char_pixel(memory, self.cell_char_index, char_x, char_y);
                if pixel == 0 {
                    continue;
                }
                let color = self.get_palette_color(memory, GPLT0, self.cell_palette_index, pixel);

                self.draw_pixel(dest_x + column, dest_y + row, color);
            }
        }

        return false;
    }

    fn draw_object_world(&mut self, memory: &RefMut<Memory>, eye: Eye) {
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

    fn draw_object(&mut self, memory: &RefMut<Memory>, eye: Eye, obj_address: usize) {
        let visible = match eye {
            Eye::Left => (memory.read_halfword(obj_address + 2) & JLON) != 0,
            Eye::Right => (memory.read_halfword(obj_address + 2) & JRON) != 0,
        };
        if !visible {
            return;
        }

        let jx = ((memory.read_halfword(obj_address) & JX) as i16)
            .wrapping_shl(6)
            .wrapping_shr(6);
        let jp = ((memory.read_halfword(obj_address + 2) & JP) as i16)
            .wrapping_shl(6)
            .wrapping_shr(6);
        // apply parallax to the x coordinate
        let jx = match eye {
            Eye::Left => jx - jp,
            Eye::Right => jx + jp,
        };

        let jy = (memory.read_halfword(obj_address + 4) & JY) as i16;
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

        for y in 0..8 {
            let char_y = if flip_vertical { 7 - y } else { y } as u16;
            for x in 0..8 {
                let char_x = if flip_horizontal { 7 - x } else { x } as u16;
                let pixel = self.get_char_pixel(memory, jca, char_x, char_y);
                if pixel == 0 {
                    continue;
                }

                let color = self.get_palette_color(memory, JPLT0, jplts, pixel);
                self.draw_pixel(jx + x, jy + y, color);
            }
        }
    }

    fn get_char_pixel(&mut self, memory: &RefMut<Memory>, index: u16, x: u16, y: u16) -> u16 {
        if self.last_char_index != index || self.last_char_row != y {
            let address = CHARACTER_TABLE + (index as usize * 16) + (y as usize * 2);
            self.last_char = memory.read_halfword(address);
            self.last_char_index = index;
            self.last_char_row = y;
        }
        let pixel = (self.last_char >> (x * 2)) & 0x3;
        pixel
    }

    fn get_palette_color(
        &self,
        memory: &RefMut<Memory>,
        base: usize,
        index: u16,
        pixel: u16,
    ) -> u16 {
        let palette = memory.read_halfword(base + (index as usize * 2));
        let color = (palette >> (pixel * 2)) & 0x03;
        color
    }

    fn draw_pixel(&mut self, column: i16, row: i16, color: u16) {
        if column < 0 || row < 0 || column >= 384 || row >= 224 {
            return;
        }
        let row_index = row as usize / 8;
        let current_value = &mut self.buffer[row_index][column as usize];
        let row_offset = row as u16 % 8;
        *current_value &= !(0b11 << (row_offset * 2));
        *current_value |= color << (row_offset * 2);
    }
}

enum BGMode {
    Normal,
    HBias,
    Affine,
}
struct Background<'a> {
    memory: &'a RefMut<'a, Memory>,
    pub mode: BGMode,
    pub bgmap_width: i16,
    pub bgmap_height: i16,
    pub bgmap_base: usize,
    pub overplane_cell: Option<usize>,

    pub src_x: i16,
    pub src_parallax_x: i16,
    pub src_y: i16,
    pub param_base: usize,
}
impl<'a> Background<'a> {
    pub fn parse(memory: &'a RefMut<'a, Memory>, address: usize) -> Background {
        let header = memory.read_halfword(address);
        let bgm = (header & BGM) >> 12;
        let mode = match bgm {
            2 => BGMode::Affine,
            1 => BGMode::HBias,
            _ => BGMode::Normal,
        };

        let scx = (header & SCX) >> 10;
        let scy = (header & SCY) >> 8;
        let bgmap_width = 2i16.pow(scx as u32);
        let bgmap_height = 2i16.pow(scy as u32);
        let bgmap_base = (header & BG_MAP_BASE) as usize;
        let has_overplane = (header & OVERPLANE_FLAG) != 0;
        let overplane_cell = if has_overplane {
            let overplane = memory.read_halfword(address + 20);
            Some(overplane as usize)
        } else {
            None
        };
        let param_base = BACKGROUND_MAP_MEMORY + (memory.read_halfword(address + 18) as usize) * 2;

        let src_x = memory.read_halfword(address + 8) as i16;
        let src_parallax_x = memory.read_halfword(address + 10) as i16;
        let src_y = memory.read_halfword(address + 12) as i16;

        Background {
            memory,
            mode,
            bgmap_width,
            bgmap_height,
            bgmap_base,
            overplane_cell,
            param_base,
            src_x,
            src_parallax_x,
            src_y,
        }
    }

    pub fn get_coords(&self, eye: Eye, x: i16, y: i16) -> (i16, i16) {
        match self.mode {
            BGMode::Normal => {
                let x = x + self.src_x;
                let y = y + self.src_y;
                match eye {
                    Eye::Left => (x - self.src_parallax_x, y),
                    Eye::Right => (x + self.src_parallax_x, y),
                }
            }
            BGMode::HBias => {
                let address = self.param_base + (y as usize * 4);
                let x = x + self.src_x;
                let y = y + self.src_y;
                match eye {
                    Eye::Left => {
                        let offset = self.memory.read_halfword(address) as i16;
                        (x - self.src_parallax_x + offset, y)
                    }
                    Eye::Right => {
                        let offset = self.memory.read_halfword(address + 2) as i16;
                        (x + self.src_parallax_x + offset, y)
                    }
                }
            }
            BGMode::Affine => {
                let address = self.param_base + (y as usize * 16);
                // These are stored as 13.3 fixed-point numbers,
                // shift left 6 to give precision 9
                let x_base = (self.memory.read_halfword(address) as i16 as i32) << 6;
                let y_base = (self.memory.read_halfword(address + 4) as i16 as i32) << 6;

                // These are stored as 7.9 signed fixed-point numbers
                let x_offset = self.memory.read_halfword(address + 6) as i16 as i32;
                let y_offset = self.memory.read_halfword(address + 8) as i16 as i32;

                // This is a 16-bit signed integer
                let parallax = self.memory.read_halfword(address + 2) as i16 as i32;

                // Parallax applies to only the left eye if negative, only the right if positive
                let mut px = x as i32;
                if parallax < 0 {
                    if let Eye::Left = eye {
                        px += parallax;
                    }
                } else {
                    if let Eye::Right = eye {
                        px += parallax;
                    }
                }

                let tx = (x_base + (x_offset * px)) >> 9;
                let ty = (y_base + (y_offset * px)) >> 9;

                (tx as i16, ty as i16)
            }
        }
    }

    pub fn get_cell_address(&self, x: i16, y: i16) -> usize {
        let bg_width = self.bgmap_width * 512;
        let bg_height = self.bgmap_height * 512;

        let bg_x = if 0 <= x && x < bg_width {
            x as usize
        } else {
            match self.overplane_cell {
                Some(index) => return BACKGROUND_MAP_MEMORY + (index * 2),
                None => modulus(x, bg_width) as usize,
            }
        };

        let bg_y = if 0 <= y && y < bg_height {
            y as usize
        } else {
            match self.overplane_cell {
                Some(index) => return BACKGROUND_MAP_MEMORY + (index * 2),
                None => modulus(y, bg_height) as usize,
            }
        };

        let map_x = bg_x / 512;
        let map_y = bg_y / 512;
        let map_index = self.bgmap_base + map_x + (map_y * self.bgmap_width as usize);

        let row = (bg_y % 512) / 8;
        let column = (bg_x % 512) / 8;
        let index = row * 64 + column;
        BACKGROUND_MAP_MEMORY + (map_index * 8192) + (index * 2)
    }
}
