use crossterm::event::{Event, KeyCode, KeyEventKind};

use crate::ball::Ball;
use crate::engine::Context;
use crate::scenes::{Scene, Transition};
use crate::texture::texture_make_circle;

pub struct BallDemo {
    ball: Ball,
}

impl BallDemo {
    pub fn new() -> Self {
        BallDemo { ball: Ball::new(0, 0, 2, 1, texture_make_circle(12, 6, '@', ' ')) }
    }
}

impl Scene for BallDemo {
    fn update(&mut self, ctx: &mut Context) -> Result<Transition, Box<dyn std::error::Error + 'static>> {
        self.ball.update(ctx.screen.width, ctx.screen.height);
        Ok(Transition::None)
    }

    fn handle_event(&mut self, _ctx: &mut Context, ev: &Event) -> Result<Transition, Box<dyn std::error::Error + 'static>> {
        if let Event::Key(k) = ev {
            if k.kind == KeyEventKind::Press && matches!(k.code, KeyCode::Esc | KeyCode::Backspace) {
                return Ok(Transition::Pop); // back to the menu underneath
            }
        }
        Ok(Transition::None)
    }

    fn render(&self, ctx: &mut Context) {
        self.ball.draw(&mut ctx.screen);
    }
}