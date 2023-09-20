extern crate sdl2;
use rand::prelude::*;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

const RESOLUTION_SCALE: u32 = 10;
const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const FONT_DATA: [u8; 80] = [
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
const BACKGROUND_COLOR: Color = Color::RGB(0, 0, 0);
const FOREGROUND_COLOR: Color = Color::RGB(0, 255, 0);

struct CPU {
    registers: [u8; 16],
    register_i: u16,
    program_counter: u16,
    vram: [[u8; WIDTH]; HEIGHT],
    ram: [u8; 0x1000],
    stack: [u16; 16],
    stack_pointer: u8,
    delay_timer: u8,
    sound_timer: u8,
    keypress_waiting: bool,
    keypress_register: u8
}

impl CPU {
    pub fn new(rom_path: &str) -> Self {
        let mut ram = [0; 0x1000];

        for i in 0..FONT_DATA.len() {
            ram[i] = FONT_DATA[i];
        }

        let rom = std::fs::read(rom_path).unwrap();
        let rom_size = rom.len();
        for i in 0..rom_size {
            ram[i + 0x200] = rom[i];
        }

        CPU {
            registers: [0; 16],
            register_i: 0,
            vram: [[0; 64]; 32],
            ram: ram,
            program_counter: 0x200,
            stack: [0; 16],
            stack_pointer: 0,
            delay_timer: 0,
            sound_timer: 0,
            keypress_waiting: false,
            keypress_register: 0
        }
    }

    fn run(&mut self, decrement_timers: bool, pressed_keys: [bool; 16]) {
        // I'm not sure if we should decrement timers if we are waiting for a keypress
        // but I'm going to assume we should
        if decrement_timers {
            if self.delay_timer > 0 {
                self.delay_timer -= 1
            }

            if self.sound_timer > 0 {
                self.sound_timer -= 1
            }
        }

        if self.keypress_waiting {
            for i in 0..pressed_keys.len() {
                if pressed_keys[i] {
                    self.registers[self.keypress_register as usize] = i as u8;
                    self.keypress_waiting = false;
                    break;
                }
            }
            return;
        }

        let opcode = self.read_opcode();
        // println!("pc = 0x{:04x}, opcode = 0x{:04x}", self.program_counter, opcode);
        self.program_counter += 2;
        
        let _c       = ((opcode & 0xF000) >> 12) as u8;
        let x       = ((opcode & 0x0F00) >> 8) as u8;
        let y       = ((opcode & 0x00F0) >> 4) as u8;
        let d       = ((opcode & 0x000F) >> 0) as u8;
        let nnn     = opcode & 0x0FFF;
        let kk      = (opcode & 0x00FF) as u8;
        let opminor = (opcode & 0x000F) as u8;

        match opcode {
            0x00E0          => self.cls(),
            0x00EE          => self.ret(),
            0x1000..=0x1FFF => self.jp(nnn),
            0x2000..=0x2FFF => self.call(nnn),
            0x3000..=0x3FFF => self.se(x, kk),
            0x4000..=0x4FFF => self.sne(x, kk),
            0x5000..=0x5FF0 => self.se_xy(x, y),
            0x6000..=0x6FFF => self.ld(x, kk),
            0x7000..=0x7FFF => self.add(x, kk),
            0x8000..=0x8FFF => match opminor {
                0x0 => self.ld_xy(x, y),
                0x1 => self.or_xy(x, y),
                0x2 => self.and_xy(x, y),
                0x3 => self.xor_xy(x, y),
                0x4 => self.add_xy(x, y),
                0x5 => self.sub_xy(x, y),
                0x6 => self.shr_x(x),
                0x7 => self.subn_xy(x, y),
                0xE => self.shl_x(x),
                _ => println!("WARN: unsupported opcode {:04x}", opcode)
            },
            0x9000..=0x9FF0 => self.sne_xy(x, y),
            0xA000..=0xAFFF => self.ld_i(nnn),
            0xB000..=0xBFFF => self.jp_0(nnn),
            0xC000..=0xCFFF => self.rnd(x, kk),
            0xD000..=0xDFFF => self.drw(x, y, d),
            0xE000..=0xEFFF => match kk {
                0x9E => self.skp(x, pressed_keys),
                0xA1 => self.sknp(x, pressed_keys),
                _    => println!("WARN: unsupported opcode {:04x}", opcode)
            },
            0xF000..=0xFFFF => match kk {
                0x07 => self.ld_x_dt(x),
                0x0A => self.ld_k(x),
                0x15 => self.ld_dt(x),
                0x18 => self.ld_st(x),
                0x1E => self.add_i(x),
                0x29 => self.ld_f(x),
                0x33 => self.ld_b(x),
                0x55 => self.ld_vx_to_i(x),
                0x65 => self.ld_i_to_vx(x),
                _    => println!("WARN: unsupported opcode {:04x}", opcode)
            }
            _ => println!("WARN: unsupported opcode {:04x}", opcode)
        }

        
    }
    
    fn read_opcode(&mut self) -> u16 {
        let p = self.program_counter as usize;
        let op_byte_1 = self.ram[p] as u16;
        let op_byte_2 = self.ram[p + 1] as u16;
        (op_byte_1 << 8) | op_byte_2
    }

    // (0x00E0) Clear the screen
    fn cls(&mut self) {
        self.vram = [[0; WIDTH]; HEIGHT];
    }

    // (0x00EE) Return from current sub-rotine 
    fn ret(&mut self) {
        if self.stack_pointer == 0 {
            panic!("stack underflow");
        }

        self.stack_pointer -= 1;
        self.program_counter = self.stack[self.stack_pointer as usize];
    }

    // (0x1NNN) Jump to address NNN (addr)
    fn jp(&mut self, addr: u16) {
        self.program_counter = addr;
    }

    // (0x2NNN) Call sub-routine at address NNN (addr)
    fn call(&mut self, addr: u16) {
        let sp = self.stack_pointer as usize;
        let stack = &mut self.stack;

        if sp > stack.len() {
            panic!("stack overflow");
        }

        stack[sp] = self.program_counter;
        self.stack_pointer += 1;
        self.program_counter = addr;
    }

    // (0x3XKK) Skip the next instruction if register X == KK
    fn se(&mut self, x: u8, kk: u8) {
        if self.registers[x as usize] == kk {
            self.program_counter += 2
        }
    }

    // (0x4XKK) Skip the next instruction if register X != KK
    fn sne(&mut self, x: u8, kk: u8) {
        if self.registers[x as usize] != kk {
            self.program_counter += 2
        }
    }

    // (0x5XY0) Skip the next instruction if register X == register Y
    fn se_xy(&mut self, x: u8, y: u8) {
        if self.registers[x as usize] == self.registers[y as usize] {
            self.program_counter += 2
        }
    }

    // (0x6XKK) Load value KK into register X
    fn ld(&mut self, x: u8, kk: u8) {
        self.registers[x as usize] = kk;
    }

    // (0x7XKK) Add value KK into register X
    fn add(&mut self, x: u8, kk: u8) {
        self.registers[x as usize] = (self.registers[x as usize] as u16  + kk as u16) as u8;
    }

    // (0x8XY0) Load register Y into register X
    fn ld_xy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] = self.registers[y as usize]
    }

    // (0x8XY1) Bitwise OR on the values of registers X and Y, stores the result in X
    fn or_xy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] |= self.registers[y as usize]
    }

    // (0x8XY2) Bitwise AND on the values of registers X and Y, stores the result in X
    fn and_xy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] &= self.registers[y as usize]
    }

    // (0x8XY3) Bitwise XOR on the values of registers X and Y, stores the result in X
    fn xor_xy(&mut self, x: u8, y: u8) {
        self.registers[x as usize] ^= self.registers[y as usize]
    }

    // (0x8XY4) Sum register Y into register X and set overflow at register 0xF 
    fn add_xy(&mut self, x: u8, y: u8) {
        let op1 = self.registers[x as usize];
        let op2 = self.registers[y as usize];
        let (res, overflow) = op1.overflowing_add(op2);
        self.registers[0xF] = overflow as u8;
        self.registers[x as usize] = res;
    }

    // (0x8XY5) Sub register Y into register X and set NOT borrow at register 0xF 
    fn sub_xy(&mut self, x: u8, y: u8) {
        let op1 = self.registers[x as usize];
        let op2 = self.registers[y as usize];
        let (res, overflow) = op1.overflowing_sub(op2);
        self.registers[0xF] = (!overflow) as u8;
        self.registers[x as usize] = res;
    }

    // (0x8X_6) Register F is set to X least-significant bit. Divide register X by 2.   
    fn shr_x(&mut self, x: u8) {
        self.registers[0xF] = self.registers[x as usize] & 1;
        self.registers[x as usize] >>= 1;
    }

    // (0x8XY7) Compute X = Y - X and set NOT borrow at register 0xF 
    fn subn_xy(&mut self, x: u8, y: u8) {
        let op1 = self.registers[x as usize];
        let op2 = self.registers[y as usize];
        let (res, overflow) = op2.overflowing_sub(op1);
        self.registers[0xF] = (!overflow) as u8;
        self.registers[x as usize] = res;
    }

    // (0x8X_6) Register F is set to X most-significant bit. Multiply register X by 2.   
    fn shl_x(&mut self, x: u8) {
        self.registers[0xF] = (self.registers[x as usize] >> 7) & 1;
        self.registers[x as usize] <<= 1;
    }

    // (0x9XY0) Skip the next instruction if register X != register Y  
    fn sne_xy(&mut self, x: u8, y: u8) {
        if self.registers[x as usize] != self.registers[y as usize] {
            self.program_counter += 2
        }
    }

    // (0xANNN) Load register I with NNN
    fn ld_i(&mut self, nnn: u16) {
        self.register_i = nnn;
    }

    // (0xBNNN) Jump to address NNN + register 0
    fn jp_0(&mut self, nnn: u16) {
        self.program_counter = nnn + self.registers[0] as u16;
    }

    // (0xCXKK) Generates a random number from 0 to 255, which is then ANDed with the value KK. The results are stored in register X.
    fn rnd(&mut self, x: u8, kk: u8) {
        let mut rng = rand::thread_rng();
        let random_number: u8 = rng.gen();
        self.registers[x as usize] = random_number & kk;
    }

    // (0xDXYN) Draw a sprite at position X, Y with N bytes of sprite data starting at
    //          the address stored in I. Set register F to 1 if any set pixels are changed
    //          to unset
    fn drw(&mut self, x: u8, y: u8, n: u8) {
        self.registers[0x0f] = 0; // Initialize collision flag to 0
        for j in 0..n {
            let y = (self.registers[y as usize] as usize + j as usize) % HEIGHT;
            for i in 0..8 {
                let x = (self.registers[x as usize] as usize + i) % WIDTH;
                let color = (self.ram[self.register_i as usize + j as usize] >> (7 - i)) & 1;
                self.registers[0x0f] |= color & self.vram[y][x]; // Set collision flag
                self.vram[y][x] ^= color;
            }
        }
    }

    // (0xEX9E) Skip the next instruction if the key stored in register X is pressed
    fn skp(&mut self, x: u8, pressed_keys: [bool; 16]) {
        let key = self.registers[x as usize];
        if pressed_keys[key as usize] {
            self.program_counter += 2;
        }
    }

    // (0xEXA1) Skip the next instruction if the key stored in register X is not pressed
    fn sknp(&mut self, x: u8, pressed_keys: [bool; 16]) {
        let key = self.registers[x as usize];
        if !pressed_keys[key as usize] {
            self.program_counter += 2;
        }
    }

    // (0xFX07) Set register X to the value of delay timer
    fn ld_x_dt(&mut self, x: u8) {
        self.registers[x as usize] = self.delay_timer;
    }

    // (0xFX0A) Wait for a key press, store the value of the key in register X
    fn ld_k(&mut self, x: u8) {
        self.keypress_waiting = true;
        self.keypress_register = x;
    }

    // (0xFX15) Set delay timer to the value of register X
    fn ld_dt(&mut self, x: u8) {
        self.delay_timer = self.registers[x as usize];
    }

    // (0xFX18) Set sound timer to the value of register X
    fn ld_st(&mut self, x: u8) {
        self.sound_timer = self.registers[x as usize];
    }

    // (0xFX1E) Add the value of register X to register I
    fn add_i(&mut self, x: u8) {
        self.register_i += self.registers[x as usize] as u16;
    }

    // (0xFX29) Set register I to the address of the sprite data corresponding to the hexadecimal digit stored in register X
    fn ld_f(&mut self, x: u8) {
        // We multiply by 5 because each sprite is 5 bytes long
        self.register_i = (self.registers[x as usize] * 5) as u16;
    }

    // (0xFX33) Store the binary-coded decimal equivalent of the value stored in register X at addresses I, I+1, and I+2
    fn ld_b(&mut self, x: u8) {
        let mut value = self.registers[x as usize];
        let i = self.register_i as usize;
        let mem = &mut self.ram;
        mem[i + 2] = value % 10;
        value /= 10;
        mem[i + 1] = value % 10;
        value /= 10;
        mem[i] = value % 10;
    }

    // (0xFX55) Store the values of registers 0 to X inclusive in memory starting at address I
    fn ld_vx_to_i(&mut self, x: u8) {
        let i = self.register_i as usize;
        let mem = &mut self.ram;
        for j in 0..=(x as usize) {
            mem[i + j] = self.registers[j];
        }
    }

    // (0xFX65) Fill registers 0 to X inclusive with the values stored in memory starting at address I
    fn ld_i_to_vx(&mut self, x: u8) {
        let i = self.register_i as usize;
        let mem = &self.ram;
        for j in 0..=(x as usize) {
            self.registers[j] = mem[i + j];
        }
    }
}

struct SquareWave {
    phase_inc: f32,
    phase: f32,
    volume: f32,
}

impl AudioCallback for SquareWave {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        // Generate a square wave
        for x in out.iter_mut() {
            *x = self.volume * if self.phase < 0.5 { 1.0 } else { -1.0 };
            self.phase = (self.phase + self.phase_inc) % 1.0;
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        panic!("Usage: cargo run <rom_path>");
    }
    let rom_path = &args[1];

    // Initialize SDL2
    let sdl_context = sdl2::init().unwrap();

    // Initialize video
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("CHIP-8", (WIDTH as u32) * RESOLUTION_SCALE, (HEIGHT as u32) * RESOLUTION_SCALE)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut events = sdl_context.event_pump().unwrap();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    // Initialize audio
    let audio_subsystem = sdl_context.audio().unwrap();
    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1),
        samples: None,
    };

    let audio_device = audio_subsystem
            .open_playback(None, &desired_spec, |spec| {
                SquareWave {
                    phase_inc: 240.0 / spec.freq as f32,
                    phase: 0.0,
                    volume: 0.25,
                }
            })
            .unwrap();
    
    let mut cpu = CPU::new(rom_path);
    let mut timer_tick = std::time::Instant::now();
    let mut cpu_tick = std::time::Instant::now();

    loop {
        // Decrement timers at 60hz
        let decrement_timers: bool;
        let elapsed = timer_tick.elapsed().as_micros();
        if  elapsed >= 16600 {
            decrement_timers = true;
            timer_tick = std::time::Instant::now();
        } else {
            decrement_timers = false;
        }

        // Process events such as keyboard input
        for event in events.poll_iter() {
            if let Event::Quit { .. } = event {
                return; // Quit the game
            };
        }

        let pressed_scancodes: Vec<Keycode> = events
            .keyboard_state()
            .pressed_scancodes()
            .filter_map(Keycode::from_scancode)
            .collect();

        let mut pressed_keys = [false; 16];

        for scancode in pressed_scancodes {
            let keycode = match scancode {
                Keycode::Num1 => Some(0x1),
                Keycode::Num2 => Some(0x2),
                Keycode::Num3 => Some(0x3),
                Keycode::Num4 => Some(0xc),
                Keycode::Q => Some(0x4),
                Keycode::W => Some(0x5),
                Keycode::E => Some(0x6),
                Keycode::R => Some(0xd),
                Keycode::A => Some(0x7),
                Keycode::S => Some(0x8),
                Keycode::D => Some(0x9),
                Keycode::F => Some(0xe),
                Keycode::Z => Some(0xa),
                Keycode::X => Some(0x0),
                Keycode::C => Some(0xb),
                Keycode::V => Some(0xf),
                _ => None,
            };

            if let Some(i) = keycode {
                pressed_keys[i] = true;
            }
        }

        // Run a single instruction
        cpu.run(decrement_timers, pressed_keys);
        
        // Draw vram to canvas
        for (y, row) in cpu.vram.iter().enumerate() {
            for (x, &col) in row.iter().enumerate() {
                let x = (x as u32) * RESOLUTION_SCALE;
                let y = (y as u32) * RESOLUTION_SCALE;
                let color = match col {
                    0 => BACKGROUND_COLOR,
                    1 => FOREGROUND_COLOR,
                    _ => panic!("Invalid color value"),
                };

                canvas.set_draw_color(color);
                let _ = canvas.fill_rect(Rect::new(x as i32, y as i32, RESOLUTION_SCALE, RESOLUTION_SCALE));
            }
        }
        canvas.present();

        // Play sound
        if cpu.sound_timer > 0 {
            audio_device.resume();
        } else {
            audio_device.pause();
        }

        // Sleep so the CPU runs at 500hz
        let elapsed = cpu_tick.elapsed().as_micros();
        if elapsed < 2000 {
            std::thread::sleep(std::time::Duration::from_micros(2000 - elapsed as u64));
        }
        cpu_tick = std::time::Instant::now();
    }
}
