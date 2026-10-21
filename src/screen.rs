
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
    pub fn write_to_persist_grid(&mut self, character: char, x: u32, y: u32,){
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
