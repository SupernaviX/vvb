use anyhow::Result;
use std::cell::RefCell;
use std::env;
use std::io::Read;
use vvb::emulator::memory::{Memory, Region};
use vvb::emulator::video::drawing::DrawingProcess;
use vvb::emulator::video::Eye;

fn get_filepath() -> Result<String> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        return Err(anyhow::anyhow!("Please pass a binary file to run"));
    }
    Ok(args[1].clone())
}

const ITERATIONS: u128 = 1000;

fn main() -> Result<()> {
    let mut file = std::fs::File::open(get_filepath()?)?;
    let mut buf = vec![];
    file.read_to_end(&mut buf)?;

    let memory = RefCell::new(Memory::new());
    if let Some(vram) = memory.borrow_mut().write_region(Region::Vram) {
        vram.copy_from_slice(&buf);
    }

    let mut xp = DrawingProcess::new();

    let left_buf_address = 0x00000000;
    let right_buf_address = 0x00008000;

    let start = std::time::Instant::now();

    for _ in 0..ITERATIONS {
        xp.start(memory.borrow());
        xp.draw_eye(&mut memory.borrow_mut(), Eye::Left, left_buf_address);
        xp.draw_eye(&mut memory.borrow_mut(), Eye::Right, right_buf_address);
    }

    let duration = start.elapsed().as_nanos();
    println!("On average, took {} ns", duration / ITERATIONS);
    Ok(())
}
