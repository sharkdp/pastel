use crate::commands::prelude::*;

use pastel::random::strategies;
use pastel::random::RandomizationStrategy;

pub struct RandomCommand;

impl GenericCommand for RandomCommand {
    fn run(&self, out: &mut Output, matches: &ArgMatches, config: &Config) -> Result<()> {
        let strategy_arg = matches.value_of("strategy").expect("required argument");

        let count = matches.value_of("number").expect("required argument");
        let count = count
            .parse::<usize>()
            .map_err(|_| PastelError::CouldNotParseNumber(count.into()))?;

        let mut strategy: Box<dyn RandomizationStrategy> = match strategy_arg {
            "vivid" => Box::new(strategies::Vivid),
            "rgb" => Box::new(strategies::UniformRGB),
            "gray" => Box::new(strategies::UniformGray),
            "lch_hue" => Box::new(strategies::UniformHueLCh),
            _ => unreachable!("Unknown randomization strategy"),
        };

        for _ in 0..count {
            out.show_color(config, &strategy.generate())?;
        }

        Ok(())
    }
}
