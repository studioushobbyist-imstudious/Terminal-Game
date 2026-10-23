
use crossterm::{cursor, execute, terminal};
use std::io::stdout;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use std::time::{Duration, Instant};

mod texture;
use texture::{texture_make_text, texture_load, texture_make_circle};

mod screen;
use screen::Screen;

mod ball;
use ball::Ball;

mod font;
use font::{font_load};

mod audio;
use audio::AudioManager;

mod animation;

use crate::animation::Tick;

struct Settings{
    debug: bool,
    fps: u64
}

impl Settings{
    fn new() -> Self{
        Settings { 
            debug: true,
            fps: 0
        }
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (width, height) = terminal::size()?;

    terminal::enable_raw_mode()?;
    execute!(stdout(), cursor::Hide)?;

    let mut screen = Screen::new(width as u32, height as u32);
    screen.clear_terminal()?;
    let settings = Settings::new();

    // region: Fps
    let fps_cap: u64 = settings.fps;
    let mut fps_timer = Instant::now();
    let mut frames = 0u32;
    let mut fps: u32;
    let mut t1 = texture_make_text(&format!("Terminal width:  {width} columns"), '\n');
    let mut t2 = texture_make_text(&format!("Terminal height: {height} rows"), '\n');
    let mut t3 = texture_make_text("FPS: 0", '\n'); 
    // endregion: Fps
    
    let font = font_load("assets/fonts/Terminal-Bound.af").map_err(std::io::Error::other)?;
    let audio = AudioManager::new()?;



    let logo = texture_load("assets/textures/ascii/Logo.at")?;

    let mut anim_seq = animation::TextureAnimationSequence::new(
        animation::Bayer4, 
        vec![
            animation::Phase::forward(fps_cap as u32, animation::Easing::EaseInOut),   
            animation::Phase::hold(fps_cap as u32 / 4, 1.0),                    
            animation::Phase::backward(fps_cap as u32, animation::Easing::EaseInOut),  
        ]
    );

    screen.persist_draw_texture(&t1, 0, 0);
    screen.persist_draw_texture(&t2, 0, 1);
    // audio.play("assets/audio/test/WSIHL.wav")?;

    let mut t : u128 = 0;
    loop {
        t = t.saturating_add(1);
        let mut screen_width = screen.width;
        let mut screen_height = screen.height;
        let frame_start = Instant::now();


        screen.clear_grid();

        frames += 1;
        if fps_timer.elapsed() >= Duration::from_secs(1) {
            fps = frames;
            frames = 0;
            fps_timer = Instant::now();
            t3 = texture_make_text(&format!("IPS: {fps}"), '\n'); 
        }

        screen.draw_texture(&t3, 0, 2); 
        
        // draw here
        screen.draw_texture_animated(&logo, 
            (screen_width/2).saturating_sub(logo.width/2), 
            (screen_height/2).saturating_sub(logo.height/2),
            &anim_seq
        );
        anim_seq.tick();
        
        screen.draw_texture(
            &font.render_text(
                "hello", 
                "straight", 
                2, 
                2
            )?, 
            0, 
            screen_height.saturating_sub(2)
        );

        screen.push_back_cursor()?;
        screen.draw()?;

        let target = if fps_cap == 0 {
            Duration::ZERO
        } else {
            Duration::from_secs_f64(1.0 / fps_cap as f64)
        };

        let remaining = target.saturating_sub(frame_start.elapsed());
        if event::poll(remaining)? { // change 3: use `remaining`, not a fresh 33 ms
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if key.code == KeyCode::Char('q') {
                        break;
                    }
                    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                        break;
                    }
                }
                Event::Resize(w, h) => {
                    screen.resize(w as u32, h as u32);
                    screen_width = screen.width;
                    screen_height = screen.height;
                    screen.clear_terminal()?;
                    if settings.debug{
                        screen.clear_persist_grid();
                        t1 = texture_make_text(&format!("Terminal width:  {screen_width} columns"), '\n');
                        t2 = texture_make_text(&format!("Terminal height: {screen_height} rows"), '\n');
                        screen.persist_draw_texture(&t1, 0, 0);
                        screen.persist_draw_texture(&t2, 0, 1);
                    }
                }
                _ => {}
            }
        }
    }

    execute!(stdout(), cursor::Show)?;
    terminal::disable_raw_mode()?;
    Ok(())
}