#[derive(Debug, PartialEq)]
pub enum PastelError {
    ColorParseError,
    CouldNotReadFromStdin,
    ColorArgRequired,
    CouldNotParseNumber,
}

impl PastelError {
    pub fn message(&self) -> &str {
        match self {
            PastelError::ColorParseError => "could not parse color",
            PastelError::CouldNotReadFromStdin => "could not read color from standard input",
            PastelError::ColorArgRequired => {
                "A color argument needs to be provided on the command line or via a pipe"
            }
            PastelError::CouldNotParseNumber => "Could not parse number",
        }
    }
}

pub type Result<T> = std::result::Result<T, PastelError>;
