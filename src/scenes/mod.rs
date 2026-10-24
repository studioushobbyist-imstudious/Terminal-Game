pub mod splash;
pub mod main_menu;
pub mod ball_demo;
pub mod settings;

use crossterm::event::Event;
use crate::engine::Context;

pub enum Transition {
    None,
    Push(Box<dyn Scene>),     // go on top, keep this one underneath (pause menu)
    Replace(Box<dyn Scene>),  // swap this scene for another (splash -> menu)
    Pop,                      // back to the one underneath
    Quit,
}

pub type SceneResult = Result<Transition, Box<dyn std::error::Error>>;

pub trait Scene {
    fn on_enter(&mut self, _ctx: &mut Context) {}
    fn handle_event(&mut self, _ctx: &mut Context, _event: &Event) -> SceneResult { Ok(Transition::None) }
    fn update(&mut self, _ctx: &mut Context) -> SceneResult { Ok(Transition::None) }
    fn render(&self, ctx: &mut Context);
}