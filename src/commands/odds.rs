use super::Command;
use crate::helpers::has_at_least_n_args;
use crate::register_command;

use async_trait::async_trait;
use statrs::distribution::{DiscreteCDF, Hypergeometric};

#[derive(Default)]
pub struct Odds;

#[async_trait]
impl Command for Odds {
    fn name(&self) -> &'static str {
        "odds"
    }

    async fn execute(&self, args: &str, _sender: &str) -> String {
        match parse_odds_args(args) {
            Ok(params) => calculate_odds(params, args),
            Err(msg) => msg,
        }
    }
}

struct OddsParams {
    deck_size: u64,
    num_successes: u64,
    num_draws: u64,
    min_successes: u64,
}

fn parse_odds_args(args: &str) -> Result<OddsParams, String> {
    let usage = String::from(
        "USAGE: odds <Deck Size> <Number of Successes> <Number of Draws> <Successes Needed>",
    );

    if !has_at_least_n_args(args, 4) {
        return Err(usage);
    }

    let parts: Vec<&str> = args.split_whitespace().collect();
    let deck_size = parts[0].parse::<u64>();
    let num_successes = parts[1].parse::<u64>();
    let num_draws = parts[2].parse::<u64>();
    let min_successes = parts[3].parse::<u64>();

    match (deck_size, num_successes, num_draws, min_successes) {
        (Ok(deck_size), Ok(num_successes), Ok(num_draws), Ok(min_successes)) => {
            if num_successes > deck_size || num_draws > deck_size {
                return Err(String::from(
                    "Error: More successes or draws than cards in deck.",
                ));
            }

            Ok(OddsParams {
                deck_size,
                num_successes,
                num_draws,
                min_successes,
            })
        }
        _ => Err(usage),
    }
}

fn calculate_odds(params: OddsParams, args: &str) -> String {
    let hyper = Hypergeometric::new(
        params.num_successes,
        params.deck_size - params.num_successes,
        params.num_draws,
    );

    match hyper {
        Ok(h) => {
            let prob = 1.0 - h.cdf(params.min_successes - 1);
            let percentage = prob * 100.0;
            format!(
                "Odds of drawing {} or more of {} cards from {} draws in a {} card deck: {:.5}%",
                params.min_successes,
                params.num_successes,
                params.num_draws,
                params.deck_size,
                percentage
            )
        }
        Err(_) => {
            log::error!("Error creating hypergeometric distribution: {args}");
            String::from("Error creating hypergeometric distribution.")
        }
    }
}

register_command!(Odds);
