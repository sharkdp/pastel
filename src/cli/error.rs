#[derive(Debug)]
pub enum PastelError {
    UnknownColorMode(String),
    ColorParseError(String),
    ColorInvalidUTF8,
    CouldNotReadFromStdin,
    ColorArgRequired,
    CouldNotParseNumber(String),
    StdoutClosed,
    GradientNumberMustBeLargerThanOne,
    DistinctColorCountMustBeLargerThanOne,
    ColorPickerExecutionError(String),
    NoColorPickerFound,
    IoError(std::io::Error),
}

impl PastelError {
    pub fn message(&self) -> String {
        match self {
            PastelError::UnknownColorMode(mode) => {
                format!("Unknown PASTEL_COLOR_MODE value ({})", mode)
            }
            PastelError::ColorParseError(color) => format!("Could not parse color '{}'", color),
            PastelError::ColorInvalidUTF8 => "Color input contains invalid UTF8".into(),
            PastelError::CouldNotReadFromStdin => "could not read color from standard input".into(),
            PastelError::ColorArgRequired => {
                "A color argument needs to be provided on the command line or via a pipe. \
                 Call this command again with '-h' or '--help' to get more information."
                    .into()
            }
            PastelError::CouldNotParseNumber(number) => {
                format!("Could not parse number '{}'", number)
            }
            PastelError::StdoutClosed => "Output pipe has been closed".into(),
            PastelError::GradientNumberMustBeLargerThanOne => {
                "The number of colors must be larger one".into()
            }
            PastelError::DistinctColorCountMustBeLargerThanOne => {
                "The number of colors must be larger one".into()
            }
            PastelError::ColorPickerExecutionError(name) => {
                format!("Error while running color picker '{}", name)
            }
            PastelError::NoColorPickerFound => {
                "Could not find any external color picker tool. See 'pastel pick -h' for more information.".into()
            }
            PastelError::IoError(err) => format!("I/O error: {}", err),
        }
    }
}

impl From<std::io::Error> for PastelError {
    fn from(err: std::io::Error) -> PastelError {
        match err.kind() {
            std::io::ErrorKind::BrokenPipe => PastelError::StdoutClosed,
            _ => PastelError::IoError(err),
        }
    }
}

pub type Result<T> = std::result::Result<T, PastelError>;
