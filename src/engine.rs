use crate::{audio::AudioManager, screen::Screen, font::{font_load, Font}, scenes::{Scene, Transition}, settings::Settings};
use std::time::{Duration, Instant};
use crossterm::event::{self, Event, KeyEventKind};
use crossterm::{cursor, execute, terminal};
use std::io::stdout;


use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn is_quit(key: &KeyEvent) -> bool {
    key.code == KeyCode::Char('q')
        || (key.code == KeyCode::Char('c')
            && key.modifiers.contains(KeyModifiers::CONTROL))
}

pub struct Context {
    pub screen: Screen,
    pub audio: AudioManager,
    pub font: Font,        // whatever your font type is called
    pub settings: Settings,
}

impl Context {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (width, height) = terminal::size()?;
        let screen = Screen::new(width as u32, height as u32);
        let font = font_load("assets/fonts/Terminal-Bound.af")?;
        let audio = AudioManager::new()?;

        Ok(Context {
            screen,
            audio,
            font,
            settings: Settings::new(),
        })
    }
}

fn apply(t: Transition, scenes: &mut Vec<Box<dyn Scene>>, ctx: &mut Context) -> bool {
    match t {
        Transition::None => {}
        Transition::Push(mut s) => { s.on_enter(ctx); scenes.push(s); }
        Transition::Replace(mut s) => { scenes.pop(); s.on_enter(ctx); scenes.push(s); }
        Transition::Pop => { scenes.pop(); }
        Transition::Quit => return true,
    }
    scenes.is_empty()   // true means "time to quit"
}

pub fn run(mut ctx: Context, first: Box<dyn Scene>) -> Result<(), Box<dyn std::error::Error>> {
    let mut scenes: Vec<Box<dyn Scene>> = Vec::new();
    apply(Transition::Push(first), &mut scenes, &mut ctx);

    loop {
        let frame_start = Instant::now();

        // update
        let Some(scene) = scenes.last_mut() else { break };
        let t = scene.update(&mut ctx);
        if apply(t?, &mut scenes, &mut ctx) { break; }

        // render
        let Some(scene) = scenes.last() else { break };
        ctx.screen.clear_grid();
        scene.render(&mut ctx);
        // debug overlay goes here, BEFORE draw (see the note below)
        ctx.screen.push_back_cursor()?;
        ctx.screen.draw()?;

        // events
        let target = Duration::from_secs_f64(1.0 / ctx.settings.fps.max(1) as f64);
        if event::poll(target.saturating_sub(frame_start.elapsed()))? {
            match event::read()? {
                Event::Resize(w, h) => {
                    ctx.screen.resize(w as u32, h as u32);
                    ctx.screen.clear_terminal()?;
                }
                Event::Key(k) if k.kind == KeyEventKind::Press && is_quit(&k) => break,
                ev => {
                    if let Some(scene) = scenes.last_mut() {
                        let t = scene.handle_event(&mut ctx, &ev);
                        if apply(t?, &mut scenes, &mut ctx) { break; }
                    }
                }
            }
        }
    }
    Ok(())
}

pub struct TerminalGuard;

impl TerminalGuard {
    pub fn new() -> std::io::Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(stdout(), cursor::Hide)?;
        Ok(TerminalGuard)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(stdout(), cursor::Show);
        let _ = terminal::disable_raw_mode();
    }
}