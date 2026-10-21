
use crossterm::{cursor, execute, terminal};
use std::io::stdout;

/// # Terminal Texture
/// ascii art for terminal
struct Texture {
    empty: char, // character treated as alpha character
    width: u16,
    height: u16,
    grid: Vec<char>
}

impl Texture {
    fn new(w: u16, h: u16, empty_character: char) -> Self{
        Texture { 
            empty: empty_character, 
            width: w, 
            height: h, 
            grid: vec!['.'; (w as usize) * (h as usize)]
        }
    }
}

struct Screen {
    width: u16,
    height: u16,
    grid: Vec<char>
}

impl Screen {
    fn new(w: u16, h: u16) -> Self {
        Screen {
            width: w,
            height: h,
            grid: vec![' '; (w as usize) * (h as usize)],
        }
    }

    fn draw(&self){
        for y in 0..self.height {
            let start = y as usize* self.width as usize;
            let end = start + self.width as usize;
            let row = &self.grid[start..end];

            let line: String = row.iter().collect();
            if y != self.height-1{
                println!("{line}");
            }
            else{
                print!("{line}");
            }
            
        }
    }

    fn write_to(&mut self, character: char, x: u16, y: u16,){
        if x < self.width && y < self.height{
            let index = x as usize + self.width as usize * y as usize;
            self.grid[index] = character;
        }
    }

    fn draw_texture(&mut self, texture: &Texture, x: u16, y: u16){
        let texture_size = texture.width * texture.height;
        for i in 0..texture_size{
            self.write_to(
                texture.grid[i as usize],
                x + i % texture.width, 
                y + i / texture.width
            );
        }
    }

    fn push_back_cursor(&self) -> std::io::Result<()>{
        execute!(stdout(), cursor::MoveTo(0, 0))?;
        Ok(())
    }

    fn clear_terminal(&self) -> std::io::Result<()>{
        execute!(stdout(), terminal::Clear(terminal::ClearType::All))?;
        Ok(())
    }

    fn clear_grid(&mut self){
        self.grid = vec![' '; (self.width as usize) * (self.height as usize)]
    }

}

fn main() -> std::io::Result<()> {
    
    let (width, height) = terminal::size()?;
    println!("Terminal width:  {width} columns");
    println!("Terminal height: {height} rows");

    let mut screen = Screen::new(width, height);
    let texture0 = Texture::new(2,2,' ');

    let mut a = 0;

    execute!(stdout(), cursor::Hide)?;

    screen.clear_terminal()?;
    loop{
        screen.clear_grid();
        screen.push_back_cursor()?;
        
        screen.grid[a] = '.';
        screen.draw_texture(&texture0, a as u16 % width, a as u16 / width);

        screen.draw();
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        a += 1;
    }
}