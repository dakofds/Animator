mod animation;
mod app;
mod drawing;
mod render;
mod ui;

use app::{App, Command};
use render::Renderer;
use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::PixelFormatEnum;
use std::time::{Duration, Instant};

const W: u32 = 900;
const H: u32 = 600;
const SAVE_PATH: &str = "projeto.json";

fn do_save(app: &App) {
    match app.save(SAVE_PATH) {
        Ok(()) => println!("salvo em {SAVE_PATH}"),
        Err(e) => eprintln!("erro ao salvar: {e}"),
    }
}

fn do_open(app: &mut App) {
    match app.load(SAVE_PATH) {
        Ok(()) => println!("carregado de {SAVE_PATH}"),
        Err(e) => eprintln!("erro ao carregar: {e}"),
    }
}

fn main() -> Result<(), String> {
    let sdl = sdl2::init()?;
    let video = sdl.video()?;
    let window = video
        .window("Luma Animator", W, H)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;
    let mut canvas = window
        .into_canvas()
        .software()
        .build()
        .map_err(|e| e.to_string())?;
    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_streaming(PixelFormatEnum::ABGR8888, W, H)
        .map_err(|e| e.to_string())?;
    let mut events = sdl.event_pump()?;

    let mut app = App::new(W as f32, H as f32, 1.0);
    let mut renderer = Renderer::new(W, H);
    let mut last = Instant::now();

    'main: loop {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'main,

                Event::Window { win_event: WindowEvent::Exposed, .. } => app.mark_dirty(),

                Event::MouseButtonDown { mouse_btn: MouseButton::Left, x, y, .. } => {
                    app.pointer_down(x as f32, y as f32, 1.0)
                }
                Event::MouseMotion { x, y, .. } => app.pointer_move(x as f32, y as f32, 1.0),
                Event::MouseButtonUp { mouse_btn: MouseButton::Left, .. } => app.pointer_up(),

                Event::KeyDown { keycode: Some(key), repeat: false, .. } => match key {
                    Keycode::Right => app.next_frame(),
                    Keycode::Left => app.prev_frame(),
                    Keycode::N => app.add_frame(),
                    Keycode::Delete => app.remove_frame(),
                    Keycode::Z => app.undo(),
                    Keycode::Space => app.toggle_play(),
                    Keycode::O => app.toggle_onion(),
                    Keycode::B => app.use_pen(),
                    Keycode::E => app.use_eraser(),
                    Keycode::LeftBracket => app.change_width(-2.0),
                    Keycode::RightBracket => app.change_width(2.0),
                    Keycode::S => do_save(&app),
                    Keycode::L => do_open(&mut app),
                    _ => {}
                },

                _ => {}
            }
        }

        while let Some(command) = app.take_command() {
            match command {
                Command::Save => do_save(&app),
                Command::Open => do_open(&mut app),
                Command::Quit => break 'main,
            }
        }

        let now = Instant::now();
        app.update(now - last);
        last = now;

        if app.take_dirty() {
            app.render(&mut renderer);
            texture
                .update(None, renderer.data(), (W * 4) as usize)
                .map_err(|e| e.to_string())?;
            canvas.copy(&texture, None, None)?;
            canvas.present();
        }

        std::thread::sleep(Duration::from_millis(8));
    }

    Ok(())
}
