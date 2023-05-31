pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

const RAM_SIZE: usize = 4096;
const NUM_REGS: usize = 16;
const STACK_SIZE: usize = 16;
const NUM_KEYS: usize = 16;
const START_ADDR: u16 = 0x200;
const FONTSET_SIZE: usize = 80;
const FONTSET: [u8; FONTSET_SIZE] = [
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
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

// delay timer and sound timer
pub struct Emu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT],
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    dt: u8,
    st: u8,
}

impl Emu {
    pub fn new() -> Self {
        let mut new_emu = Self {
            pc: START_ADDR,
            ram: [0; RAM_SIZE],
            screen: [false; SCREEN_WIDTH * SCREEN_HEIGHT],
            v_reg: [0; NUM_REGS],
            i_reg: 0,
            sp: 0,
            stack: [0; STACK_SIZE],
            keys: [false; NUM_KEYS],
            dt: 0,
            st: 0,
        };

        new_emu[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_emu
    }

    pub fn reset(&mut self) {
        self.pc = START_ADDR;
        self.ram = [0; RAM_SIZE];
        self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
        self.v_reg = [0; NUM_REGS];
        self.i_reg = 0;
        self.sp = 0;
        self.stack = [0; STACK_SIZE];
        self.keys = [false; NUM_KEYS];
        self.dt = 0;
        self.st = 0;
        self.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);
    }

    pub fn tick(&mut self) {
        let op = self.fetch();
        self.execute(op);
    }

    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte = self.ram[(self.pc + 1) as usize] as u16;
        // each operation is two bytes so we move the higher byte to the left and then bitwise or
        // the lower byte
        let op: u16 = (higher_byte << 8) | lower_byte;
        self.pc += 2;

        op
    }

    fn execute(op: u16) {
        // splits the two bytes in to nibbles or hex digits
        // e.g. for the first digit 1101 1010 1100 1110 & 1111 0000 0000 0000 = 1101
        // you then move it to the front of the two bytes
        let nibble1 = (op & 0xF000) >> 12;
        let nibble2 = (op & 0x0F00) >> 8;
        let nibble3 = (op & 0x00F0) >> 4;
        let nibble4 = (op & 0x000F);

        match (nibble1, nibble2, nibble3, nibble4) {
            // NOP
            (0, 0, 0, 0) => return,
            // clear screen
            (0, 0, 0xE, 0) => {
                self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT];
            }
            // return from subroutine
            (0, 0, 0xE, 0) => {
                let addr = self.pop();
                self.pc = addr;
            }
            // jump to address
            (1, _, _, _) => {
                self.pc = 0xFFF & self.op;
            }
            // call subroutine
            (2, _, _, _) => {
                self.push(self.pc);
                self.pc = 0xFFF & op;
            }
            // skip next if VX == NN
            (3, _, _, _) => {
                let x = nibble2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            }
            // skip next if VX != NN
            (4, _, _, _) => {
                let x = nibble2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            }
            // skip next if VX == VY
            (5, _, _, _) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            }
            // VX = NN
            (6, _, _, _) => {
                let x = nibble2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = nn; 
            }
            // VX += NN
            (7, _, _, _) => {
                let x = nibble2 as usize;
                let nn - (op & 0xFF) as u8;
                self.v_reg[x] = self.v_reg[x].wrapping_add(n);
            }
            // VX = VY 
            (8, _, _, 0) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;
                self.v_reg[x] = self.v_reg[y];
            } 
            // VX |= VY
            (8, _, _, 1) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;
                self.v_reg[x] |= self.v_reg[y];
            }
            // VX &= VY
            (8, _, _, 2) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;
                self.v_reg[x] &= self.v_reg[y];
            }
            // VX ^= VY
            (8, _, _, 3) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;
                self.v_reg[x] ^= self.v_reg[y];
            }
            // VX += VY (carrys)
            (8, _, _, 4) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;

                let (val, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                let flag = if carry {1} else {0};

                self.v_reg[x] = val;
                self.v_reg[0xF] = flag;
            }
            // VX -= VY (borrows)
            (8, _, _, 5) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;

                let (val, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                let flag = if borrow {0} else {1};

                self.v_reg[x] = val;
                self.v_reg[0xF] = flag;
            }
            // VX >>= 1
            (8, _, _, 6) => {
                let x = nibble2 as usize;
                let dropped = self.v_reg[x] & 1;
                self.v_reg[x] >>= 1;
                self.v_reg[0xF] = dropped;
            }
            // VX = VY - VX
            (8, _, _, 7) => {
                let x = nibble2 as usize;
                let y = nibble3 as usize;

                let (val, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[x]);
                let flag if borrow {0} else {1};

                self.v_reg[x] = val;
                self.v_reg[0xF] = borrow;
            },
            // VX <<= 1
            (8, _, _, 0xE) => {
                let x = nibble2 as usize;
                let missed = (self.v_reg[x] >> 7) & 1;
                self.v_reg[x] <<= 1;
                self.v_reg[0xF] = missed;
            }
            // skip if VX != VY
            (9, _, _, 0) => {
                let x = self.v_reg[nibble2 as usize];
                let y = self.v_reg[nibble3 as usize];
                if x != y {
                    self.pc += 2;
                }
            }
            // I = NNN
            (0xA, _, _, _) => {
                let nnn = op & 0xFFF;
                self.i_reg = nnn;
            }
            // jump to V0 + nnn
            (0xB, _, _, _) => {
                let nnn = op & 0xFFF;
                self.pc = (self.v_reg[0] as u16) + nnn;
            }

            (_, _, _, _) => unimplemented!("Unimplemented op code: {}", op),
        }
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }
        if self.st > 0 {
            if self.st == 1 {
                // make sound here
            }
            self.st -= 1;
        }
    }

    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }
}
