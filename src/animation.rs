

#[derive(Clone, Copy)]
pub enum Easing {
    Linear,
    EaseIn,      // starts slow, ends fast
    EaseOut,     // starts fast, ends slow
    EaseInOut,   // slow at both ends
    SineInOut,   // like EaseInOut, a bit gentler
}

impl Easing {
    pub fn apply(self, t: f32) -> f32 {
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            Easing::EaseInOut => t * t * (3.0 - 2.0 * t),
            Easing::SineInOut => 0.5 - 0.5 * (std::f32::consts::PI * t).cos(),
        }
    }
}

pub struct Timeline { pub frame: u32, pub frames: u32, pub easing: Easing, pub resets: u32 }

impl Timeline {
    pub fn new(frames: u32, easing: Easing) -> Self {
        Timeline { frame: 0, frames, easing, resets: 0 }
    }
    

    pub fn progress(&self) -> f32 {
        (self.frame as f32 / self.frames.max(1) as f32).clamp(0.0, 1.0)
    }
    pub fn t(&self) -> f32 {
        self.easing.apply(self.progress())
    }
    pub fn done(&self) -> bool { self.frame >= self.frames }
    pub fn tick(&mut self) { self.frame = self.frame.saturating_add(1); }
    pub fn reset(&mut self){self.frame = 0; self.resets = self.resets.saturating_add(1)}
}

pub trait Tick {
    fn tick(&mut self);
    fn finished(&self) -> bool { false }
}

pub struct Ctx { pub x: u32, pub y: u32, pub width: u32, pub height: u32 }

pub trait TextureAnimation: Tick {
    fn visible(&self, _ctx: &Ctx) -> bool { true }
    fn map_char(&self, ch: char, _ctx: &Ctx) -> char { ch }
    fn offset(&self, _ctx: &Ctx) -> (i32, i32) { (0, 0) }

    fn transform(&self, ch: char, ctx: &Ctx) -> Option<(char, i32, i32)> {
        if !self.visible(ctx) { return None; }
        let (dx, dy) = self.offset(ctx);
        Some((self.map_char(ch, ctx), ctx.x as i32 + dx, ctx.y as i32 + dy))
    }
}

pub trait Field {
    fn value(&self, ctx: &Ctx) -> f32; // 0.0 up to, but not including, 1.0
}

pub struct Wipe<F: Field> { pub field: F, pub timeline: Timeline }

impl<F: Field> Tick for Wipe<F> {
    fn tick(&mut self) {
        self.timeline.tick();
    }

    fn finished(&self) -> bool {
        self.timeline.done()
    }
}

impl<F: Field> TextureAnimation for Wipe<F> {
    fn visible(&self, ctx: &Ctx) -> bool {
        self.field.value(ctx) < self.timeline.t()
    }
}
// (plus the three-line Tick impl that forwards to the timeline)

pub struct Horizontal;
impl Field for Horizontal {
    fn value(&self, ctx: &Ctx) -> f32 {
        ctx.x as f32 / ctx.width as f32          // 0.0 at the left edge, just under 1.0 at the right
    }
}

pub struct Vertical;
impl Field for Vertical {
    fn value(&self, ctx: &Ctx) -> f32 {
        ctx.y as f32 / ctx.height as f32
    }
}

pub struct Checker { pub size: u32}
impl Field for Checker {
    fn value(&self, ctx: &Ctx) -> f32 {
        let s = self.size.max(1);
        if (ctx.x / s + ctx.y / s) % 2 == 0 { 0.0 } else { 0.5 }
    }
}

pub struct Bayer4;
impl Field for Bayer4 {
    fn value(&self, ctx: &Ctx) -> f32 {
        const M: [[u8; 4]; 4] = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];
        M[(ctx.y % 4) as usize][(ctx.x % 4) as usize] as f32 / 16.0
    }
}

pub struct Dither;
impl Field for Dither {
    fn value(&self, ctx: &Ctx) -> f32 {
        ctx.y as f32 / ctx.height as f32
    }
}


#[derive(Clone, Copy)]
pub struct Phase {
    pub duration: u32,   // frames
    pub easing: Easing,
    pub from: f32,       // wipe progress at the start of the phase
    pub to: f32,         // ...and at the end
}

impl Phase {
    pub fn new(duration: u32, easing: Easing, from: f32, to: f32) -> Self {
        Phase { duration, easing, from, to }
    }
    pub fn forward(duration: u32, easing: Easing) -> Self { Phase::new(duration, easing, 0.0, 1.0) }
    pub fn backward(duration: u32, easing: Easing) -> Self { Phase::new(duration, easing, 1.0, 0.0) }
    pub fn hold(duration: u32, at: f32) -> Self { Phase::new(duration, Easing::Linear, at, at) }
}

pub struct TextureAnimationSequence<F: Field> {
    pub field: F,
    pub phases: Vec<Phase>,
    pub looping: bool,
    phase: usize,
    frame: u32,
}

impl<F: Field> TextureAnimationSequence<F> {
    pub fn new(field: F, phases: Vec<Phase>) -> Self {
        TextureAnimationSequence { field, phases, looping: false, phase: 0, frame: 0 }
    }

    pub fn with_loop(mut self) -> Self {
        self.looping = true;
        self
    }

    pub fn reset(&mut self) {
        self.phase = 0;
        self.frame = 0;
    }

    /// current wipe progress, 0.0..=1.0
    pub fn t(&self) -> f32 {
        match self.phases.get(self.phase) {
            Some(p) => {
                let progress = (self.frame as f32 / p.duration.max(1) as f32).clamp(0.0, 1.0);
                p.from + (p.to - p.from) * p.easing.apply(progress)
            }
            // every phase is over: stay where the last one ended
            None => self.phases.last().map_or(1.0, |p| p.to),
        }
    }
}

impl<F: Field> Tick for TextureAnimationSequence<F> {
    fn tick(&mut self) {
        let Some(p) = self.phases.get(self.phase) else { return };
        let duration = p.duration.max(1);

        self.frame = self.frame.saturating_add(1);
        if self.frame >= duration {
            self.frame = 0;
            self.phase += 1;
            if self.looping && self.phase >= self.phases.len() {
                self.phase = 0;
            }
        }
    }

    fn finished(&self) -> bool {
        !self.looping && self.phase >= self.phases.len()
    }
}

impl<F: Field> TextureAnimation for TextureAnimationSequence<F> {
    fn visible(&self, ctx: &Ctx) -> bool {
        self.field.value(ctx) < self.t()
    }
}
