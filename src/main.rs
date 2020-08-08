use std::fs::File;
use std::io;
use std::io::prelude::*;


const FONTS: [u8; 80] = [
    0xf0, 0x90, 0x90, 0x90, 0xf0,  // 0
    0x20, 0x60, 0x20, 0x20, 0x70,  // 1
    0xf0, 0x10, 0xf0, 0x80, 0xf0,  // 2
    0xf0, 0x10, 0xf0, 0x10, 0xf0,  // 3
    0x90, 0x90, 0xf0, 0x10, 0x10,  // 4
    0xf0, 0x80, 0xf0, 0x10, 0xf0,  // 5
    0xf0, 0x80, 0xf0, 0x90, 0xf0,  // 6
    0xf0, 0x10, 0x20, 0x40, 0x40,  // 7
    0xf0, 0x90, 0xf0, 0x90, 0xf0,  // 8
    0xf0, 0x90, 0xf0, 0x10, 0xf0,  // 9
    0xf0, 0x90, 0xf0, 0x90, 0x90,  // A
    0xe0, 0x90, 0xe0, 0x90, 0xe0,  // B
    0xf0, 0x80, 0x80, 0x80, 0xf0,  // C
    0xe0, 0x90, 0x90, 0x90, 0xe0,  // D
    0xf0, 0x80, 0xf0, 0x80, 0xf0,  // E
    0xf0, 0x80, 0xf0, 0x80, 0x80   // F
];

struct Chip8 {
    mem: [u8; 4096],
    reg: [u8; 16],
    i_reg: u16,
    pc: u16,  // program counter
    sp: u8,  // stack pointer
    stack: [u16; 16],
    dt: u8,  // delay timer
    st: u8,  // sound timer
    input: [bool; 16],
}

impl Chip8 {
    fn new() -> Chip8 {
        let mut c = Chip8 {
            mem: [0; 4096],
            reg: [0; 16],
            i_reg: 0,
            dt: 0,
            st: 0,
            pc: 0x200,
            stack: [0; 16],
            sp: 0,
            input: [false; 16]
        };
        c.load_fonts_to_mem();
        c
    }

    fn load_fonts_to_mem(&mut self){
        for (i, byte) in FONTS.iter().enumerate(){
            self.mem[i] = *byte;
        }
    }

    fn load_file_to_mem(&mut self, path: &str) -> io::Result<()> {
        let mut f = File::open(path)?;
        let mut buf: [u8; 3584] = [0; 3584];

        f.read(&mut buf)?;
        
        for (i, byte) in buf.iter().enumerate(){
            self.mem[i + 0x200] = *byte;
        }
        Ok(())
    }

    fn get_instruction(&mut self) -> u16 {
        let instruction = ((self.mem[self.pc as usize] as u16) << 8) | self.mem[(self.pc+1) as usize] as u16;
        self.pc += 2;
        instruction
    }
}


fn main() {
    let mut c = Chip8::new();
}
