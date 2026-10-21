use crate::texture::{parse_texture_char, Texture};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// A single glyph in a font.
pub struct Glyph {
    pub texture: Texture,
    pub spacing: u32,
}

/// A font containing characters, modes, and glyph sizes.
pub struct Font {
    pub name: String,
    pub trans: char,

    // Character -> mode -> available glyph sizes
    pub characters: HashMap<char, HashMap<String, Vec<Glyph>>>,
}

impl Font {
    pub fn new(name: String, trans: char) -> Self {
        Self {
            name,
            trans,
            characters: HashMap::new(),
        }
    }

    /// Add a glyph, rejecting duplicate character/mode/size combinations.
    pub fn add_glyph(
        &mut self,
        character: char,
        mode: String,
        glyph: Glyph,
    ) -> Result<(), String> {
        let sizes = self.characters
            .entry(character)
            .or_default()
            .entry(mode)
            .or_default();

        let width = glyph.texture.width;
        let height = glyph.texture.height;

        if sizes.iter().any(|existing| {
            existing.texture.width == width
                && existing.texture.height == height
        }) {
            return Err(format!(
                "Duplicate glyph: {character:?}, size {width}x{height}"
            ));
        }

        sizes.push(glyph);
        Ok(())
    }

    /// Find the glyph whose dimensions are closest to the requested size.
    pub fn get_glyph(
        &self,
        character: char,
        mode: &str,
        requested_width: u32,
        requested_height: u32,
    ) -> Option<&Glyph> {
        let modes = self.characters.get(&character)?;
        let sizes = modes.get(mode)?;

        sizes.iter().min_by_key(|glyph| {
            let dw = glyph.texture.width as i64
                - requested_width as i64;
            let dh = glyph.texture.height as i64
                - requested_height as i64;

            (dw * dw + dh * dh) as u64
        })
    }

    /// Render a string into a texture.
    ///
    /// Glyphs are top-aligned. Later glyphs overwrite earlier glyphs
    /// wherever their non-transparent characters overlap.
    pub fn render_text(
        &self,
        text: &str,
        mode: &str,
        requested_width: u32,
        requested_height: u32,
    ) -> Result<Texture, String> {
        let mut selected: Vec<(&Glyph, u32)> = Vec::new();
        let mut x = 0u32;
        let mut output_width = 0u32;
        let mut output_height = 0u32;

        // Select glyphs and calculate the required texture dimensions.
        for character in text.chars() {
            let glyph = self.get_glyph(
                character,
                mode,
                requested_width,
                requested_height,
            ).ok_or_else(|| {
                format!(
                    "No glyph for character {character:?} in mode {mode:?}"
                )
            })?;

            let right = x.checked_add(glyph.texture.width)
                .ok_or("Rendered text width overflow")?;

            output_width = output_width.max(right);
            output_height = output_height.max(glyph.texture.height);

            selected.push((glyph, x));

            // Spacing is the total horizontal advance.
            x = x.checked_add(glyph.spacing)
                .ok_or("Rendered text advance overflow")?;
        }

        // Create a transparent canvas.
        let area = (output_width as usize)
            .checked_mul(output_height as usize)
            .ok_or("Rendered texture size overflow")?;

        let mut output = Texture {
            apply_trans: true,
            trans: self.trans,
            width: output_width,
            height: output_height,
            grid: vec![self.trans; area],
        };

        // Composite each glyph onto the canvas.
        for (glyph, offset_x) in selected {
            for gy in 0..glyph.texture.height {
                for gx in 0..glyph.texture.width {
                    let source_index =
                        gy as usize * glyph.texture.width as usize
                            + gx as usize;

                    let character = glyph.texture.grid[source_index];

                    // Transparent glyph characters do not overwrite
                    // characters already drawn underneath.
                    if character == glyph.texture.trans {
                        continue;
                    }

                    let target_x = offset_x + gx;
                    let target_y = gy;

                    let target_index =
                        target_y as usize * output.width as usize
                            + target_x as usize;

                    output.grid[target_index] = character;
                }
            }
        }

        Ok(output)
    }
}

/// Parse a font from its file contents.
pub fn font_parse(input: &str) -> Result<Font, String> {
    let mut trans: Option<char> = None;
    let mut font: Option<Font> = None;

    let mut current_char: Option<char> = None;
    let mut current_mode: Option<String> = None;
    let mut current_size: Option<(u32, u32)> = None;
    let mut current_spacing: Option<u32> = None;

    let mut lines = input.lines().enumerate();

    while let Some((line_index, raw_line)) = lines.next() {
        // Trim metadata, never glyph artwork.
        let line = raw_line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (key, value) = line
            .split_once(char::is_whitespace)
            .ok_or_else(|| {
                format!(
                    "Invalid font declaration on line {}: {line}",
                    line_index + 1
                )
            })?;

        let value = value.trim();

        match key {
            "transparent" => {
                if trans.is_some() {
                    return Err(
                        "Duplicate transparent declaration".into()
                    );
                }

                let character = parse_texture_char(value)?;
                trans = Some(character);

                // The filename will be assigned by font_load().
                font = Some(Font::new(String::new(), character));
            }

            "char" => {
                current_char = Some(parse_texture_char(value)?);
                current_mode = None;
                current_size = None;
                current_spacing = None;
            }

            "mode" => {
                if current_char.is_none() {
                    return Err(
                        "Mode declared before character".into()
                    );
                }

                if value.is_empty() {
                    return Err("Empty font mode".into());
                }

                current_mode = Some(value.to_string());
                current_size = None;
                current_spacing = None;
            }

            "size" => {
                if current_char.is_none() || current_mode.is_none() {
                    return Err(
                        "Size declared before character or mode".into()
                    );
                }

                let (w, h) = value
                    .split_once('x')
                    .ok_or_else(|| {
                        format!("Invalid glyph size: {value}")
                    })?;

                let width: u32 = w.trim()
                    .parse()
                    .map_err(|_| {
                        format!("Invalid glyph width: {w}")
                    })?;

                let height: u32 = h.trim()
                    .parse()
                    .map_err(|_| {
                        format!("Invalid glyph height: {h}")
                    })?;

                if width == 0 || height == 0 {
                    return Err(
                        "Glyph dimensions must be positive".into()
                    );
                }

                // Prevent accidentally allocating enormous glyphs.
                if width > 65_535 || height > 65_535 {
                    return Err(
                        "Glyph dimensions cannot exceed 65535".into()
                    );
                }

                let area = (width as usize)
                    .checked_mul(height as usize)
                    .ok_or("Glyph dimensions overflow")?;

                if area > 1_000_000 {
                    return Err(
                        "Glyph exceeds the 1,000,000-character limit".into()
                    );
                }

                current_size = Some((width, height));
                current_spacing = None;
            }

            "spacing" => {
                if current_size.is_none() {
                    return Err(
                        "Spacing declared before glyph size".into()
                    );
                }

                current_spacing = Some(
                    value.parse::<u32>()
                        .map_err(|_| {
                            format!("Invalid glyph spacing: {value}")
                        })?
                );
            }

            "glyph" => {
                let character = current_char
                    .ok_or("Glyph declared before character")?;

                let mode = current_mode
                    .clone()
                    .ok_or("Glyph declared before mode")?;

                let (width, height) = current_size
                    .ok_or("Glyph declared before size")?;

                let spacing = current_spacing
                    .ok_or("Glyph declared before spacing")?;

                let mut grid = Vec::with_capacity(
                    width as usize * height as usize
                );

                // Read exactly the declared number of artwork rows.
                for row in 0..height {
                    let (_, artwork_line) = lines.next()
                        .ok_or_else(|| {
                            format!(
                                "Unexpected end of file in glyph \
                                 {character:?}: expected {height} rows, \
                                 got {row}"
                            )
                        })?;

                    let chars: Vec<char> =
                        artwork_line.chars().collect();

                    if chars.len() != width as usize {
                        return Err(format!(
                            "Glyph {character:?}, row {} has {} \
                             characters; expected {width}",
                            row + 1,
                            chars.len()
                        ));
                    }

                    grid.extend(chars);
                }

                let texture = Texture {
                    apply_trans: true,
                    trans: trans.ok_or(
                        "Missing transparent declaration"
                    )?,
                    width,
                    height,
                    grid,
                };

                let glyph = Glyph { texture, spacing };

                font.as_mut()
                    .ok_or("Missing transparent declaration")?
                    .add_glyph(character, mode, glyph)?;

                // Require a fresh size and spacing for the next glyph.
                current_size = None;
                current_spacing = None;
            }

            _ => {
                return Err(format!(
                    "Unknown font declaration on line {}: {key}",
                    line_index + 1
                ));
            }
        }
    }

    let font = font.ok_or("Missing transparent declaration")?;

    if font.characters.is_empty() {
        return Err("Font contains no glyphs".into());
    }

    Ok(font)
}

/// Load a font from a .af file.
pub fn font_load(path: impl AsRef<Path>) -> Result<Font, String> {
    let path = path.as_ref();

    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read font: {e}"))?;

    let mut font = font_parse(&content)?;

    font.name = path
        .file_stem()
        .and_then(|name| name.to_str())
        .ok_or("Invalid font filename")?
        .to_string();

    Ok(font)
}