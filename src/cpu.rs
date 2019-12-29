use crate::drivers::screen_driver::{CHIP8_WIDTH, CHIP8_HEIGHT};
use rand::Rng;

const CHIP8_RAM: usize = 4096;
const OPCODE_SIZE: usize = 2;

// built in sprites in memory for the CHIP8 chip
pub const FONT: [u8; 80] = [
    0xF0,
    0x90,
    0x90,
    0x90,
    0xF0,
    0x20,
    0x60,
    0x20,
    0x20,
    0x70,
    0xF0,
    0x10,
    0xF0,
    0x80,
    0xF0,
    0xF0,
    0x10,
    0xF0,
    0x10,
    0xF0,
    0x90,
    0x90,
    0xF0,
    0x10,
    0x10,
    0xF0,
    0x80,
    0xF0,
    0x10,
    0xF0,
    0xF0,
    0x80,
    0xF0,
    0x90,
    0xF0,
    0xF0,
    0x10,
    0x20,
    0x40,
    0x40,
    0xF0,
    0x90,
    0xF0,
    0x90,
    0xF0,
    0xF0,
    0x90,
    0xF0,
    0x10,
    0xF0,
    0xF0,
    0x90,
    0xF0,
    0x90,
    0x90,
    0xE0,
    0x90,
    0xE0,
    0x90,
    0xE0,
    0xF0,
    0x80,
    0x80,
    0x80,
    0xF0,
    0xE0,
    0x90,
    0x90,
    0x90,
    0xE0,
    0xF0,
    0x80,
    0xF0,
    0x80,
    0xF0,
    0xF0,
    0x80,
    0xF0,
    0x80,
    0x80,
];

// return the CPU state after each tick to tell the main loop to play a sound or update the display
pub struct CpuState<'a>{
    pub display: &'a[[u8; CHIP8_WIDTH]; CHIP8_HEIGHT],
    pub display_updated: bool,
    pub play_sound: bool
}

// PC counter next state
enum PCState {
    Next,
    Skip,
    Jump(usize)
}

// used for skip if opcode calls
impl PCState {
    fn skip_if(condition: bool) -> PCState {
        if condition {
            PCState::Skip
        } else {
            PCState::Next
        }
    }
}

pub struct CPU {
    display: [[u8; CHIP8_WIDTH]; CHIP8_HEIGHT],
    display_update: bool,
    ram: [u8; CHIP8_RAM],
    stack: [usize; 16],
    v_registers: [u8; 16],
    i: usize,
    pc: usize,
    sp: usize,
    delay_timer: u8,
    sound_timer: u8,
    keypad: [bool; 16],
    keypad_waiting: bool,
    keypad_register: usize,
}

impl CPU{
    // initialize the RAM and copy the font values into the RAM
    pub fn new() -> CPU{
        let mut ram = [0u8; CHIP8_RAM];
        for i in 0..FONT.len() {
            ram[i] = FONT[i];
        }

        CPU {
            display: [[0; CHIP8_WIDTH]; CHIP8_HEIGHT],
            display_update: false,
            ram,
            stack: [0; 16],
            v_registers: [0; 16],
            i: 0,
            pc: 0x200,
            sp: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypad: [false; 16],
            keypad_waiting: false,
            keypad_register: 0,
        }
    }

    // load the ROM as a byte array into RAM
    pub fn load(&mut self, dat: &[u8]){
        let mut addr = 0x200;
        for (_i, &byte) in dat.iter().enumerate() {
            if addr < 4096 {
                self.ram[addr] = byte;
            } else {
                break;
            }
            addr+=1;
        }
    }

    // clear the screen
    fn clear_screen(&mut self) -> PCState {
        for y in 0..CHIP8_HEIGHT {
            for x in 0..CHIP8_WIDTH {
                self.display[y][x] = 0;
            }
        }
        self.display_update = true;
        PCState::Next
    }

    // The following functions are implementations of the equivalent opcodes needed for later
    fn ret(&mut self) -> PCState {
        self.sp -= 1;
        PCState::Jump(self.stack[self.sp])
    }

    fn jump(&mut self, nnn: usize) -> PCState {
        PCState::Jump(nnn)
    }

    fn call(&mut self, nnn: usize) -> PCState {
        self.stack[self.sp] = self.pc + OPCODE_SIZE;
        self.sp += 1;
        PCState::Jump(nnn)
    }

    fn skip_if_equal_register(&mut self, x: usize, kk: u8) -> PCState {
        PCState::skip_if(self.v_registers[x] == kk)
    }

    fn skip_if_not_equal_register(&mut self, x: usize, kk: u8) -> PCState {
        PCState::skip_if(self.v_registers[x] != kk)
    }

    fn skip_if_registers_equal(&mut self, x: usize, y: usize) -> PCState {
        PCState::skip_if(self.v_registers[x] == self.v_registers[y])
    }

    fn skip_if_registers_not_equal(&mut self, x: usize, y: usize) -> PCState {
        PCState::skip_if(self.v_registers[x] != self.v_registers[y])
    }

    fn skip_if_key_pressed(&mut self, x: usize) -> PCState {
        PCState::skip_if(self.keypad[self.v_registers[x] as usize])
    }

    fn skip_if_key_not_pressed(&mut self, x: usize) -> PCState {
        PCState::skip_if(!self.keypad[self.v_registers[x] as usize])
    }

    fn load_register(&mut self, x: usize, kk: u8) -> PCState {
        self.v_registers[x] = kk;
        PCState::Next
    }

    fn add_register(&mut self, x: usize, kk: u8) -> PCState {
        let vx = self.v_registers[x] as u16;
        let val = kk as u16;
        let result = vx + val;
        self.v_registers[x] = result as u8;
        PCState::Next
    }

    fn load_register_into_register(&mut self, x: usize, y: usize) -> PCState {
        self.v_registers[x] = self.v_registers[y];
        PCState::Next
    }

    fn or_register_by_register(&mut self, x: usize, y: usize) -> PCState {
        self.v_registers[x] |= self.v_registers[y];
        PCState::Next
    }

    fn and_register_by_register(&mut self, x: usize, y: usize) -> PCState {
        self.v_registers[x] &= self.v_registers[y];
        PCState::Next
    }

    fn xor_register_by_register(&mut self, x: usize, y:usize) -> PCState {
        self.v_registers[x] ^= self.v_registers[y];
        PCState::Next
    }

    fn add_registers_and_carry(&mut self, x: usize, y:usize) -> PCState {
        let vx = self.v_registers[x] as u16;
        let vy = self.v_registers[y] as u16;
        let result = vx + vy;
        self.v_registers[x] = result as u8;
        self.v_registers[0xF] = if result > 0xFF { 1 } else { 0 };
        PCState::Next
    }

    fn subtract_and_carry(&mut self, x: usize, y: usize) -> PCState {
        self.v_registers[0xF] = if self.v_registers[x] > self.v_registers[y] { 1 } else { 0 };
        self.v_registers[x] = self.v_registers[x].wrapping_sub(self.v_registers[y]);
        PCState::Next
    }

    fn shift_right(&mut self, x: usize) -> PCState {
        self.v_registers[0xF] = self.v_registers[x] & 1;
        self.v_registers[x] >>= 1;
        PCState::Next
    }

    fn shift_left(&mut self, x: usize) -> PCState {
        self.v_registers[0xF] = (self.v_registers[x] & 0b10000000) >> 7;
        self.v_registers[x] <<= 1;
        PCState::Next
    }

    fn subtract_and_carry_inverted(&mut self, x: usize, y: usize) -> PCState {
        self.v_registers[0x0f] = if self.v_registers[y] > self.v_registers[x] { 1 } else { 0 };
        self.v_registers[x] = self.v_registers[y].wrapping_sub(self.v_registers[x]);
        PCState::Next
    }

    fn load_i(&mut self, nnn: usize) -> PCState {
        self.i = nnn;
        PCState::Next
    }

    fn jump_with_addition(&mut self, nnn: usize) -> PCState {
        PCState::Jump(nnn + self.v_registers[0] as usize)
    }

    fn gen_rand(&mut self, x: usize, kk: u8) -> PCState {
        let mut rng = rand::thread_rng();
        self.v_registers[x] = rng.gen::<u8>() & kk;
        PCState::Next
    }

    fn draw_sprite(&mut self, x: usize, y: usize, n: usize) -> PCState {
        self.v_registers[0xF] = 0;
        for byte in 0..n {
            let y = (self.v_registers[y] as usize + byte) % CHIP8_HEIGHT;
            for bit in 0..8 {
                let x = (self.v_registers[x] as usize + bit) % CHIP8_WIDTH;
                let color = (self.ram[self.i + byte] >> (7 - bit) as u8) & 1;
                self.v_registers[0xF] |= color & self.display[y][x];
                self.display[y][x] ^= color;
            }
        }
        self.display_update = true;
        PCState::Next
    }

    fn load_delay_timer_to_register(&mut self, x: usize) -> PCState {
        self.v_registers[x] = self.delay_timer;
        PCState::Next
    }

    fn wait_for_keypress(&mut self, x: usize) -> PCState {
        self.keypad_waiting = true;
        self.keypad_register = x;
        PCState::Next
    }

    fn set_delay_timer(&mut self, x: usize) -> PCState {
        self.delay_timer = self.v_registers[x];
        PCState::Next
    }

    fn set_sound_timer(&mut self, x: usize) -> PCState {
        self.sound_timer = self.v_registers[x];
        PCState::Next
    }

    fn add_register_to_i(&mut self, x: usize) -> PCState {
        self.i += self.v_registers[x] as usize;
        self.v_registers[0xF] = if self.i > 0x0F00 { 1 } else { 0 };
        PCState::Next
    }

    fn set_i_to_sprite(&mut self, x: usize) -> PCState {
        self.i = (self.v_registers[x] as usize) * 5;
        PCState::Next
    }

    fn spread_decimal(&mut self, x: usize) -> PCState {
        self.ram[self.i] = self.v_registers[x] / 100;
        self.ram[self.i + 1] = (self.v_registers[x] % 100) / 10;
        self.ram[self.i + 2] = self.v_registers[x] % 10;
        PCState::Next
    }

    fn dump_registers_to_mem(&mut self, x: usize) -> PCState {
        for i in 0..x + 1 {
            self.ram[self.i + i] = self.v_registers[i];
        }
        PCState::Next
    }

    fn load_registers_from_mem(&mut self, x: usize) -> PCState{
        for i in 0..x + 1 {
            self.v_registers[i] = self.ram[self.i + i];
        }
        PCState::Next
    }

    // read the next opcode from the byte array
    fn get_opcode(&self) -> u16{
        (self.ram[self.pc] as u16) << 8 | (self.ram[self.pc + 1]) as u16
    }

    // run the opcode
    fn run_opcode(&mut self, opcode: u16){
        // split the opcode into each individual bytes
        let bits = (
            ((opcode & 0xF000) >> 12) as u8,
            ((opcode & 0x0F00) >> 8) as u8,
            ((opcode & 0x00F0) >> 4) as u8,
            (opcode & 0x000F) as u8,
        );

        // note variable names are used as defined by the CPU specification
        // found at http://www.cs.columbia.edu/~sedwards/classes/2016/4840-spring/designs/Chip8.pdf
        let nnn = (opcode & 0x0FFF) as usize;
        let kk = (opcode & 0x00FF) as u8;
        let x = bits.1 as usize;
        let y = bits.2 as usize;
        let n = bits.3 as usize;

        // map each opcode to the given function providing the required parameters as specified
        // by the CPU specs
        let pc_change = match bits {
            (0x0, 0x0, 0xE, 0x0) => self.clear_screen(),
            (0x0, 0x0, 0xE, 0xE) => self.ret(),
            (0x1, _, _, _) => self.jump(nnn),
            (0x2, _, _, _) => self.call(nnn),
            (0x3, _, _, _) => self.skip_if_equal_register(x, kk),
            (0x4, _, _, _) => self.skip_if_not_equal_register(x, kk),
            (0x5, _, _, 0x0) => self.skip_if_registers_equal(x, y),
            (0x6, _, _, _) => self.load_register(x, kk),
            (0x7, _, _, _) => self.add_register(x, kk),
            (0x8, _, _, 0x0) => self.load_register_into_register(x, y),
            (0x8, _, _, 0x1) => self.or_register_by_register(x, y),
            (0x8, _, _, 0x2) => self.and_register_by_register(x, y),
            (0x8, _, _, 0x3) => self.xor_register_by_register(x, y),
            (0x8, _, _, 0x4) => self.add_registers_and_carry(x, y),
            (0x8, _, _, 0x5) => self.subtract_and_carry(x, y),
            (0x8, _, _, 0x6) => self.shift_right(x),
            (0x8, _, _, 0x7) => self.subtract_and_carry_inverted(x, y),
            (0x8, _, _, 0xE) => self.shift_left(x),
            (0x9, _, _, 0x0) => self.skip_if_registers_not_equal(x, y),
            (0xA, _, _, _) => self.load_i(nnn),
            (0xB, _, _, _) => self.jump_with_addition(nnn),
            (0xC, _, _, _) => self.gen_rand(x, kk),
            (0xD, _, _, _) => self.draw_sprite(x, y, n),
            (0xE, _, 0x9, 0xE) => self.skip_if_key_pressed(x),
            (0xE, _, 0xA, 0x1) => self.skip_if_key_not_pressed(x),
            (0xF, _, 0x0, 0x7) => self.load_delay_timer_to_register(x),
            (0xF, _, 0x0, 0xA) => self.wait_for_keypress(x),
            (0xF, _, 0x1, 0x5) => self.set_delay_timer(x),
            (0xF, _, 0x1, 0x8) => self.set_sound_timer(x),
            (0xF, _, 0x1, 0xE) => self.add_register_to_i(x),
            (0xF, _, 0x2, 0x9) => self.set_i_to_sprite(x),
            (0xF, _, 0x3, 0x3) => self.spread_decimal(x),
            (0xF, _, 0x5, 0x5) => self.dump_registers_to_mem(x),
            (0xF, _, 0x6, 0x5) => self.load_registers_from_mem(x),
            _ => {eprintln!("Invalid opcode {:x}", opcode); PCState::Next}
        };

        // based on how the program counter should change modify the PC
        match pc_change {
            PCState::Next => self.pc += OPCODE_SIZE,
            PCState::Skip => self.pc += 2 * OPCODE_SIZE,
            PCState::Jump(addr) => self.pc = addr,
        }
    }

    // main function for a CPU "tick" or operation
    pub fn tick(&mut self, keyboard: [bool; 16]) -> CpuState{
        self.keypad = keyboard;
        self.display_update = false;
        // if we are waiting for a keypress then you know wait
        if self.keypad_waiting{
            for i in 0..16{
                if self.keypad[i]{
                    self.keypad_waiting = false;
                    self.v_registers[self.keypad_register] = i as u8;
                    break;
                }
            }
        }
        else{
            // decrement our counters if required
            if self.delay_timer > 0{
                self.delay_timer -= 1;
            }
            if self.sound_timer > 0{
                self.sound_timer -= 1;
            }
            // get our opcode and run it
            let opcode = self.get_opcode();
            self.run_opcode(opcode);
        }

        // return our CPU state to tell the main loop what updates are required
        CpuState {
            display: &self.display,
            display_updated: self.display_update,
            play_sound: self.sound_timer > 0,
        }
    }
}