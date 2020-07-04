const MEM_SIZE: usize = 4096;
const GFX_MEM_SIZE: usize = 64*32;
const STACK_SIZE: usize = 16;

pub struct Chip8 {
    mem: Memory,
    regs: Registers,
    gfx: Graphics,
    stack: Stack,
    keyboard: Keyboard,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 {
            mem: Memory::new(),
            regs: Registers::new(),
            gfx: Graphics::new(),
            stack: Stack::new(),
            keyboard: Keyboard::new(),
        }
    }
}

struct Memory([u8; MEM_SIZE]);
impl Memory {
    fn new() -> Self {
        Self {
            0: [0; MEM_SIZE],
        }
    }
}

struct Registers {
    v: [u8; 16],
    i: u16,
    pc: u16,
    delay_timer: u8,
    sound_timer: u8,
}
impl Registers {
    fn new() -> Self {
        Self {
            v: [0; 16],
            i: 0,
            pc: 0,
            delay_timer: 0,
            sound_timer: 0,
        }
    }
}

struct Graphics([bool; GFX_MEM_SIZE]);
impl Graphics {
    fn new() -> Self {
        Self {
            0: [false; GFX_MEM_SIZE],
        }
    }
}

struct Stack {
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

struct Keyboard([bool; 16]);
impl Keyboard {
    fn new() -> Self {
        Self {
            0: [false; 16]
        }
    }
}