pub mod config;
mod environment;

use config::SymbolPair;
use environment::Environment;
use std::{io::IsTerminal, time::Duration};
use tokio::sync::watch;
use tracing_subscriber::{prelude::*, EnvFilter};

#[derive(Debug, Clone)]
pub struct PositionShape {
    pub upper_price: f64,
    pub lower_price: f64,
    pub base_liquidity: f64,
    pub quote_liquidity: f64,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub pair: SymbolPair,
    pub shape: PositionShape,
}

#[derive(Debug, Clone)]
pub struct Status {
    pub height: u64,
    pub positions: Vec<Position>,
}

#[derive(Clone)]
pub struct Move {
    pub shape: PositionShape,
}

pub struct Feeler {
    environment: Environment,
    moves: watch::Receiver<Option<Move>>,
    status: watch::Sender<Option<Status>>,
}

impl Feeler {
    async fn next_move(&mut self) -> anyhow::Result<Move> {
        let res = self.moves.wait_for(Option::is_some).await?;
        Ok(res.as_ref().cloned().expect("Impossible 000-000"))
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        let mut height = 0u64;
        loop {
            let moove = self.next_move().await?;
            height += 1;
            let status = Status {
                height,
                positions: vec![Position {
                    pair: self.environment.pair().clone(),
                    shape: moove.shape,
                }],
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

pub fn rhythm_and_feeler(config: &config::Config) -> anyhow::Result<(Rhythm, Feeler)> {
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
            environment: Environment::new(config)?,
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
