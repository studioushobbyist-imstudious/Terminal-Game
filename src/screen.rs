
use crate::texture::Texture;

use crossterm::{cursor, execute, terminal};
use std::io::stdout;
use std::io::Write;

pub struct Screen {
    pub width: u32,
    pub height: u32,
    pub grid: Vec<char>,
    pub persist_grid: Vec<char>
}

impl Screen {
    pub fn new(w: u32, h: u32) -> Self {
        Screen {
            width: w,
            height: h,
            grid: vec![' '; (w as usize) * (h as usize)],
            persist_grid: vec![' '; (w as usize) * (h as usize)]
        }
    }

    pub fn draw(&self) -> std::io::Result<()>{
        for y in 0..self.height {
            let start = y as usize* self.width as usize;
            let end = start + self.width as usize;
            let row = &self.grid[start..end];

            let line: String = row.iter().collect();
            if y != self.height-1{
                print!("{line}\r\n");
            }
            else{
                print!("{line}");
            }
        }
        stdout().flush()?;
        Ok(())
    }
    /// # Overlay
    pub fn write_to(&mut self, character: char, x: u32, y: u32,){
        if x < self.width && y < self.height{
            let index = x as usize + self.width as usize * y as usize;
            self.grid[index] = character;
        }
    }
    /// # BackGround
    pub fn persist_write_to(&mut self, character: char, x: u32, y: u32,){
        if x < self.width && y < self.height{
            let index = x as usize + self.width as usize * y as usize;
            self.persist_grid[index] = character;
        }
    }

    pub fn draw_texture(&mut self, texture: &Texture, x: u32, y: u32){
        let texture_width = texture.width as usize;
        let texture_height = texture.height as usize;
        let texture_size = texture_width * texture_height;
        for i in 0..texture_size{
            let character = texture.grid[i as usize];

            if character == texture.trans {continue}

            self.write_to(
                character,
                (x as usize + i % texture.width as usize) as u32, 
                (y as usize+ i / texture.width as usize) as u32
            );
        }
    }
    pub fn draw_texture_scaled(&mut self, texture: &Texture, x: u32, y: u32, s: f32) {
        if !(s > 0.0) || texture.width == 0 || texture.height == 0 {
            return;
        }

        let tw = texture.width as usize;
        let th = texture.height as usize;

        // size of the scaled image, in cells
        let dest_w = (tw as f32 * s).round() as usize;
        let dest_h = (th as f32 * s).round() as usize;

        // only loop over cells that can actually land on screen
        let dest_w = dest_w.min((self.width as usize).saturating_sub(x as usize));
        let dest_h = dest_h.min((self.height as usize).saturating_sub(y as usize));

        for dy in 0..dest_h {
            for dx in 0..dest_w {
                // which source cell does this destination cell come from?
                let sx = (((dx as f32 + 0.5) / s) as usize).min(tw - 1);
                let sy = (((dy as f32 + 0.5) / s) as usize).min(th - 1);

                let character = texture.grid[sy * tw + sx];
                if character == texture.trans {
                    continue;
                }

                self.write_to(character, x + dx as u32, y + dy as u32);
            }
        }
    }
    pub fn draw_line(&mut self,character: char,x1: u32,y1: u32,x2: u32,y2: u32,) {
        let mut x1 = x1 as i32;
        let mut y1 = y1 as i32;

        let x2 = x2 as i32;
        let y2 = y2 as i32;

        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();

        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };

        let mut error = dx - dy;

        loop {
            // Draw the character if the point is inside the screen.
            if x1 >= 0 && y1 >= 0 && (x1 as u32) < self.width && (y1 as u32) < self.height{
                self.write_to(character, x1 as u32, y1 as u32);
            }

            // Stop after drawing the endpoint.
            if x1 == x2 && y1 == y2 {
                break;
            }

            let error2 = 2 * error;

            if error2 > -dy {
                error -= dy;
                x1 += sx;
            }

            if error2 < dx {
                error += dx;
                y1 += sy;
            }
        }
    }
    pub fn fill(&mut self, character: char) {
        self.grid.fill(character);
    }
    pub fn draw_outline_rectangle(&mut self,character: char,x: u32,y: u32,w: u32,h: u32) {
        if w == 0 || h == 0 {
            return;
        }

        let right = x.saturating_add(w - 1);
        let bottom = y.saturating_add(h - 1);

        self.draw_line(character, x, y, right, y);
        self.draw_line(character, x, bottom, right, bottom);
        self.draw_line(character, x, y, x, bottom);
        self.draw_line(character, right, y, right, bottom);
    }
    pub fn draw_fill_rectangle(&mut self, character: char, x: u32, y: u32, w: u32, h: u32) {
        let x_end = x.saturating_add(w).min(self.width);
        let y_end = y.saturating_add(h).min(self.height);
        if x >= x_end || y >= y_end {
            return; // required here: an inverted range would panic when slicing
        }

        let tw = self.width as usize;
        for py in y as usize..y_end as usize {
            self.grid[py * tw + x as usize..py * tw + x_end as usize].fill(character);
        }
    }

    
    pub fn persist_draw_texture(&mut self, texture: &Texture, x: u32, y: u32){
        let texture_width = texture.width as usize;
        let texture_height = texture.height as usize;
        let texture_size = texture_width * texture_height;
        for i in 0..texture_size{
            let character = texture.grid[i as usize];

            if character == texture.trans {continue}

            self.persist_write_to(
                character,
                (x as usize + i % texture.width as usize) as u32, 
                (y as usize+ i / texture.width as usize) as u32
            );
        }
    }
    pub fn persist_draw_texture_scaled(&mut self, texture: &Texture, x: u32, y: u32, s: f32) {
        if !(s > 0.0) || texture.width == 0 || texture.height == 0 {
            return;
        }

        let tw = texture.width as usize;
        let th = texture.height as usize;

        // size of the scaled image, in cells
        let dest_w = (tw as f32 * s).round() as usize;
        let dest_h = (th as f32 * s).round() as usize;

        // only loop over cells that can actually land on screen
        let dest_w = dest_w.min((self.width as usize).saturating_sub(x as usize));
        let dest_h = dest_h.min((self.height as usize).saturating_sub(y as usize));

        for dy in 0..dest_h {
            for dx in 0..dest_w {
                // which source cell does this destination cell come from?
                let sx = (((dx as f32 + 0.5) / s) as usize).min(tw - 1);
                let sy = (((dy as f32 + 0.5) / s) as usize).min(th - 1);

                let character = texture.grid[sy * tw + sx];
                if character == texture.trans {
                    continue;
                }

                self.persist_write_to(character, x + dx as u32, y + dy as u32);
            }
        }
    }
    pub fn persist_draw_line(&mut self,character: char,x1: u32,y1: u32,x2: u32,y2: u32,) {
        let mut x1 = x1 as i32;
        let mut y1 = y1 as i32;

        let x2 = x2 as i32;
        let y2 = y2 as i32;

        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();

        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };

        let mut error = dx - dy;

        loop {
            // Draw the character if the point is inside the screen.
            if x1 >= 0 && y1 >= 0 && (x1 as u32) < self.width && (y1 as u32) < self.height{
                self.persist_write_to(character, x1 as u32, y1 as u32);
            }

            // Stop after drawing the endpoint.
            if x1 == x2 && y1 == y2 {
                break;
            }

            let error2 = 2 * error;

            if error2 > -dy {
                error -= dy;
                x1 += sx;
            }

            if error2 < dx {
                error += dx;
                y1 += sy;
            }
        }
    }
    pub fn persist_fill(&mut self, character: char) {
        self.persist_grid.fill(character);
    }
    pub fn persist_draw_outline_rectangle(&mut self,character: char,x: u32,y: u32,w: u32,h: u32) {
        if w == 0 || h == 0 {
            return;
        }

        let right = x.saturating_add(w - 1);
        let bottom = y.saturating_add(h - 1);

        self.persist_draw_line(character, x, y, right, y);
        self.persist_draw_line(character, x, bottom, right, bottom);
        self.persist_draw_line(character, x, y, x, bottom);
        self.persist_draw_line(character, right, y, right, bottom);
    }
    pub fn persist_draw_fill_rectangle(&mut self, character: char, x: u32, y: u32, w: u32, h: u32) {
        let x_end = x.saturating_add(w).min(self.width);
        let y_end = y.saturating_add(h).min(self.height);
        if x >= x_end || y >= y_end {
            return; // required here: an inverted range would panic when slicing
        }

        let tw = self.width as usize;
        for py in y as usize..y_end as usize {
            self.persist_grid[py * tw + x as usize..py * tw + x_end as usize].fill(character);
        }
    }

    
    pub fn push_back_cursor(&self) -> std::io::Result<()>{
        execute!(stdout(), cursor::MoveTo(0, 0))?;
        Ok(())
    }
    pub fn clear_terminal(&self) -> std::io::Result<()>{
        execute!(stdout(), terminal::Clear(terminal::ClearType::All))?;
        Ok(())
    }
    pub fn clear_grid(&mut self) {
        self.grid.copy_from_slice(&self.persist_grid);
    }
    pub fn clear_persist_grid(&mut self){
        self.persist_grid = vec![' '; self.width as usize * self.height as usize];
    }
    pub fn resize(&mut self, w: u32, h: u32) {
        self.width = w;
        self.height = h;
        self.grid = vec![' '; w as usize * h as usize];
        self.persist_grid = vec![' '; w as usize * h as usize];
    }

}
