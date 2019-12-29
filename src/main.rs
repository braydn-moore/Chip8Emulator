use std::{env, thread};
use std::time::Duration;
use crate::drivers::rom_driver::RomDriver;
use crate::drivers::audio_driver::AudioDriver;
use crate::drivers::screen_driver::ScreenDriver;
use crate::drivers::input_driver::InputDriver;
use crate::cpu::CPU;
use std::process::exit;

pub mod drivers;
pub mod cpu;


pub fn main(){
    // set a sleep time between ticks so we don't absolutely blow through our program
    let sleep_duration = Duration::from_millis(2);

    // make our SDL handle
    let sdl_context = sdl2::init().unwrap();

    // search the provided arguments for the ROM to load
    let args: Vec<String> = env::args().collect();
    if args.len() < 2{
        eprintln!("Error, please provide a file to run as an argument");
        exit(1);
    }
    let rom_file = &args[1];

    // initialize our drivers and CPU
    let rom_driver = RomDriver::new(rom_file);
    let mut audio_driver = AudioDriver::new(&sdl_context);
    let mut screen_driver = ScreenDriver::new(&sdl_context);
    let mut input_driver = InputDriver::new(&sdl_context);
    let mut cpu = CPU::new();
    cpu.load(&rom_driver.rom);

    // main game loop
    while let Ok(keypad) = input_driver.poll() {
        // make a CPU cycle and update the required drivers based on the CPU
        let output = cpu.tick(keypad);
        if output.display_updated {
            screen_driver.draw(output.display);
        }
        if output.play_sound {
            audio_driver.start();
        } else {
            audio_driver.stop();
        }
        // sleep a little bit between cycles so we don't run too quickly
        thread::sleep(sleep_duration);
    }
}