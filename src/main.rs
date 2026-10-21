
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

fn main() -> std::io::Result<()> {
    let (width, height) = terminal::size()?;

    let fps_cap: u64 = 0;
    let mut fps_timer = Instant::now();
    let mut frames = 0u32;
    let mut fps: u32;

    let mut screen = Screen::new(width as u32, height as u32);
    let mut ball = Ball::new(0, 0, 2, 1, texture_make_circle(12, 6, '@', ' '));
    let swag_texture = texture_load("assets/SwagTest.at").map_err(std::io::Error::other)?;

    let font = font_load("assets/Terminal-Gothic.af").map_err(std::io::Error::other)?;
    let mut text_selected = true;
    let mut text_texture = font.render_text(
        "aaa",
        "straight",
        5,
        7,
    ).map_err(std::io::Error::other)?;

    screen.draw_texture(&text_texture, 0, 0);

    terminal::enable_raw_mode()?;
    execute!(stdout(), cursor::Hide)?;
    screen.clear_terminal()?;
    let start = Instant::now();

    let mut t1 = texture_make_text(&format!("Terminal width:  {width} columns"), '\n');
    let mut t2 = texture_make_text(&format!("Terminal height: {height} rows"), '\n');
    let mut t3 = texture_make_text("FPS: 0", '\n'); // Declare t3 outside the loop

    screen.persist_draw_texture(&t1, 0, 0);
    screen.persist_draw_texture(&t2, 0, 1);
    
    loop {
        let t = start.elapsed().as_secs_f32();

        let frame_start = Instant::now();
        let mut screen_width = screen.width;
        let mut screen_height = screen.height;

        screen.clear_grid();

        let s = 0.8 + 0.2 * (t * 3.0).sin();
        let draw_width = (swag_texture.width as f32 * s) as u32;
        let draw_height = (swag_texture.height as f32 * s) as u32;

        let x = screen_width.saturating_sub(draw_width) / 2;
        let y = screen_height.saturating_sub(draw_height) / 2;

        screen.draw_texture_scaled(&swag_texture, x, y, s);

        ball.update(screen_width, screen_height);
        ball.draw(&mut screen);

        // Count frames, update the fps value and regenerate t3 once per second
        frames += 1;
        if fps_timer.elapsed() >= Duration::from_secs(1) {
            fps = frames;
            frames = 0;
            fps_timer = Instant::now();
            t3 = texture_make_text(&format!("IPS: {fps}"), '\n'); 
            text_texture = font.render_text(
                &format!("{fps} ips"),
                "straight",
                5,
                7,
            ).map_err(std::io::Error::other)?;

            text_texture.apply_trans = if text_selected {true} else {false}

        }

        // Draw t3 EVERY frame so it doesn't get cleared by clear_grid()
        screen.draw_texture(&t3, 0, 2); 
        screen.draw_texture(&text_texture, 20, 40);

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
                    if key.code == KeyCode::Char('s') {
                        text_selected = !text_selected;
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
                    t1 = texture_make_text(&format!("Terminal width:  {screen_width} columns"), '\n');
                    t2 = texture_make_text(&format!("Terminal height: {screen_height} rows"), '\n');
                    screen.persist_draw_texture(&t1, 0, 0);
                    screen.persist_draw_texture(&t2, 0, 1);
                }
                _ => {}
            }
        }
    }

    execute!(stdout(), cursor::Show)?;
    terminal::disable_raw_mode()?;
    Ok(())
}