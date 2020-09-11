use ggez::{Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler};
use ggez::graphics;


mod simulator;
use simulator::Chip8;

const PIXEL_SIZE: u64 = 10;

fn keyboard_map(number: u8) -> ggez::input::keyboard::KeyCode {
    use ggez::input::keyboard::KeyCode;
    match number {
        0 => KeyCode::X,
        1 => KeyCode::Key1,
        2 => KeyCode::Key2,
        3 => KeyCode::Key3,
        4 => KeyCode::Q,
        5 => KeyCode::W,
        6 => KeyCode::R,
        7 => KeyCode::A,
        8 => KeyCode::S,
        9 => KeyCode::D,
        10 => KeyCode::Z,
        11 => KeyCode::C,
        12 => KeyCode::Key4,
        13 => KeyCode::R,
        14 => KeyCode::F,
        15 => KeyCode::V,
        _ => panic!("Invalid keycode")
    }
}


struct State {
    vm: Chip8,
    last_update: core::time::Duration,
    rects: Vec<Vec<Box<graphics::Mesh>>>
}
impl State {
    fn new(ctx: &mut Context) -> State {
        let mut x = State {vm: Chip8::new(), last_update: core::time::Duration::new(0, 0), rects: generate_rects(ctx)};
        x.vm.initialize();
        x.vm.load_program("programs/PONG".to_string());
        x
    }
}

fn generate_rects(ctx: &mut Context) -> Vec<Vec<Box<graphics::Mesh>>> {
    let mut vec = Vec::<Vec::<Box<graphics::Mesh>>>::with_capacity(simulator::GFX_WIDTH);
    for x in 0..simulator::GFX_WIDTH {
        let mut temp = Vec::<Box<graphics::Mesh>>::with_capacity(simulator::GFX_HEIGHT);
        for y in 0..simulator::GFX_HEIGHT {
            let rect = graphics::Rect::new(
                (x as u64*PIXEL_SIZE) as f32,
                (y as u64*PIXEL_SIZE) as f32,
                PIXEL_SIZE as f32,
                PIXEL_SIZE as f32);

            let mesh = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                rect,
                graphics::WHITE
            ).unwrap();

            let boxed = Box::new(mesh);
            temp.push(boxed);
        }
        vec.push(temp);
    }
    vec
}

impl<'a> event::EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let mut pressed = false;
        let mut key = 0;
        for i in 0..16 {
            if ggez::input::keyboard::is_key_pressed(ctx, keyboard_map(i as u8)) && !self.vm.keyboard.0[i] {
                pressed = true;
                key = i;
            }
            self.vm.keyboard.0[i] = ggez::input::keyboard::is_key_pressed(ctx, keyboard_map(i as u8));
        }

        if self.vm.waiting_for_keyboard && pressed {
            self.vm.waiting_for_keyboard = false;
            self.vm.regs[self.vm.keyboard_register] = key as u8;
        }

        let time = ggez::timer::time_since_start(ctx);
        if (time - self.last_update) > core::time::Duration::new(0, 1_000_000_000 / 60) {
            self.last_update = time;
            self.vm.cycle();
        }
        else {
            println!("false");
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        for x in 0..simulator::GFX_WIDTH {
            for y in 0..simulator::GFX_HEIGHT {
                if self.vm.gfx.0[x][y] {
                    graphics::draw(ctx, &(*self.rects[x][y]), graphics::DrawParam::new()).unwrap();
                }
            }
        }

        graphics::present(ctx)
    }
}

fn main() {
    let (mut ctx, mut event_loop) = ContextBuilder::new("Chip8", "Jakub Jendryka").build().unwrap();
    let mode =  ggez::conf::WindowMode::default();
    mode.dimensions((64*PIXEL_SIZE) as f32, (32*PIXEL_SIZE) as f32);
    graphics::set_mode(&mut ctx, mode).unwrap();
    graphics::set_resizable(&mut ctx, false).unwrap();
    let mut state = State::new(&mut ctx);

    match event::run(&mut ctx, &mut event_loop, &mut state) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e)
    }
}
