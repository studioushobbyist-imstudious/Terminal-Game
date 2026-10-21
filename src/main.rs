
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

fn main() -> std::io::Result<()> {
    let (width, height) = terminal::size()?;

    let mut fps_timer = Instant::now();
    let mut frames = 0u32;
    let mut fps = 0u32;

    let mut screen = Screen::new(width as u32, height as u32);
    let mut ball = Ball::new(0, 0, 2, 1, texture_make_circle(12, 6, '@', ' '));
    let swag_texture = texture_load("assets/SwagTest.at").map_err(std::io::Error::other)?;



    terminal::enable_raw_mode()?;
    execute!(stdout(), cursor::Hide)?;
    screen.clear_terminal()?;
    let start = Instant::now();
    
    loop {
        let t = start.elapsed().as_secs_f32();

        let frame_start = Instant::now();
        let screen_width = screen.width;
        let screen_height = screen.height;
        let t1 = texture_make_text(&format!("Terminal width:  {screen_width} columns"), '\n');
        let t2 = texture_make_text(&format!("Terminal height: {screen_height} rows"), '\n');
        let t3 = texture_make_text(&format!("FPS: {fps}"), '\n'); // change 1: FPS text

        screen.clear_grid();

        let s = 0.8 + 0.2 * (t * 3.0).sin();
        screen.draw_texture_scaled(&swag_texture, 
            screen_width/2-(swag_texture.width as f32 * s) as u32/2, 
            screen_height/2-(swag_texture.height as f32 * s) as u32/2, s
        );


        ball.update(screen_width, screen_height);
        ball.draw(&mut screen);

        screen.draw_texture(&t1, 0, 0);
        screen.draw_texture(&t2, 0, 1);
        screen.draw_texture(&t3, 0, 2); 

        screen.push_back_cursor()?;
        screen.draw()?;

        // change 2: count frames, publish the number once per second
        frames += 1;
        if fps_timer.elapsed() >= Duration::from_secs(1) {
            fps = frames;
            frames = 0;
            fps_timer = Instant::now();
        }

        let target = Duration::from_millis(33);
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
                    screen.clear_terminal()?;
                }
                _ => {}
            }
        }
    }

    execute!(stdout(), cursor::Show)?;
    terminal::disable_raw_mode()?;
    Ok(())
}