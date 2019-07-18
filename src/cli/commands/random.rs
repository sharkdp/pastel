use crate::commands::prelude::*;
use crate::commands::show::show_color;

use pastel::random::strategies;
use pastel::random::RandomizationStrategy;

pub struct RandomCommand;

impl GenericCommand for RandomCommand {
    fn run(&self, out: &mut dyn Write, matches: &ArgMatches, config: &Config) -> Result<()> {
        let strategy_arg = matches.value_of("strategy").expect("required argument");

        let count = matches.value_of("number").expect("required argument");
        let count = count
            .parse::<usize>()
            .map_err(|_| PastelError::CouldNotParseNumber(count.into()))?;

        let mut strategy: Box<dyn RandomizationStrategy> = match strategy_arg {
            "vivid" => Box::new(strategies::Vivid),
            "uniform_rgb" => Box::new(strategies::UniformRGB),
            "uniform_gray" => Box::new(strategies::UniformGray),
            _ => unreachable!("Unknown randomization strategy"),
        };

        for _ in 0..count {
            show_color(out, &config, &strategy.generate())?;
        }

        Ok(())
    }
}
