use std::fs;
use std::path::Path;

/// # Terminal Texture
/// ascii art for terminal
pub struct Texture {
    pub trans: char, // character treated as transparancy character
    pub width: u32,
    pub height: u32,
    pub grid: Vec<char>
}

impl Texture {
    pub fn new(w: u32, h: u32, trans_char: char) -> Self{
        Texture { 
            trans: trans_char, 
            width: w, 
            height: h, 
            grid: vec![' '; (w as usize) * (h as usize)]
        }
    }
    pub fn write_to(&mut self, character: char, x: u32, y: u32){
        if x < self.width && y < self.height{
            let index = x as usize + self.width as usize * y as usize;
            self.grid[index] = character;
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

}

pub fn texture_make_text(text: &str, trans: char) -> Texture {
    let width = text.chars().count() as u32;
    let height = 1;

    Texture {
        trans,
        width,
        height,
        grid: text.chars().collect(),
    }
}

pub fn texture_make_circle(width: u32, height: u32, char1: char, char2: char) -> Texture {
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

pub fn parse_texture_char(value: &str) -> Result<char, String> {
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

pub fn texture_parse(input: &str) -> Result<Texture, String> {
    let mut width: Option<u32> = None;
    let mut height: Option<u32> = None;
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
                let n: u32 = value
                    .parse()
                    .map_err(|_| format!("Invalid width: {value}"))?;

                if !(1..=256).contains(&n) {
                    return Err("Width must be between 1 and 256".into());
                }

                width = Some(n);
            }

            "height" => {
                let n: u32 = value
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

pub fn texture_load(path: impl AsRef<Path>) -> Result<Texture, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read texture: {e}"))?;

    texture_parse(&content)
}

  