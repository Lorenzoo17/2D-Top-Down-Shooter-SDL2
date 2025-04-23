use std::time::Duration;
use std::time::Instant;

const SCREEN_WIDTH:u32 = 800;
const SCREEN_HEIGHT:u32 = 600;

mod game;
mod modules;
use game::Game;

fn main() -> Result<(), String>{

    let screen_widht = SCREEN_WIDTH;
    let screen_height = SCREEN_HEIGHT;
    
    let sdl_context = sdl2::init()?; // ? vale solo se Err() di result e' stringa
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem.window("Rust game", screen_widht, screen_height)
    .position_centered()
    .build()
    .unwrap();

    let mut canvas = window.into_canvas().build().expect("Errore creazione canvas");
    let mut texture_creator = canvas.texture_creator();
    let mut event_pump = sdl_context.event_pump()?;

    let mut last_frame:Instant = Instant::now();
    let mut delta_time:f32;

    let mut game = Game::new(&mut canvas, &mut texture_creator, &mut event_pump).unwrap();
    game.start()?;

    'running: loop{

        let current_time = Instant::now();

        delta_time = (current_time - last_frame).as_secs_f32();
        last_frame = current_time;

        // gestione input
        if game.manage_events() == false{
            break 'running;
        }

        game.render().unwrap();

        game.update(delta_time);
        
        // 60 -> settaggio frame rate
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}
