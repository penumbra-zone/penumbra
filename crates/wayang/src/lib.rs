mod environment;
pub mod options;

use environment::Environment;
use options::SymbolPair;
use std::time::Duration;
use tokio::sync::watch;

#[derive(Debug, Clone)]
pub struct Status {
    pub height: u64,
    pub price: f64,
    pub pair: SymbolPair,
}

#[derive(Clone)]
pub struct Move {
    pub price: f64,
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
                price: moove.price,
                pair: self.environment.pair().clone(),
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

pub fn rhythm_and_feeler(options: options::Options) -> (Rhythm, Feeler) {
    let (moves_in, moves_out) = watch::channel(None);
    let (status_in, mut status_out) = watch::channel(None);
    // So that we can immediately get a status.
    status_out.mark_changed();
    (
        Rhythm {
            moves: moves_in,
            status: status_out,
        },
        Feeler {
            moves: moves_out,
            status: status_in,
            environment: Environment::new(options),
        },
    )
}
