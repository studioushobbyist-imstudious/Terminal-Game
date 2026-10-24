pub struct Settings{
    pub debug: bool,
    pub fps: u64
}

impl Settings{
    pub fn new() -> Self{
        Settings { 
            debug: false,
            fps: 60
        }
    }
}