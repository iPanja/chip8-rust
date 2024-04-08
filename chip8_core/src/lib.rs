use rand::random;

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

pub struct Emu {
    pc: u16,
    ram: [u8; RAM_SIZE],
    screen: [bool; SCREEN_WIDTH * SCREEN_HEIGHT], // bool to indicate either black or white pixel
    v_reg: [u8; NUM_REGS],
    i_reg: u16,
    sp: u16,
    stack: [u16; STACK_SIZE],
    keys: [bool; NUM_KEYS],
    dt: u8,
    st: u8,
}

impl Emu {
    // Constructor
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

        new_emu.ram[..FONTSET_SIZE].copy_from_slice(&FONTSET);

        new_emu
    }

    // Reset emulator to default
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

    // Game tick
    pub fn tick(&mut self) {
        // Fetch op
        let op = self.fetch();
        // Decode
        // Execute
        self.execute(op);
    }

    // Core loop
    fn fetch(&mut self) -> u16 {
        let higher_byte = self.ram[self.pc as usize] as u16;
        let lower_byte: u16 = self.ram[(self.pc + 1) as usize] as u16;
        let op = (higher_byte << 8) | lower_byte;
        self.pc += 2;

        op
    }

    fn execute(&mut self, op: u16) {
        let digit1 = (op & 0xF000) >> 12;
        let digit2 = (op & 0x0F00) >> 8;
        let digit3 = (op & 0x00F0) >> 4;
        let digit4: u16 = op & 0x000F;

        match (digit1, digit2, digit3, digit4) {
            (0, 0, 0, 0) => return, // NOP - do nothing
            (0, 0, 0xE, 0) => self.screen = [false; SCREEN_WIDTH * SCREEN_HEIGHT], // Clear screen
            (0, 0, 0xE, 0xE) => {
                let ret_addr = self.pop();
                self.pc = ret_addr;
            } // Return from subroutine
            (1, _, _, _) => {
                // 1NNN
                let nnn = op & 0xFFF;
                self.pc = nnn;
            } // Jump to addres 0xNNN
            (2, _, _, _) => {
                // 2NNN
                self.push(self.pc);
                self.pc = op & 0xFFF;
            } // Call subroutine at 0xNNN
            (3, _, _, _) => {
                // 3XNN
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] == nn {
                    self.pc += 2;
                }
            } // Skip instruction if VX == 0xNN
            (4, _, _, _) => {
                // 4XNN
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                if self.v_reg[x] != nn {
                    self.pc += 2;
                }
            } // Skip instruction if VX != 0xNN
            (5, _, _, 0) => {
                // 5XY0
                let x = digit2 as usize;
                let y = digit3 as usize;
                if self.v_reg[x] == self.v_reg[y] {
                    self.pc += 2;
                }
            } // Skip instruction if VX == VY
            (6, _, _, _) => {
                // 6XNN
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = nn;
            } // Set register VX = 0xNN
            (7, _, _, _) => {
                // 7XNN
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                self.v_reg[x] = self.v_reg[x].wrapping_add(nn); // Overflow safe addition method
            } // Increment register VX by 0xNN
            (8, _, _, 0) => {
                // 8XY0
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] = self.v_reg[y];
            } // Set register VX = register VY
            (8, _, _, 1) => {
                // 8XY1
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] |= self.v_reg[y]
            } // Set register VX |= VY (bit or)
            (8, _, _, 2) => {
                // 8XY2
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] &= self.v_reg[y]
            } // Set register VX &= VY (bit and)
            (8, _, _, 3) => {
                // 8XY3
                let x = digit2 as usize;
                let y = digit3 as usize;
                self.v_reg[x] ^= self.v_reg[y]
            } // Set register VX ^= VY (bit xor?)
            (8, _, _, 4) => {
                // 8XY4
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, carry) = self.v_reg[x].overflowing_add(self.v_reg[y]);
                let new_vf = if carry { 1 } else { 0 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf; // Special carry bit register
            } // Increment register VX by VY, handling overflow (store carry bit in VF (0xF))
            (8, _, _, 5) => {
                // 8XY5
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_reg[x].overflowing_sub(self.v_reg[y]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            }
            (8, _, _, 6) => {
                // 8XY6
                let x = digit2 as usize;
                let lsb = self.v_reg[x] & 1;
                self.v_reg[x] >>= 1;
                self.v_reg[0xF] = lsb;
            } // Right shift VX, store dropped bit into VF
            (8, _, _, 7) => {
                // 8XY7
                let x = digit2 as usize;
                let y = digit3 as usize;

                let (new_vx, borrow) = self.v_reg[y].overflowing_sub(self.v_reg[y]);
                let new_vf = if borrow { 0 } else { 1 };

                self.v_reg[x] = new_vx;
                self.v_reg[0xF] = new_vf;
            } // VX = VY - VX, clearing VF if borrow
            (8, _, _, 0xE) => {
                // 8XYE
                let x = digit2 as usize;
                let msb = (self.v_reg[x] >> 7) & 1;
                self.v_reg[x] <<= 1;
                self.v_reg[0xF] = msb;
            } // Left shift VX, store dropped bit into VF
            (9, _, _, 0) => {
                // 9XY0
                let x = digit2 as usize;
                let y = digit3 as usize;

                if self.v_reg[x] != self.v_reg[y] {
                    self.pc += 2;
                }
            } // Skip instruction if VX != VY
            (0xA, _, _, _) => {
                // ANNN
                let nnn = op & 0xFFF;
                self.i_reg = nnn;
            } // Set the I Register to 0xNNN
            (0xB, _, _, _) => {
                // BNNN
                let nnn = op & 0xFFF;
                self.pc = (self.v_reg[0] as u16) + nnn;
            } // Jump to V0 + 0xNNN
            (0xC, _, _, _) => {
                // CXNN
                let x = digit2 as usize;
                let nn = (op & 0xFF) as u8;
                let rng: u8 = random();

                self.v_reg[x] = rng & nn;
            } // VX = rand() & 0xNN
            (0xD, _, _, _) => {
                // Sprite is 0xN pixels tall
                // On/Off based on value in I

                //
                // Get (x, y) coordinate to draw the sprite at
                let x_coord = self.v_reg[digit2 as usize] as u16;
                let y_coord = self.v_reg[digit3 as usize] as u16;
                let num_rows = digit4; // How many rows heigh the sprite is

                let mut flipped = false; // Track if any pixels were flipped

                for y_line in 0..num_rows {
                    // Determine which memory address the current row's data is stored
                    let addr = self.i_reg + y_line as u16;
                    let pixels = self.ram[addr as usize];
                    // Iterate over columns in the row (u8)
                    for x_line in 0..8 {
                        // Fetch current pixel via bitmask
                        let pixel = pixels & (0b1000_0000) >> x_line;
                        // Flip pixel only if a 1
                        if pixel != 0 {
                            // Sprites may wrap around the screen (hence the modulo)
                            let x = (x_coord + x_line) as usize % SCREEN_WIDTH;
                            let y = (y_coord + y_line) as usize % SCREEN_HEIGHT;
                            // Convert into array index
                            let idx = x + (SCREEN_WIDTH * y);

                            flipped |= self.screen[idx];
                            self.screen[idx] ^= true;
                        }
                    }
                }

                // VF set if any pixels flipped
                self.v_reg[0xF] = if flipped { 1 } else { 0 };
            } // Draw sprite at (VX, VY)
            (0xE, _, 9, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];

                if self.keys[vx as usize] {
                    self.pc += 2;
                }
            } // Skip instruction if key is pressed (key VX)
            (0xE, _, 0xA, 1) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x];

                if !self.keys[vx as usize] {
                    self.pc += 2;
                }
            } // Skip instruction if key is NOT pressed (key VX)
            (0xF, _, 0, 7) => {
                let x = digit2 as usize;
                self.v_reg[x] = self.dt;
            } // Set register VX = Delay Timer
            (0xF, _, 0, 0xA) => {
                // Wait for key press
                let x = digit2 as usize;
                let mut pressed = false;
                for i in 0..self.keys.len() {
                    if self.keys[i] {
                        // Key is pressed
                        self.v_reg[x] = i as u8;
                        pressed = true;
                        break;
                    }
                }

                if !pressed {
                    self.pc -= 2; // Cause loop since we are blocking until a key is pressed
                }
            } // Wait for key press, stores index in register VX (blocking operation)
            (0xF, _, 1, 5) => {
                let x = digit2 as usize;
                self.dt = self.v_reg[x];
            } // Delay Timer = register VX
            (0xF, _, 1, 8) => {
                let x = digit2 as usize;
                self.st = self.v_reg[x];
            } // Sound Timer = register VX
            (0xF, _, 1, 0xE) => {
                let x = digit2 as usize;
                let vx = self.v_reg[x] as u16;
                self.i_reg = self.i_reg.wrapping_add(vx);
            } // Increment I Register by VX
            (0xF, _, 2, 9) => {
                let x = digit2 as usize;
                let c = self.v_reg[x] as u16;
                self.i_reg = c * 5 // Each character is 5 bytes long
            } // Set I Register to the address of the font character in VX
            (0xF, _, 3, 3) => {
                // TODO: Optimizie this function!
                // Example, convert 0x64 (d100) to 0x1, 0x0, 0x0 (100 in decimal but represented weirdly in hex)
                // VX stores an 8-bit number (u8), representing 0-255
                // => 3 bytes
                let x = digit2 as usize;
                let vx = self.v_reg[x] as f32;

                let hundreds = (vx / 100.0).floor() as u8;
                let tens = ((vx / 10.0) % 10.0).floor() as u8;
                let ones = (vx % 10.0) as u8;

                self.ram[self.i_reg as usize] = hundreds;
                self.ram[(self.i_reg + 1) as usize] = tens;
                self.ram[(self.i_reg + 2) as usize] = ones;
            } // Set I Register to the Binary-Coded Decimal of VX
            (0xF, _, 5, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;

                for idx in 0..=x {
                    self.ram[i + idx] = self.v_reg[idx];
                }
            } // Store V0 thru VX into RAM starting at where I Register points
            (0xF, _, 6, 5) => {
                let x = digit2 as usize;
                let i = self.i_reg as usize;

                for idx in 0..=x {
                    self.v_reg[idx] = self.ram[i + idx]
                }
            } // Fills V0 thru VX with RAM values starting at where I Register points
            (_, _, _, _) => unimplemented!("Unimplmenented opcode: {}", op),
        }
    }

    pub fn tick_timers(&mut self) {
        if self.dt > 0 {
            self.dt -= 1;
        }

        if self.st > 0 {
            if self.st == 1 {
                // BEEP
            }
            self.st -= 1;
        }
    }

    // Stack handling
    fn push(&mut self, val: u16) {
        self.stack[self.sp as usize] = val;
        self.sp += 1;
    }

    fn pop(&mut self) -> u16 {
        self.sp -= 1;
        self.stack[self.sp as usize]
    }

    // Exposing core to frontend
    // Expose display
    pub fn get_display(&self) -> &[bool] {
        &self.screen
    }

    // Expose pressing keys
    pub fn keypress(&mut self, idx: usize, pressed: bool) {
        self.keys[idx] = pressed;
    }

    // Expose loading game code into our RAM
    pub fn load(&mut self, data: &[u8]) {
        let start = START_ADDR as usize;
        let end = (START_ADDR as usize) + data.len();

        self.ram[start..end].copy_from_slice(data);
    }
}
