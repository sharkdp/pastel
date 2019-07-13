pub struct Config {
    pub padding: usize,
    pub colorpicker_width: usize,
}

impl Config {
    pub fn new() -> Config {
        Config {
            padding: 2,
            colorpicker_width: 40,
        }
    }
}
