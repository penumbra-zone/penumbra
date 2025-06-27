mod client;
pub mod config;
mod environment;
mod registry;

use client::Client;
use config::{Symbol, SymbolPair};
use regex::Regex;
use std::{fmt::Display, io::IsTerminal, str::FromStr, time::Duration};
use tokio::sync::watch;
use tracing_subscriber::{prelude::*, EnvFilter};

#[derive(Debug, Clone)]
pub struct PositionShape {
    pub upper_price: f64,
    pub lower_price: f64,
    pub base_liquidity: f64,
    pub quote_liquidity: f64,
}

impl FromStr for PositionShape {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"(\d+\.?\d*)/(\d+\.?\d*)\s+\[(\d+\.?\d*)\s*,\s*(\d+\.?\d*)\]")?;

        let captures = re.captures(s.trim()).ok_or_else(|| {
            anyhow::anyhow!(
                "expected format 'base_liquidity/quote_liquidity [lower_price, upper_price]'"
            )
        })?;

        let base_liquidity = captures[1]
            .parse::<f64>()
            .map_err(|_| anyhow::anyhow!("invalid base liquidity: {}", &captures[1]))?;
        let quote_liquidity = captures[2]
            .parse::<f64>()
            .map_err(|_| anyhow::anyhow!("invalid quote liquidity: {}", &captures[2]))?;
        let lower_price = captures[3]
            .parse::<f64>()
            .map_err(|_| anyhow::anyhow!("invalid lower price: {}", &captures[3]))?;
        let upper_price = captures[4]
            .parse::<f64>()
            .map_err(|_| anyhow::anyhow!("invalid upper price: {}", &captures[4]))?;

        Ok(PositionShape {
            upper_price,
            lower_price,
            base_liquidity,
            quote_liquidity,
        })
    }
}

impl Display for PositionShape {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}/{} [{}, {}]",
            self.base_liquidity, self.quote_liquidity, self.lower_price, self.upper_price
        )
    }
}

#[derive(Debug, Clone)]
pub struct Position {
    pub pair: SymbolPair,
    pub shape: PositionShape,
}

impl FromStr for Position {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"([a-zA-Z]+/[a-zA-Z]+)\s+(.+)")?;

        let captures = re.captures(s.trim())
            .ok_or_else(|| anyhow::anyhow!("expected format 'SYMBOL1/SYMBOL2 base_liquidity/quote_liquidity [lower_price, upper_price]'"))?;

        let shape_str = &captures[2];

        let pair = SymbolPair::from_str(&captures[1])?;
        let shape = PositionShape::from_str(shape_str)?;

        Ok(Position { pair, shape })
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.pair, self.shape)
    }
}

#[derive(Debug, Clone)]
pub struct Status {
    pub height: u64,
    pub positions: Vec<Position>,
}

#[derive(Clone)]
pub struct Move {
    pub position: Position,
}

pub struct Feeler {
    moves: watch::Receiver<Option<Move>>,
    status: watch::Sender<Option<Status>>,
    client: Client,
}

impl Feeler {
    async fn next_move(&mut self) -> anyhow::Result<Move> {
        let res = self.moves.wait_for(Option::is_some).await?;
        Ok(res.as_ref().cloned().expect("Impossible 000-000"))
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        let registry = self.client.registry().await?;
        dbg!(&registry.lookup(&Symbol::from_str("UM").unwrap()));
        dbg!(&registry.lookup(&Symbol::from_str("USDC").unwrap()));
        let mut height = 0u64;
        loop {
            let moove = self.next_move().await?;
            height += 1;
            let status = Status {
                height,
                positions: vec![moove.position],
            };
            self.status.send(Some(status))?;
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}

pub struct Rhythm {
    moves: watch::Sender<Option<Move>>,
    status: watch::Receiver<Option<Status>>,
}

impl Rhythm {
    pub async fn sense(&mut self) -> anyhow::Result<Option<Status>> {
        self.status.changed().await?;
        let res = self.status.borrow_and_update();
        Ok(res.as_ref().cloned())
    }

    pub async fn do_move(&self, moove: Move) -> anyhow::Result<()> {
        self.moves.send(Some(moove))?;
        Ok(())
    }
}

pub async fn rhythm_and_feeler(config: &config::Config) -> anyhow::Result<(Rhythm, Feeler)> {
    let client = Client::init(config.view_service.as_str()).await?;
    let (moves_in, moves_out) = watch::channel(None);
    let (status_in, mut status_out) = watch::channel(None);
    // So that we can immediately get a status.
    status_out.mark_changed();
    Ok((
        Rhythm {
            moves: moves_in,
            status: status_out,
        },
        Feeler {
            moves: moves_out,
            status: status_in,
            client,
        },
    ))
}

pub fn init_tracing() -> anyhow::Result<()> {
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(std::io::stdout().is_terminal())
        .with_writer(std::io::stderr)
        .with_target(true);
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;

    let registry = tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer);
    registry.init();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_from_str() {
        let position = Position::from_str("UM/USDC 100/200 [0.8, 0.9]").unwrap();
        assert_eq!(position.pair.to_string(), "UM/USDC");
        assert_eq!(position.shape.base_liquidity, 100.0);
        assert_eq!(position.shape.quote_liquidity, 200.0);
        assert_eq!(position.shape.lower_price, 0.8);
        assert_eq!(position.shape.upper_price, 0.9);
    }

    #[test]
    fn test_position_from_str_with_decimals() {
        let position = Position::from_str("UM/USDC 100.5/200.25 [0.8, 0.9]").unwrap();
        assert_eq!(position.pair.to_string(), "UM/USDC");
        assert_eq!(position.shape.base_liquidity, 100.5);
        assert_eq!(position.shape.quote_liquidity, 200.25);
        assert_eq!(position.shape.lower_price, 0.8);
        assert_eq!(position.shape.upper_price, 0.9);
    }

    #[test]
    fn test_position_from_str_invalid_format() {
        assert!(Position::from_str("invalid").is_err());
        assert!(Position::from_str("UM/USDC").is_err());
        assert!(Position::from_str("UM 100/200 [0.8, 0.9]").is_err());
    }
}
