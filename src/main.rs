
use crossterm::{cursor, execute, terminal};
use std::io::stdout;
use std::io::Write;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use std::time::{Duration, Instant};

use std::fs;
use std::path::Path;
// TODO: draw text via n x 1 texture
// Allow ascii text art textures for different sizes via a font based file

/// # Terminal Texture
/// ascii art for terminal
struct Texture {
    trans: char, // character treated as transparancy character
    width: u16,
    height: u16,
    grid: Vec<char>
}

impl Texture {
    fn new(w: u16, h: u16, trans_char: char) -> Self{
        Texture { 
            trans: trans_char, 
            width: w, 
            height: h, 
            grid: vec![' '; (w as usize) * (h as usize)]
        }
    }
}

fn texture_make_text(text: &str, trans: char) -> Texture {
    let width = text.chars().count() as u16;
    let height = 1;

    Texture {
        trans,
        width,
        height,
        grid: text.chars().collect(),
    }
}

fn texture_make_circle(width: u16, height: u16, char1: char, char2: char) -> Texture {
    let mut texture = Texture::new(width, height, ' ');

    let cx = (width as f32 - 1.0) / 2.0;
    let cy = (height as f32 - 1.0) / 2.0;
    let radius = width as f32 / 4.0;

    for y in 0..height {
        for x in 0..width {
            let dx = (x as f32 - cx) / 1.5;
            let dy = (y as f32 - cy) *1.4;

            let distance = (dx * dx + dy * dy).sqrt();

            let index = (y as usize) * (width as usize) + (x as usize);

            if distance <= radius {
                texture.grid[index] = char1;
            } else {
                texture.grid[index] = char2;
            }
        }
    }

    texture
}

fn parse_texture_char(value: &str) -> Result<char, String> {
    let value = value.trim();

    // Unicode code point, e.g. U+2800
    if let Some(hex) = value.strip_prefix("U+") {
        let code = u32::from_str_radix(hex, 16)
            .map_err(|_| format!("Invalid Unicode code point: {value}"))?;

        return char::from_u32(code)
            .ok_or_else(|| format!("Invalid Unicode scalar value: {value}"));
    }

    // Quoted character, e.g. '@' or ' '
    if value.starts_with('\'') && value.ends_with('\'') {
        let inner = &value[1..value.len() - 1];

        let character = match inner {
            "\\n" => '\n',
            "\\r" => '\r',
            "\\t" => '\t',
            "\\0" => '\0',
            "\\\\" => '\\',
            "\\'" => '\'',
            _ => {
                let mut chars = inner.chars();

                let ch = chars.next()
                    .ok_or("Empty character literal")?;

                if chars.next().is_some() {
                    return Err(format!(
                        "Expected exactly one character: {value}"
                    ));
                }

                ch
            }
        };

        return Ok(character);
    }

    Err(format!(
        "Expected a Unicode code point or quoted character, got: {value}"
    ))
}

fn texture_parse(input: &str) -> Result<Texture, String> {
    let mut width: Option<u16> = None;
    let mut height: Option<u16> = None;
    let mut trans: Option<char> = None;

    let mut lines = input.lines();
    let mut body_started = false;
    let mut body_lines: Vec<&str> = Vec::new();

    for line in lines.by_ref() {
        let line = line.trim();

        if line == "body" {
            body_started = true;
            break;
        }

        // Ignore empty lines and comments in the header.
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (key, value) = line
            .split_once(char::is_whitespace)
            .ok_or_else(|| format!("Invalid header line: {line}"))?;

        let value = value.trim();

        match key {
            "width" => {
                let n: u16 = value
                    .parse()
                    .map_err(|_| format!("Invalid width: {value}"))?;

                if !(1..=256).contains(&n) {
                    return Err("Width must be between 1 and 256".into());
                }

                width = Some(n);
            }

            "height" => {
                let n: u16 = value
                    .parse()
                    .map_err(|_| format!("Invalid height: {value}"))?;

                if !(1..=256).contains(&n) {
                    return Err("Height must be between 1 and 256".into());
                }

                height = Some(n);
            }

            "transparent" => {
                trans = Some(parse_texture_char(value)?);
            }

            _ => {
                return Err(format!("Unknown header field: {key}"));
            }
        }
    }

    if !body_started {
        return Err("Missing body marker".into());
    }

    let width = width.ok_or("Missing width")?;
    let height = height.ok_or("Missing height")?;
    let trans = trans.ok_or("Missing transparent character")?;

    body_lines.extend(lines);

    if body_lines.len() != height as usize {
        return Err(format!(
            "Expected {height} rows, found {}",
            body_lines.len()
        ));
    }

    let mut grid = Vec::with_capacity(
        width as usize * height as usize
    );

    for (y, line) in body_lines.iter().enumerate() {
        let chars: Vec<char> = line.chars().collect();

        if chars.len() != width as usize {
            return Err(format!(
                "Row {} has {} characters; expected {}",
                y + 1,
                chars.len(),
                width
            ));
        }

        grid.extend(chars);
    }

    Ok(Texture {
        trans,
        width,
        height,
        grid,
    })
}

fn texture_load(path: impl AsRef<Path>) -> Result<Texture, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read texture: {e}"))?;

    texture_parse(&content)
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

    fn draw(&self) -> std::io::Result<()>{
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

    fn write_to(&mut self, character: char, x: u16, y: u16,){
        if x < self.width && y < self.height{
            let index = x as usize + self.width as usize * y as usize;
            self.grid[index] = character;
        }
    }

    fn draw_texture(&mut self, texture: &Texture, x: u16, y: u16){
        let texture_width = texture.width as usize;
        let texture_height = texture.height as usize;
        let texture_size = texture_width * texture_height;
        for i in 0..texture_size{
            let character = texture.grid[i as usize];

            if character == texture.trans {continue}

            self.write_to(
                character,
                (x as usize + i % texture.width as usize) as u16, 
                (y as usize+ i / texture.width as usize) as u16
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

    fn resize(&mut self, w: u16, h: u16) {
        self.width = w;
        self.height = h;
        self.grid = vec![' '; w as usize * h as usize];
    }
}

struct Ball {
    x: i32,
    y: i32,
    vx: i32, // columns per frame
    vy: i32, // rows per frame
    texture: Texture,
}

impl Ball {
    fn new(x: i32, y: i32, vx: i32, vy: i32, texture: Texture) -> Self {
        Ball { x, y, vx, vy, texture }
    }

    fn update(&mut self, screen_w: u16, screen_h: u16) {
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

    fn draw(&self, screen: &mut Screen) {
        screen.draw_texture(&self.texture, self.x as u16, self.y as u16);
    }
}
fn main() -> std::io::Result<()> {
    let (width, height) = terminal::size()?;

    let mut fps_timer = Instant::now();
    let mut frames = 0u32;
    let mut fps = 0u32;

    let mut screen = Screen::new(width, height);
    let mut ball = Ball::new(0, 0, 2, 1, texture_make_circle(12, 6, '@', ' '));
    let swag_texture = texture_load("assets/SwagTest.at")
        .map_err(std::io::Error::other)?;
    terminal::enable_raw_mode()?;
    execute!(stdout(), cursor::Hide)?;
    screen.clear_terminal()?;

    loop {
        let frame_start = Instant::now();
        let screen_width = screen.width;
        let screen_height = screen.height;
        let t1 = texture_make_text(&format!("Terminal width:  {screen_width} columns"), '\n');
        let t2 = texture_make_text(&format!("Terminal height: {screen_height} rows"), '\n');
        let t3 = texture_make_text(&format!("FPS: {fps}"), '\n'); // change 1: FPS text

        screen.clear_grid();

        screen.draw_texture(&swag_texture, screen_width/2-swag_texture.width/2, screen_height/2-swag_texture.height/2);
        ball.update(screen_width, screen_height);
        ball.draw(&mut screen);

        screen.draw_texture(&t1, 0, 0);
        screen.draw_texture(&t2, 0, 1);
        screen.draw_texture(&t3, 0, 2); 

        screen.push_back_cursor()?;
        screen.draw()?;

        // change 2: count frames, publish the number once per second
        frames += 1;
        if fps_timer.elapsed() >= Duration::from_secs(1) {
            fps = frames;
            frames = 0;
            fps_timer = Instant::now();
        }

        let target = Duration::from_millis(33);
        let remaining = target.saturating_sub(frame_start.elapsed());
        if event::poll(remaining)? { // change 3: use `remaining`, not a fresh 33 ms
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if key.code == KeyCode::Char('q') {
                        break;
                    }
                    if key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL) {
                        break;
                    }
                }
                Event::Resize(w, h) => {
                    screen.resize(w, h);
                    screen.clear_terminal()?;
                }
                _ => {}
            }
        }
    }

    execute!(stdout(), cursor::Show)?;
    terminal::disable_raw_mode()?;
    Ok(())
}