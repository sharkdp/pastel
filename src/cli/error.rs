#[derive(Debug, PartialEq)]
pub enum PastelError {
    ColorParseError(String),
    ColorInvalidUTF8,
    CouldNotReadFromStdin,
    ColorArgRequired,
    CouldNotParseNumber(String),
}

impl PastelError {
    pub fn message(&self) -> String {
        match self {
            PastelError::ColorParseError(color) => format!("Could not parse color '{}'", color),
            PastelError::ColorInvalidUTF8 => "Color input contains invalid UTF8".into(),
            PastelError::CouldNotReadFromStdin => "could not read color from standard input".into(),
            PastelError::ColorArgRequired => {
                "A color argument needs to be provided on the command line or via a pipe. \
                 Call the same command with '-h' or '--help' to get more information."
                    .into()
            }
            PastelError::CouldNotParseNumber(number) => {
                format!("Could not parse number '{}'", number)
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, PastelError>;
