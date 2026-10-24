use crossterm::event::{Event, KeyCode, KeyEventKind};
use rodio::decoder::Settings;

use crate::animation::{Bayer4, Ctx, Easing, Phase, TextureAnimationSequence, Tick};
use crate::engine::Context;
use crate::scenes::ball_demo::BallDemo;
use crate::scenes::settings;
use crate::scenes::{Scene, Transition};
use crate::texture::{texture_make_text, Texture, texture_load};

#[derive(Clone, Copy)]
enum MenuAction {
    Start,
    Settings,
    Quit,
}

struct MenuItem {
    texture: Texture,
    action: MenuAction,
}

pub struct MainMenu {
    title: Texture,
    items: Vec<MenuItem>,
    selected: usize,
    selection_delay: u32,
    marker_left: Texture,
    marker_right: Texture,
    intro: TextureAnimationSequence<Bayer4>,
}

fn center(total: u32, size: u32) -> u32 {
    (total / 2).saturating_sub(size / 2)
}

impl MainMenu {
    pub fn new(ctx: &Context) -> Result<Self, Box<dyn std::error::Error>> {
        // if the font can't render a label, fall back to plain text instead of failing
        let title = texture_load("assets/textures/ascii/Logo.at")?;
        let start = ctx.font.render_text("start", "straight", 2, 2)
            .unwrap_or_else(|_| texture_make_text("start", '\n'));
        let settings = ctx.font.render_text("settings", "straight", 2, 2)
            .unwrap_or_else(|_| texture_make_text("settings", '\n'));
        let quit = ctx.font.render_text("quit", "straight", 2, 2)
            .unwrap_or_else(|_| texture_make_text("quit", '\n'));

        let fps = ctx.settings.fps as u32;
        Ok(
        MainMenu {
            title,
            items: vec![
                MenuItem { texture: start, action: MenuAction::Start },
                MenuItem { texture: settings, action: MenuAction::Settings },
                MenuItem { texture: quit, action: MenuAction::Quit },
                
            ],
            selected: 0,
            selection_delay: 0,
            marker_left: texture_make_text(">", '\n'),
            marker_right: texture_make_text("<", '\n'),
            intro: TextureAnimationSequence::new(
                Bayer4,
                vec![Phase::forward(fps / 2, Easing::EaseOut)], // half a second
            ),
        })
    }

    fn activate(&self, _ctx: &Context) -> Transition {
        match self.items[self.selected].action {
            MenuAction::Start => Transition::Push(Box::new(BallDemo::new())),
            MenuAction::Settings => Transition::Push(Box::new(settings::SettingsScene::new(&_ctx))),
            MenuAction::Quit => Transition::Quit
        }
    }
    fn up(&mut self, n: usize, _ctx: &Context){ 
        if self.selection_delay == 0{
            self.selection_delay = _ctx.settings.fps as u32/ 4;
            self.selected = (self.selected + n - 1) % n;
            let _ = _ctx.audio.play("assets/audio/sfx/MainMenu-Select.wav");
        }
    }
    fn down(&mut self, n: usize, _ctx: &Context){
        
        if self.selection_delay == 0{
            self.selection_delay = _ctx.settings.fps as u32/ 4;
            self.selected = (self.selected + 1) % n;
            let _ =_ctx.audio.play("assets/audio/sfx/MainMenu-Select.wav");
        }
    }
}

impl Scene for MainMenu {
    fn update(&mut self, _ctx: &mut Context) -> Result<Transition, Box<dyn std::error::Error + 'static>> {
        self.intro.tick();
        self.selection_delay = self.selection_delay.saturating_sub(1);
        Ok(Transition::None)
    }

    fn handle_event(&mut self, _ctx: &mut Context, ev: &Event) -> Result<Transition, Box<dyn std::error::Error + 'static>> {
        let Event::Key(key) = ev else { return Ok(Transition::None) };
        if key.kind != KeyEventKind::Press || !self.intro.finished() {
            return Ok(Transition::None);
        }

        let n = self.items.len();


        match key.code {
            KeyCode::Up | KeyCode::Char('w') => self.up(n, &_ctx),
            KeyCode::Down | KeyCode::Char('s') => self.down(n, &_ctx),
            KeyCode::Enter | KeyCode::Char(' ') => return Ok(self.activate(&_ctx)),
            _ => {}
        }
        Ok(Transition::None)
    }

    fn render(&self, ctx: &mut Context) {
        let (sw, sh) = (ctx.screen.width, ctx.screen.height);
        let gap = 1;

        // centre the whole block (title + items) vertically
        let items_h: u32 = self.items.iter().map(|i| i.texture.height + gap).sum();
        let block_h = self.title.height + 2 + items_h;
        let mut y = sh.saturating_sub(block_h) / 2;

        ctx.screen.draw_texture_animated(
            &self.title,
            center(sw, self.title.width),
            y,
            &self.intro,
        );
        y += self.title.height + 2;

        // the items appear once the title has finished revealing
        if !self.intro.finished() {
            return;
        }

        for (i, item) in self.items.iter().enumerate() {
            let x = center(sw, item.texture.width);
            ctx.screen.draw_texture(&item.texture, x, y);

            if i == self.selected {
                let my = y + item.texture.height / 2;
                ctx.screen.draw_texture(&self.marker_left, x.saturating_sub(3), my);
                ctx.screen.draw_texture(&self.marker_right, x + item.texture.width + 2, my);
            }
            y += item.texture.height + gap;
        }
    }
}