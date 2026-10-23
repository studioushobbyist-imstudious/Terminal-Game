use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink};
use std::fs::File;
use std::io::BufReader;

pub struct AudioManager {
    stream: MixerDeviceSink,
}

impl AudioManager {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let stream = DeviceSinkBuilder::open_default_sink()?;
        Ok(Self { stream })
    }

    pub fn play(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let file = BufReader::new(File::open(path)?);
        let source = Decoder::try_from(file)?;

        // Add the sound to Rodio's mixer.
        // Playback continues without waiting for it to finish.
        self.stream.mixer().add(source);

        Ok(())
    }
}