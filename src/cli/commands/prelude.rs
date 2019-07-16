pub use crate::config::Config;
pub use crate::error::{PastelError, Result};

pub use clap::ArgMatches;

pub use super::io::*;
pub use super::traits::*;

pub use pastel::Color;

pub use crate::termcolor::ToTermColor;
pub use ansi_term::Color as TermColor;
