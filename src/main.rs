use crossterm::terminal;

struct Screen {
    width: u16,
    height: u16,
    grid: Vec<char>,
}

impl Screen {
    fn new(w: u16, h: u16) -> Self {
        Screen {
            width: w,
            height: h,
            grid: vec!['#'; (w as usize) * (h as usize)],
        }
    }

    fn draw(&self){

        for y in 0..self.height {
            let start = (y * self.width) as usize;
            let end = start + self.width as usize;
            let row = &self.grid[start..end];

            let line: String = row.iter().collect();

            println!("{line}");
        }

    }
}

fn main() -> std::io::Result<()> {
    
    let (width, height) = terminal::size()?;
    println!("Terminal width:  {width} columns");
    println!("Terminal height: {height} rows");
    let screen = Screen::new(width, height);
    screen.draw();



    Ok(())
}