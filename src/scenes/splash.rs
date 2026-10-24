use crate::texture::{Texture, texture_load};
use crate::animation::{TextureAnimationSequence,Bayer4,Phase,Easing};
use crate::engine::Context;
use crate::scenes::{Transition, Scene};
use crate::animation::Tick;
use crossterm::event::{Event, KeyEventKind};
use crate::scenes::main_menu::MainMenu;

pub struct Splash {
    logo: Texture,
    credit: Texture,
    tooltip: Texture,
    anim: TextureAnimationSequence<Bayer4>,
    index: u32,
}

impl Splash {
    pub fn new(ctx: &Context) -> Result<Self, Box<dyn std::error::Error>> {
        let fps = ctx.settings.fps as u32;
        Ok(Splash {
            logo: texture_load("assets/textures/ascii/Logo.at")?,
            credit: ctx.font.render_text("made by marcus morgan", "straight", 3, 3)?,
            tooltip: ctx.font.render_text("press any button to skip", "straight", 2, 2)?,
            anim: TextureAnimationSequence::new(Bayer4, vec![
                Phase::forward(fps, Easing::EaseInOut),
                Phase::hold(fps / 4, 1.0),
                Phase::backward(fps, Easing::EaseInOut),
                Phase::hold(fps / 4, 0.0),
            ]),
            index: 0,
        })
    }
}

impl Scene for Splash {
    fn update(&mut self, _ctx: &mut Context) -> Result<Transition, Box<dyn std::error::Error + 'static>> {
        self.anim.tick();
        if self.anim.finished() {
            self.index += 1;
            self.anim.reset();
            if self.index >= 2 {
                return Ok(Transition::Replace(Box::new(MainMenu::new(&_ctx)?)));
            }
        }
        Ok(Transition::None) 
    }

    fn handle_event(&mut self, _ctx: &mut Context, ev: &Event) -> Result<Transition, Box<dyn std::error::Error + 'static>>{
        // any key skips the splash
        if let Event::Key(k) = ev {
            if k.kind == KeyEventKind::Press {
                return Ok(Transition::Replace(Box::new(MainMenu::new(&_ctx)?)));
            }
        }
        Ok(Transition::None)
    }

    fn render(&self, ctx: &mut Context) {
        let tex = if self.index == 0 { &self.logo } else { &self.credit };
        let (w, h) = (ctx.screen.width, ctx.screen.height);
        ctx.screen.draw_texture_animated(
            tex,
            (w / 2).saturating_sub(tex.width / 2),
            (h / 2).saturating_sub(tex.height / 2),
            &self.anim,
        );
        ctx.screen.draw_texture(&self.tooltip, 0, ctx.screen.height.saturating_sub(self.tooltip.height));
    }
}