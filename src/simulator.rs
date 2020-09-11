use rand::random;

const MEM_SIZE: usize = 4096;
pub const GFX_WIDTH: usize = 64;
pub const GFX_HEIGHT: usize = 32;
const STACK_SIZE: usize = 16;
const FONT_SET: [u8; 16*5] = [
  0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
  0x20, 0x60, 0x20, 0x20, 0x70, // 1
  0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
  0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
  0x90, 0x90, 0xF0, 0x10, 0x10, // 4
  0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
  0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
  0xF0, 0x10, 0x20, 0x40, 0x40, // 7
  0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
  0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
  0xF0, 0x90, 0xF0, 0x90, 0x90, // A
  0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
  0xF0, 0x80, 0x80, 0x80, 0xF0, // C
  0xE0, 0x90, 0x90, 0x90, 0xE0, // D
  0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
  0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

#[allow(non_camel_case_types)]
type u4 = u8;

pub type Opcode = (u4, u4, u4, u4);

type Execute = fn(chip8: &mut Chip8, opcode: &Opcode);
pub struct Operation {
    execute: Execute,
    display: &'static str,
    name: &'static str,
}

macro_rules! op {
    ($name: ident, $display: literal, $execute: expr) => {
        const $name: Operation = Operation {
            name: stringify!($name),
            display: $display,
            execute: $execute,
        };
    }
}

op!(OP_OOEO, "CLEAR screen", |chip8: &mut Chip8, opcode: &Opcode| { 
    chip8.gfx.clear(); 
    chip8.regs.pc += 2; 
});

op!(OP_OOEE, "RETURN from subroutine", |chip8: &mut Chip8, opcode: &Opcode| { 
    chip8.regs.pc = chip8.stack.pop()
});

op!(OP_1nnn, "JUMP to nnn{bcd}", |chip8: &mut Chip8, opcode: &Opcode| { 
    chip8.regs.pc = merge_u16(0x0, opcode.1, opcode.2, opcode.3)
});

op!(OP_2nnn, "CALL abc{bcd}", |chip8: &mut Chip8, opcode: &Opcode| { 
    chip8.stack.push(chip8.regs.pc + 2);
    chip8.regs.pc = merge_u16(0x0, opcode.1, opcode.2, opcode.3)
});

op!(OP_3xkk, "SKIP if Vx{Vb} equal to kk{cd}", |chip8: &mut Chip8, opcode: &Opcode| {
    chip8.regs.pc += if chip8.regs[opcode.1] == merge(opcode.2, opcode.3) {4} else {2};
});

op!(OP_4xkk, "SKIP if Vx{Vb} not equal to kk{cd}", |chip8: &mut Chip8, opcode: &Opcode| {
    chip8.regs.pc += if chip8.regs[opcode.1] != merge(opcode.2, opcode.3) {4} else {2};
});

op!(OP_5xy0, "SKIP if Vx{Vb} equal to Vy{Vc}", |chip8: &mut Chip8, opcode: &Opcode| {
    chip8.regs.pc += if chip8.regs[opcode.1] == chip8.regs[opcode.2] {4} else {2};
});

op!(OP_6xkk, "SET Vx{Vb} to kk{cd}", |chip8: &mut Chip8, opcode: &Opcode| {
    chip8.regs[opcode.1] = merge(opcode.2, opcode.3);
    chip8.regs.pc += 2;
});

op!(OP_7xkk, "ADD kk{cd} to Vx{Vb} without carry", |chip8: &mut Chip8, opcode: &Opcode| {
    chip8.regs[opcode.1] = chip8.regs[opcode.1].wrapping_add(merge(opcode.2, opcode.3));
    chip8.regs.pc += 2;
});

op!(OP_8xy0, "MOVE Vy{Vc} to Vx{Vb}", |chip8: &mut Chip8, opcode: &Opcode| {
    chip8.regs[opcode.1] = chip8.regs[opcode.2]; 
    chip8.regs.pc += 2;
});

op!(OP_8xy1, "OR Vx{Vb} with Vy{Vc} and store to Vx", |chip8: &mut Chip8, opcode: &Opcode| {
    chip8.regs[opcode.1] |= chip8.regs[opcode.2]; 
    chip8.regs.pc += 2;
});

op!(OP_8xy2, "AND Vx{Vb} with Vy{Vc} and store to Vx", |chip8: &mut Chip8, opcode: &Opcode| {
    chip8.regs[opcode.1] &= chip8.regs[opcode.2]; 
    chip8.regs.pc += 2;
});

op!(OP_8xy3, "XOR Vx{Vb} with Vy{Vc} and store to Vx", |chip8: &mut Chip8, opcode: &Opcode| {
    chip8.regs[opcode.1] ^= chip8.regs[opcode.2]; 
    chip8.regs.pc += 2;
});

op!(OP_8xy4, "ADD Vy{Vc} to Vx{Vb} and store to Vx with carry", |chip8: &mut Chip8, opcode: &Opcode| {
    if chip8.regs[opcode.1] as u16 + chip8.regs[opcode.2] as u16 > std::u8::MAX as u16 {
        chip8.regs[0xF] = 1
    }
    else {
        chip8.regs[0xF] = 0
    }
    chip8.regs[opcode.1] = chip8.regs[opcode.1].wrapping_add(chip8.regs[opcode.2]);
    chip8.regs.pc += 2; 
});

op!(OP_8xy5, "SUB Vy{Vc} from Vx{Vb} and store to Vx with borrow", |chip8: &mut Chip8, opcode: &Opcode| {
    if (chip8.regs[opcode.1] as i16 - chip8.regs[opcode.2] as i16) < 0 {
        chip8.regs[0xF] = 0
    }
    else {
        chip8.regs[0xF] = 1
    }
    chip8.regs[opcode.1] = chip8.regs[opcode.1].wrapping_sub(chip8.regs[opcode.2]);
    chip8.regs.pc += 2; 
});

op!(OP_8xy6, "SHIFT RIGHT Vx{Vb} and store LSB to VF", |chip8: &mut Chip8, opcode: &Opcode| {
    chip8.regs[0xF] = chip8.regs[opcode.1] & 1;
    chip8.regs[opcode.1] >>= 1;
    chip8.regs.pc += 2;
});

op!(OP_8xy7, "SUB Vx{Vc} from Vy{Vb} and store to Vx with borrow", |chip8: &mut Chip8, opcode: &Opcode| {
    if (chip8.regs[opcode.2] as i16 - chip8.regs[opcode.1] as i16) < 0 {
        chip8.regs[0xF] = 0
    }
    else {
        chip8.regs[0xF] = 1;
    }
    chip8.regs[opcode.1] = chip8.regs[opcode.2].wrapping_sub(chip8.regs[opcode.1]);
    chip8.regs.pc += 2; 
});

op!(OP_8xyE, "SHIFT LEFT Vx{Vb} and store MSB to VF", |chip8: &mut Chip8, opcode: &Opcode| {
    chip8.regs[0xF] = (chip8.regs[opcode.1] & (1<<7))>>7;
    chip8.regs[opcode.1] <<= 1;
    chip8.regs.pc += 2;
});

op!(OP_9xy0, "SKIP if Vx{Vb} != Vy{Vc}", |chip8: &mut Chip8, opcode: &Opcode| {
    chip8.regs.pc += if chip8.regs[opcode.1] != chip8.regs[opcode.2] {4} else {2};
});

op!(OP_Annn, "SET I to nnn{bcd}", |chip8: &mut Chip8, opcode: &Opcode| {
    chip8.regs.i = merge_u16(0, opcode.1, opcode.2, opcode.3); 
    chip8.regs.pc += 2;
});

op!(OP_Bnnn, "JUMP to nnn{bcd} + VO", |chip8: &mut Chip8, opcode: &Opcode| {
    chip8.regs.pc = merge_u16(0x0, opcode.1, opcode.2, opcode.3) + chip8.regs[0] as u16; 
    chip8.regs.pc += 2;
});

op!(OP_Cxkk, "RAND to Vx{Vb} and AND with kk{cd}", |chip8: &mut Chip8, opcode: &Opcode| {
    chip8.regs[opcode.1] = rand::random::<u8>() & merge(opcode.2, opcode.3); 
    chip8.regs.pc += 2;
});

op!(OP_Dxyn, "DRAW n{d} byte sprite at coordinates (Vx{Vb}, Vy{Vc})", |chip8: &mut Chip8, opcode: &Opcode| {
    chip8.regs[0xF] = 0;
    for byte in 0..opcode.3 {
        let y = ((chip8.regs[opcode.2] + byte) % 32) as usize;
        for bit in 0..8 {
            let x = ((chip8.regs[opcode.1]+bit) % 64) as usize;
            let value = ((chip8.mem[chip8.regs.i+byte as u16] >> (7-bit)) & 1) == 1;
            if chip8.gfx.0[x][y] && value {
                chip8.regs[0xF] = 1;
            }
            chip8.gfx.0[x][y] ^= value;
        }
    }
    chip8.gfx.draw();
    chip8.regs.pc += 2;
});

op!(OP_Ex9E, "SKIP if Vx{Vb} key is pressed", |chip8: &mut Chip8, opcode: &Opcode| {
    chip8.regs.pc += if chip8.keyboard.0[opcode.1 as usize] {4} else {2};
});

op!(OP_ExA1, "SKIP if Vx{Vb} key is NOT pressed", |chip8: &mut Chip8, opcode: &Opcode| {
    chip8.regs.pc += if !chip8.keyboard.0[chip8.regs[opcode.1] as usize] {4} else {2};
});

op!(OP_Fx07, "SET Vx{Vb} to delay_timer", |chip8: &mut Chip8, opcode: &Opcode| {
    chip8.regs[opcode.1] = chip8.regs.delay_timer; 
    chip8.regs.pc += 2;
});

op!(OP_Fx0A, "WAIT for keypress. Store key to Vx{Vb}", |chip8: &mut Chip8, opcode: &Opcode| {
    chip8.waiting_for_keyboard = true;
    chip8.keyboard_register = opcode.1; 
    chip8.regs.pc += 2;
});

op!(OP_Fx15, "SET delay_timer to Vx{Vb}", |chip8: &mut Chip8, opcode: &Opcode| {
    chip8.regs.delay_timer = chip8.regs[opcode.1]; 
    chip8.regs.pc += 2;
});

op!(OP_Fx18, "SET sound_timer to Vx{Vb}", |chip8: &mut Chip8, opcode: &Opcode| {
    chip8.regs.sound_timer = chip8.regs[opcode.1]; 
    chip8.regs.pc += 2;
});

op!(OP_Fx1E, "ADD Vx{Vb} to I and store in I", |chip8: &mut Chip8, opcode: &Opcode| {
    chip8.regs.i += chip8.regs[opcode.1] as u16; 
    chip8.regs.pc += 2;
});

op!(OP_Fx29, "SET I to location of sprite for digit stored in Vx{Vb}", |chip8: &mut Chip8, opcode: &Opcode| {
    chip8.regs.i = (chip8.regs[opcode.1] as u16)*5; 
    chip8.regs.pc += 2;
});

op!(OP_Fx33, "STORE BCD of Vx{Vb} in memory starting with address I", |chip8: &mut Chip8, opcode: &Opcode| {
    chip8.mem[chip8.regs.i] = chip8.regs[opcode.1] / 100;
    chip8.mem[chip8.regs.i + 1] = (chip8.regs[opcode.1] % 100) / 10;
    chip8.mem[chip8.regs.i + 2] = chip8.regs[opcode.1] % 10;
    chip8.regs.pc += 2;
});

op!(OP_Fx55, "STORE registers V0-Vx{Vb} to mem[I]-mem[I+x]", |chip8: &mut Chip8, opcode: &Opcode| {
    for i in 0..=opcode.1 {
        chip8.mem[chip8.regs.i + i as u16] = chip8.regs[i];
    }
    chip8.regs.pc += 2;
});

op!(OP_Fx65, "LOAD mem[I]-mem[I+x] to registers V0-Vx{Vb}", |chip8: &mut Chip8, opcode: &Opcode| {
    for i in 0..=opcode.1 {
        chip8.regs[i] = chip8.mem[chip8.regs.i + i as u16];
    }
    chip8.regs.pc += 2;
});

pub struct Chip8 {
    pub mem: Memory,
    pub regs: Registers,
    pub gfx: Graphics,
    pub stack: Stack,
    pub keyboard: Keyboard,
    pub waiting_for_keyboard: bool,
    pub keyboard_register: u4,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            mem: Memory::new(),
            regs: Registers::new(),
            gfx: Graphics::new(),
            stack: Stack::new(),
            keyboard: Keyboard::new(),
            waiting_for_keyboard: false,
            keyboard_register: 0,
        }
    }

    pub fn initialize(&mut self) {
        self.mem.load_fontset();
    }
    
    pub fn load_program(&mut self, file: String) {
        self.mem.load_program(file)
    }

    pub fn cycle(&mut self) {
        let opcode = self.fetch();
        let operation = self.decode(opcode);
        // self.print_operation(&operation, &opcode);
        (operation.execute)(self, &opcode);

        if self.regs.delay_timer > 0 {
            self.regs.delay_timer -= 1;
        }
        if self.regs.sound_timer > 0 {
            self.regs.sound_timer -= 1;
        }
    }

    fn print_operation(&mut self, operation: &Operation, opcode: &Opcode) {
        // let vb = format!("(V{:X} = {:X})", opcode.1, self.regs[opcode.1]);
        // let vc = format!("(V{:X} = {:X})", opcode.2, self.regs[opcode.2]);
        let output = operation.display.to_string(); //.replace("{Vb}", vb).replace({Vc}, vc)
        println!("{:X}: {}: {}", merge_u16(opcode.0, opcode.1, opcode.2, opcode.3), operation.name, operation.display)
    }

    fn fetch(&self) -> Opcode {
        let first = split(self.mem[self.regs.pc.into()]);
        let second = split(self.mem[(self.regs.pc+1).into()]);
        (first.0, first.1, second.0, second.1)
    }

    fn decode(&mut self, opcode: Opcode) -> Operation {
        match opcode {
            (0x0, 0x0, 0xE, 0x0) => OP_OOEO,
            (0x0, 0x0, 0xE, 0xE) => OP_OOEE,
            (0x1, _, _, _) => OP_1nnn,
            (0x2, _, _, _) => OP_2nnn,
            (0x3, _, _, _) => OP_3xkk,
            (0x4, _, _, _) => OP_4xkk,
            (0x5, _, _, 0x0) => OP_5xy0,
            (0x6, _, _, _) => OP_6xkk,
            (0x7, _, _, _) => OP_7xkk,
            (0x8, _, _, 0x0) => OP_8xy0,
            (0x8, _, _, 0x1) => OP_8xy1,
            (0x8, _, _, 0x2) => OP_8xy2,
            (0x8, _, _, 0x3) => OP_8xy3,
            (0x8, _, _, 0x4) => OP_8xy4,
            (0x8, _, _, 0x5) => OP_8xy5,
            (0x8, _, _, 0x6) => OP_8xy6,
            (0x8, _, _, 0x7) => OP_8xy7,
            (0x8, _, _, 0xE) => OP_8xyE,
            (0x9, _, _, 0x0) => OP_9xy0,
            (0xA, _, _, _) => OP_Annn,
            (0xB, _, _, _) => OP_Bnnn,
            (0xC, _, _, _) => OP_Cxkk,
            (0xD, _, _, _) => OP_Dxyn,
            (0xE, _, 0x9, 0xE) => OP_Ex9E,
            (0xE, _, 0xA, 0x1) => OP_ExA1,
            (0xF, _, 0x0, 0x7) => OP_Fx07,
            (0xF, _, 0x0, 0xA) => OP_Fx0A,
            (0xF, _, 0x1, 0x5) => OP_Fx15,
            (0xF, _, 0x1, 0x8) => OP_Fx18,
            (0xF, _, 0x1, 0xE) => OP_Fx1E,
            (0xF, _, 0x2, 0x9) => OP_Fx29,
            (0xF, _, 0x3, 0x3) => OP_Fx33,
            (0xF, _, 0x5, 0x5) => OP_Fx55,
            (0xF, _, 0x6, 0x5) => OP_Fx65,
            _ => panic!("Unknown opcode: {:?}", opcode),
        }
    }
}


#[test] 
fn test_fetch() {
    let mut chip = Chip8::new();
    chip.mem[2] = 0b01111110;
    chip.mem[3] = 0b10000001;
    chip.regs.pc = 2;
    assert_eq!(chip.fetch(), (0b0111, 0b1110, 0b1000, 0b0001));
}

pub struct Memory([u8; MEM_SIZE]);
impl Memory {
    fn new() -> Self {
        Self {
            0: [0; MEM_SIZE],
        }
    }
    fn load_fontset(&mut self) {
        for i in 0..(5*16) {
            self.0[i] = FONT_SET[i]
        }
    }
    pub fn load_program(&mut self, file: String) {
        let data = &std::fs::read(file).unwrap();
        if data.len() > MEM_SIZE - 0x200 {
            panic!("Image to big")
        }
        for i in 0..data.len() {
            self.0[i + 0x200] = data[i];
        }
    }
}
impl std::ops::Index<u16> for Memory {
    type Output = u8;
    fn index(&self, index: u16) -> &u8 {
        &self.0[index as usize]
    }
}
impl std::ops::IndexMut<u16> for Memory {
    fn index_mut(&mut self, index: u16) -> &mut u8 {
        &mut self.0[index as usize]
    }
}

pub struct Registers {
    pub v: [u8; 16],
    pub i: u16,
    pub pc: u16,
    pub delay_timer: u8,
    pub sound_timer: u8,
}
impl Registers {
    fn new() -> Self {
        Self {
            v: [0; 16],
            i: 0,
            pc: 0x200,
            delay_timer: 0,
            sound_timer: 0,
        }
    }
}
impl std::ops::Index<u4> for Registers {
    type Output = u8;
    fn index(&self, index: u4) -> &u8 {
        &self.v[index as usize]
    }
}
impl std::ops::IndexMut<u4> for Registers {
    fn index_mut(&mut self, index: u4) -> &mut u8 {
        &mut self.v[index as usize]
    }
}


pub struct Graphics(pub [[bool; GFX_HEIGHT]; GFX_WIDTH]);
impl Graphics {
    fn new() -> Self {
        Self {
            0: [[false; GFX_HEIGHT]; GFX_WIDTH],
        }
    }

    pub fn clear(&mut self) {
        self.0 = [[false; GFX_HEIGHT]; GFX_WIDTH];
        self.draw();
    }

    pub fn draw(&self) {
    //     print!("\x1B[2J\x1B[1;1H");
    //     for _ in 0..GFX_WIDTH {
    //         print!("X");
    //     }
    //     println!("");
    //     for y in 0..GFX_HEIGHT {
    //         print!("X");
    //         for x in 0..GFX_WIDTH {
    //             print!("{}", if self.0[x][y] {"█"} else {" "});
    //         }
    //         println!("X");
    //     }
    //     for _ in 0..GFX_WIDTH {
    //         print!("X");
    //     }
    //     println!("");
    }
}

pub struct Stack {
    stack: [u16; STACK_SIZE],
    sp: u8,
}
impl Stack {
    fn new() -> Self {
        Self {
            stack: [0; STACK_SIZE],
            sp: 0,
        }
    }
}

impl Stack {
    pub fn push(&mut self, value: u16) {
        self.stack[self.sp as usize] = value;
        self.sp += 1;
    }
    pub fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }
}

#[test]
fn test_stack() {
    let mut stack = Stack::new();
    stack.push(22);
    stack.push(12);
    assert_eq!(12, stack.pop());
    assert_eq!(22, stack.pop());
}

pub struct Keyboard(pub [bool; 16]);
impl Keyboard {
    fn new() -> Self {
        Self {
            0: [false; 16]
        }
    }
}


pub fn split(a: u8) -> (u4, u4) {
    let b0 = a & ((1 << 4) - 1);
    let b1 = a >> 4;
    (b1, b0)
}

#[test]
fn test_split() {
    assert_eq!(split(0b01111110), (0b0111, 0b1110));
    assert_eq!(split(0b01010001), (0b0101, 0b0001));
}

pub fn merge(a: u4, b: u4) -> u8 {
    (a << 4) | b
}

#[test]
fn test_merge() {
    assert_eq!(merge(0b0101, 0b0001), 0b01010001);
    assert_eq!(merge(0b1101, 0b1001), 0b11011001);
}

pub fn merge_u16(a: u4, b: u4, c: u4, d: u4) -> u16 {
    ((a as u16) << 12) | ((b as u16) << 8) | ((c as u16) << 4) | d as u16
}

#[test]
fn test_merge_u16() {
    assert_eq!(merge_u16(0b0001, 0b0010, 0b0100, 0b1000), 0b0001_0010_0100_1000)
}
