use crate::texture::Texture;
use crate::screen::Screen;

pub struct Ball {
    x: i32,
    y: i32,
    vx: i32, // columns per frame
    vy: i32, // rows per frame
    texture: Texture,
}

impl Ball {
    pub fn new(x: i32, y: i32, vx: i32, vy: i32, texture: Texture) -> Self {
        Ball { x, y, vx, vy, texture }
    }

    pub fn update(&mut self, screen_w: u32, screen_h: u32) {
        self.x += self.vx;
        self.y += self.vy;

        // furthest the top-left corner can go before the texture leaves the screen
        let max_x = (screen_w as i32 - self.texture.width as i32).max(0);
        let max_y = (screen_h as i32 - self.texture.height as i32).max(0);

        if self.x < 0 {
            self.x = 0;
            self.vx = -self.vx;
        } else if self.x > max_x {
            self.x = max_x;
            self.vx = -self.vx;
        }

        if self.y < 0 {
            self.y = 0;
            self.vy = -self.vy;
        } else if self.y > max_y {
            self.y = max_y;
            self.vy = -self.vy;
        }
    }

    pub fn draw(&self, screen: &mut Screen) {
        screen.draw_texture(&self.texture, self.x as u32, self.y as u32);
    }
}
