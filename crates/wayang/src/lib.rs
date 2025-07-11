mod client;
pub mod config;
pub mod dex;

use client::Client;
use dex::{effectively_the_same_position, PenumbraPosition, Position, Registry, Symbol};
use penumbra_sdk_dex::lp::position::State;
use penumbra_sdk_keys::keys::AddressIndex;
use rand_core::OsRng;
use std::{io::IsTerminal as _, str::FromStr as _};
use tokio::sync::watch;
use tracing_subscriber::{prelude::*, EnvFilter};

#[derive(Debug, Clone)]
pub struct Status {}

#[derive(Clone)]
pub struct Move {
    pub position: Position,
}

pub struct Feeler {
    account: u32,
    moves: watch::Receiver<Option<Move>>,
    status: watch::Sender<Option<Status>>,
    client: Client,
}

impl Feeler {
    async fn next_move(&mut self) -> anyhow::Result<Move> {
        let res = self.moves.wait_for(Option::is_some).await?;
        Ok(res.as_ref().cloned().expect("Impossible 000-004"))
    }

    async fn wait_for_change(
        &mut self,
        registry: &Registry,
        from: &PenumbraPosition,
    ) -> anyhow::Result<()> {
        self.moves
            .wait_for(|m| {
                let Some(m) = m else { return false };
                let Ok(p) = m.position.to_penumbra(&mut OsRng, registry) else {
                    return false;
                };
                !effectively_the_same_position(&p, from)
            })
            .await?;
        Ok(())
    }

    async fn tick(&mut self, registry: &Registry) -> anyhow::Result<()> {
        let moove = self.next_move().await?;
        let desired_position = moove.position.to_penumbra(&mut OsRng, registry)?;
        let positions = self.client.positions().await?;
        let res = self
            .client
            .build_and_submit(AddressIndex::new(self.account), |mut planner| {
                let mut matching_position = false;
                let mut did_something = false;
                for position in positions.iter() {
                    match position.state {
                        State::Withdrawn { sequence } if !position.reserves.empty() => {
                            did_something = true;
                            planner.position_withdraw(
                                position.id(),
                                position.reserves.clone(),
                                position.phi.pair,
                                sequence + 1,
                            );
                        }
                        State::Closed => {
                            did_something = true;
                            planner.position_withdraw(
                                position.id(),
                                position.reserves.clone(),
                                position.phi.pair,
                                0,
                            );
                        }
                        State::Opened
                            if !matching_position
                                && effectively_the_same_position(&position, &desired_position) =>
                        {
                            matching_position = true;
                        }
                        State::Opened => {
                            did_something = true;
                            planner.position_close(position.id());
                        }
                        _ => {}
                    }
                }
                if !matching_position {
                    did_something = true;
                    planner.position_open(desired_position.clone());
                }
                if did_something {
                    Ok(Some(planner))
                } else {
                    Ok(None)
                }
            })
            .await?;
        // If we didn't end up creating a transaction, we need to wait until the desired state
        // changes.
        match res {
            None => {
                self.wait_for_change(registry, &desired_position).await?;
            }
            Some(tx) => tracing::info!("detected transaction: {}", tx.id()),
        };
        let status = Status {};
        self.status.send(Some(status))?;
        Ok(())
    }

    pub async fn run(mut self) -> anyhow::Result<()> {
        let registry = self.client.registry().await?;
        dbg!(&registry.lookup(&Symbol::from_str("UM").unwrap()));
        dbg!(&registry.lookup(&Symbol::from_str("USDC").unwrap()));
        loop {
            if let Err(e) = self.tick(&registry).await {
                tracing::warn!(error = %e, "Failed to process move");
            }
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
    let client = Client::init(config.grpc_url.as_str(), config.view_service.as_str()).await?;
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
            account: config.account,
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
