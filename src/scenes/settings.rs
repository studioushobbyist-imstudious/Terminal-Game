use crossterm::event::{Event, KeyCode, KeyEventKind};

use crate::animation::{Bayer4, Easing, Phase, TextureAnimationSequence, Tick};
use crate::engine::Context;
use crate::scenes::{Scene, Transition};
use crate::texture::{texture_make_text, Texture};

#[derive(Clone, Copy)]
enum SettingsAction {
    Back
}

struct SettingsItem {
    texture: Texture,
    action: SettingsAction,
}

pub struct SettingsScene {
    items: Vec<SettingsItem>,
    selected: usize,
    marker_right: Texture,
}

fn _center(total: u32, size: u32) -> u32 {
    (total / 2).saturating_sub(size / 2)
}

impl SettingsScene {
    pub fn new(ctx: &Context) -> Self {
        // if the font can't render a label, fall back to plain text instead of failing
        let back = ctx.font.render_text("back", "straight", 2, 2)
            .unwrap_or_else(|_| texture_make_text("back", '\n'));


        let _fps = ctx.settings.fps as u32;

        SettingsScene {
            items: vec![
                SettingsItem { texture: back, action: SettingsAction::Back }
            ],
            selected: 0,
            marker_right: texture_make_text("<", '\n'),
        }
    }

    fn activate(&self) -> Transition {
        match self.items[self.selected].action {
            SettingsAction::Back => Transition::Pop,

        }
    }
}

impl Scene for SettingsScene {
    fn update(&mut self, _ctx: &mut Context) -> Result<Transition, Box<dyn std::error::Error + 'static>> {
        Ok(Transition::None)
    }

    fn handle_event(&mut self, _ctx: &mut Context, ev: &Event) -> Result<Transition, Box<dyn std::error::Error + 'static>> {
        let Event::Key(key) = ev else { return Ok(Transition::None) };

        let n = self.items.len();
        match key.code {
            KeyCode::Up | KeyCode::Char('w') => self.selected = (self.selected + n - 1) % n,
            KeyCode::Down | KeyCode::Char('s') => self.selected = (self.selected + 1) % n,
            KeyCode::Enter | KeyCode::Char(' ') => return Ok(self.activate()),
            _ => {}
        }
        Ok(Transition::None)
    }

    fn render(&self, ctx: &mut Context) {
        let (_sw, _sh) = (ctx.screen.width, ctx.screen.height);
        let gap = 1;

        let mut y = 0;

        for (i, item) in self.items.iter().enumerate() {
            let x = 0;
            ctx.screen.draw_texture(&item.texture, x, y);

            if i == self.selected {
                ctx.screen.draw_texture(&self.marker_right, 
                    item.texture.width + 2, 
                    i as u32 * item.texture.height
                );
                ctx.screen.draw_texture(&self.marker_right, 
                    item.texture.width + 2, 
                    (i as u32 * item.texture.height).saturating_add(1)
                );
            }
            y += item.texture.height + gap;
        }
    }
}