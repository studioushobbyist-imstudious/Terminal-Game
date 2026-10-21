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
    fn new(w: u32, h: u32, trans_char: char) -> Self{
        Texture { 
            trans: trans_char, 
            width: w, 
            height: h, 
            grid: vec![' '; (w as usize) * (h as usize)]
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

  