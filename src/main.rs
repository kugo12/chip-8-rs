use std::fs::File;
use std::io;
use std::io::prelude::*;
use rand::random;


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

fn get_pattern(pattern: u16, val: u16) -> u8 {
    let val = val & pattern;
    let out = match pattern & 0x00ff {
        0x0000 => val >> 8,
        0x00f0 => val >> 4,
        _ => val
    };
    out as u8
}

#[derive(Debug)]
enum OPCode {
    CLS,                // clear screen
    RET,                // return subroutine
    JMP(u16),           // jump to NNN
    JMPV(u16),          // jump to NNN + V0
    CALL(u16),          // call subroutine at NNN
    SE(u8, u8),         // if equal skip next instruction
    SNE(u8, u8),        // if not equal skip next instruction
    LD(*mut u8, u8),    // set Vx to Vy
    LDI(u16),           // set I register to NNN
    LDRI(u8),           // read registers from V0 to Vx starting from I reg mem address
    LDIR(u8),           // store registers from V0 to Vx starting from I reg mem address
    LDK(u8),            // wait for input, store input value at Vx
    LDF(u8),            // set I to memory location of font letter Vx
    LDBCD(u8),          // store Vx BCD representation at I, I+1, I+2
    ADD(u8, u8),        // Vx = Vx + Vy
    ADDB(u8, u8),       // Vx = Vx + kk
    ADDI(u8),           // I = I + Vx
    SUBN(u8, u8),       // if Vy > Vx: VF = 1 else: VF = 0. Vx = Vy - Vx 
    SUB(u8, u8),        // if Vx > Vy: VF = 1 else: VF = 0. Vx = Vx - Vy
    OR(u8, u8),         // Vx |= Vy
    AND(u8, u8),        // Vx &= Vy
    XOR(u8, u8),        // Vx ^= Vy
    SHR(u8),            // set VF to Vx LSb, Vx >>= 1
    SHL(u8),            // set VF to Vx MSb, Vx <<= 1
    RND(u8, u8),        // Vx = random u8 & kk
    DRW(u8, u8, u8),    // draw sprite from I reg address, pos (Vx, Vy), n -> how long in bytes is sprite
    SKP(u8),            // if Vx key value is pressed skip next instruction
    SKNP(u8),           // if Vx key value is not pressed skip next instruction

    UNKNOWN(u16)
}

impl OPCode {
    fn run(instr: u16, chip8: &mut Chip8) -> bool {
        let op = OPCode::detect(instr, chip8);
        
        match op {
            OPCode::LD(p, v) => {
                unsafe{ p.write(v) }
            },
            OPCode::LDI(addr) => {
                chip8.i_reg = addr;
            },
            OPCode::UNKNOWN(inst) => {
                println!("Unknown instruction ({:x})", inst); 
                return false;
            },
            OPCode::RET => {
                chip8.pc = chip8.stack[chip8.sp as usize];
                chip8.sp -= 1;
            },
            OPCode::JMP(addr) => {
                chip8.pc = addr;
            },
            OPCode::JMPV(addr) => {
                chip8.pc = addr + (chip8.reg[0] as u16);
            },
            OPCode::CALL(addr) => {
                chip8.sp += 1;
                chip8.stack[chip8.sp as usize] = chip8.pc;
                chip8.pc = addr;
            },
            OPCode::SE(x, y) => {
                if x == y {
                    chip8.pc += 2;
                }
            },
            OPCode::SNE(x, y) => {
                if x != y {
                    chip8.pc += 2;
                }
            },
            OPCode::ADDB(vx, byte) => {
                chip8.reg[vx as usize] += byte;
            },
            OPCode::OR(vx, vy) => {
                chip8.reg[vx as usize] |= chip8.reg[vy as usize];
            },
            OPCode::AND(vx, vy) => {
                chip8.reg[vx as usize] &= chip8.reg[vy as usize];
            },
            OPCode::XOR(vx, vy) => {
                chip8.reg[vx as usize] ^= chip8.reg[vy as usize];
            },
            OPCode::ADD(vx, vy) => {
                let (val, carry) = chip8.reg[vx as usize].overflowing_add(chip8.reg[vy as usize]);
                chip8.reg[vx as usize] = val;
                chip8.reg[0xF] = carry as u8;
            },
            OPCode::SUB(vx, vy) => {
                let (val, borrow) = chip8.reg[vx as usize].overflowing_sub(chip8.reg[vy as usize]);
                chip8.reg[vx as usize] = val;
                chip8.reg[0xF] = borrow as u8;
            },
            OPCode::SUBN(vx, vy) => {
                let (val, borrow) = chip8.reg[vy as usize].overflowing_sub(chip8.reg[vx as usize]);
                chip8.reg[vx as usize] = val;
                chip8.reg[0xF] = borrow as u8;
            },
            OPCode::SHR(vx) => {
                chip8.reg[0xF] = chip8.reg[vx as usize] & 0b00000001;
                chip8.reg[vx as usize] >>= 1;
            },
            OPCode::SHL(vx) => {
                chip8.reg[0xF] = (chip8.reg[vx as usize] & 0b10000000) >> 7;
                chip8.reg[vx as usize] <<= 1;
            },
            OPCode::ADDI(vx) => {
                chip8.i_reg += chip8.reg[vx as usize] as u16;
            },
            OPCode::LDF(ch) => {
                chip8.i_reg = 5 * ch as u16;
            },
            OPCode::LDBCD(vx) => {
                let mut vx = chip8.reg[vx as usize].clone();
                let mut index: u16 = 3;
                while index > 0 {
                    index -= 1;
                    chip8.mem[(chip8.i_reg + index) as usize] = vx % 10;
                    vx /= 10;
                }
            },
            OPCode::LDIR(vx) => {
                let mut index: u8 = 0x0;

                while index < vx + 1 {
                    chip8.mem[(chip8.i_reg + index as u16) as usize] = chip8.reg[index as usize];
                    index += 1;
                }
                chip8.i_reg += vx as u16 + 1;
            },
            OPCode::LDRI(vx) => {
                let mut index: u8 = 0x0;

                while index < vx + 1 {
                    chip8.reg[index as usize] = chip8.mem[(chip8.i_reg + index as u16) as usize];
                    index += 1;
                }
                chip8.i_reg += vx as u16 + 1;
            },
            OPCode::SKP(vx) => {
                if chip8.input[chip8.reg[vx as usize] as usize] {
                    chip8.pc += 2;
                }
            },
            OPCode::SKNP(vx) => {
                if !chip8.input[chip8.reg[vx as usize] as usize] {
                    chip8.pc += 2;
                }
            },
            OPCode::RND(vx, byte) => {
                chip8.reg[vx as usize] = random::<u8>() & byte;
            },
            OPCode::CLS => {
                chip8.screen = [0; 32*64];
            },
            OPCode::DRW(vx, vy, n) => {
                let mut pos: u16 = {
                    let vx = chip8.reg[vx as usize] as u16;
                    let vy = chip8.reg[vy as usize] as u16;
                    // print!("{0}, {1}", vx, vy);
                    vx + vy*64
                };
                // print!(", {}\n", pos);

                let mut sprite: u8;
                let mut screen_slice: &mut [u8];
                let mut index: u16 = 0;
                let mut erased = false;
                while (index as u8) < n + 1 {
                    sprite = chip8.mem[(chip8.i_reg + index) as usize].clone().reverse_bits();

                    screen_slice = &mut chip8.screen[pos as usize .. (pos + 8) as usize];
                    for i in screen_slice.iter_mut() {
                        if *i + (sprite % 2) == 2 { erased = true; }
                        *i ^= sprite % 2;
                        sprite /= 2;
                    }

                    index += 1;
                    pos += 64;
                }

                chip8.reg[0xF] = erased as u8;
                
            }
            _ => println!("{:?} not implemented yet", op),
        };
        true
    }

    fn detect(inst: u16, chip8: &mut Chip8) -> OPCode {
        match inst & 0xF000 {
            0x0000 => {
                match inst {
                    0x00e0 => OPCode::CLS,
                    0x00ee => OPCode::RET,
                    _ => OPCode::JMP(inst & 0x0fff)
                }
            },
            0x1000 => OPCode::JMP(inst & 0x0fff),
            0x2000 => OPCode::CALL(inst & 0x0fff),
            0x3000 => OPCode::SE(chip8.reg[get_pattern(0x0f00, inst) as usize], get_pattern(0x00ff, inst)),
            0x4000 => OPCode::SNE(chip8.reg[get_pattern(0x0f00, inst) as usize], get_pattern(0x00ff, inst)),
            0x5000 => OPCode::SE(chip8.reg[get_pattern(0x0f00, inst) as usize], chip8.reg[get_pattern(0x00f0, inst) as usize]),
            0x6000 => OPCode::LD(&mut chip8.reg[get_pattern(0x0f00, inst) as usize], get_pattern(0x00ff, inst)),
            0x7000 => OPCode::ADDB(get_pattern(0x0f00, inst), get_pattern(0x00ff, inst)),
            0x8000 => {
                match inst & 0x000f {
                    0x0000 => OPCode::LD(&mut chip8.reg[get_pattern(0x0f00, inst) as usize], chip8.reg[get_pattern(0x00f0, inst) as usize]),
                    0x0001 => OPCode::OR(get_pattern(0x0f00, inst), get_pattern(0x00f0, inst)),
                    0x0002 => OPCode::AND(get_pattern(0x0f00, inst), get_pattern(0x00f0, inst)),
                    0x0003 => OPCode::XOR(get_pattern(0x0f00, inst), get_pattern(0x00f0, inst)),
                    0x0004 => OPCode::ADD(get_pattern(0x0f00, inst), get_pattern(0x00f0, inst)),
                    0x0005 => OPCode::SUB(get_pattern(0x0f00, inst), get_pattern(0x00f0, inst)),
                    0x0006 => OPCode::SHR(get_pattern(0x0f00, inst)),
                    0x0007 => OPCode::SUBN(get_pattern(0x0f00, inst), get_pattern(0x00f0, inst)),
                    0x000e => OPCode::SHL(get_pattern(0x0f00, inst)),
                    _ => OPCode::UNKNOWN(inst)
                }
            },
            0x9000 => OPCode::SNE(chip8.reg[get_pattern(0x0f00, inst) as usize], chip8.reg[get_pattern(0x00f0, inst) as usize]),
            0xA000 => OPCode::LDI(inst & 0x0fff),
            0xB000 => OPCode::JMPV(inst & 0x0fff),
            0xC000 => OPCode::RND(get_pattern(0x0f00, inst), get_pattern(0x00ff, inst)),
            0xD000 => OPCode::DRW(get_pattern(0x0f00, inst), get_pattern(0x00f0, inst), get_pattern(0x000f, inst)),
            0xE000 => {
                match inst & 0x00ff {
                    0x009e => OPCode::SKP(chip8.reg[get_pattern(0x0f00, inst) as usize]),
                    0x00a1 => OPCode::SKNP(chip8.reg[get_pattern(0x0f00, inst) as usize]),
                    _ => OPCode::UNKNOWN(inst)
                }
            },
            0xF000 => {
                match inst & 0x00ff {
                    0x0007 => OPCode::LD(&mut chip8.reg[get_pattern(0x0f00, inst) as usize], chip8.dt),
                    0x000a => OPCode::LDK(get_pattern(0x0f00, inst)),
                    0x0015 => OPCode::LD(&mut chip8.dt, chip8.reg[get_pattern(0x0f00, inst) as usize]),
                    0x0018 => OPCode::LD(&mut chip8.st, chip8.reg[get_pattern(0x0f00, inst) as usize]),
                    0x001e => OPCode::ADDI(get_pattern(0x0f00, inst)),
                    0x0029 => OPCode::LDF(get_pattern(0x0f00, inst)),
                    0x0033 => OPCode::LDBCD(get_pattern(0x0f00, inst)),
                    0x0055 => OPCode::LDIR(get_pattern(0x0f00, inst)),
                    0x0065 => OPCode::LDRI(get_pattern(0x0f00, inst)),
                    _ => OPCode::UNKNOWN(inst)
                }
            },
            _ => OPCode::UNKNOWN(inst)
        }
    }
}

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
    screen: [u8; 32*64]
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
            input: [false; 16],
            screen: [0; 32*64]
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

    fn get_instruction(&mut self) -> Option<u16> {
        let instruction = ((self.mem[self.pc as usize] as u16) << 8) | self.mem[(self.pc+1) as usize] as u16;
        if instruction == 0x0000 {
            None
        } else {
            self.pc += 2;
            Some(instruction)
        }
    }

    fn tick(&mut self) -> bool {
        match self.get_instruction() {
            Some(inst) => OPCode::run(inst, self),
            None => false
        }
    }
}


fn main() -> io::Result<()> {
    let mut c = Chip8::new();
    c.load_file_to_mem(&"pong.rom")?;

    while c.tick(){
        
    }
    Ok(())
}
