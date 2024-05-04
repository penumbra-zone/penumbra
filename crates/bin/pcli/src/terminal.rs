use std::io::{IsTerminal, Read, Write};

use anyhow::Result;
use penumbra_custody::threshold::{SigningRequest, Terminal};
use termion::input::TermRead;
use tonic::async_trait;

/// For threshold custody, we need to implement this weird terminal abstraction.
///
/// This actually does stuff to stdin and stdout.
pub struct ActualTerminal;

#[async_trait]
impl Terminal for ActualTerminal {
    async fn confirm_request(&self, signing_request: &SigningRequest) -> Result<bool> {
        let (description, json) = match signing_request {
            SigningRequest::TransactionPlan(plan) => {
                ("transaction", serde_json::to_string_pretty(plan)?)
            }
            SigningRequest::ValidatorDefinition(def) => {
                ("validator definition", serde_json::to_string_pretty(def)?)
            }
            SigningRequest::ValidatorVote(vote) => {
                ("validator vote", serde_json::to_string_pretty(vote)?)
            }
        };
        println!("Do you approve this {description}?");
        println!("{json}");
        println!("Press enter to continue");
        self.next_response().await?;
        Ok(true)
    }

    async fn explain(&self, msg: &str) -> Result<()> {
        println!("{}", msg);
        Ok(())
    }

    async fn broadcast(&self, data: &str) -> Result<()> {
        println!("{}", data);
        Ok(())
    }

    async fn next_response(&self) -> Result<Option<String>> {
        // Use raw mode to allow reading more than 1KB/4KB of data at a time
        // See https://unix.stackexchange.com/questions/204815/terminal-does-not-accept-pasted-or-typed-lines-of-more-than-1024-characters
        use termion::raw::IntoRawMode;
        tracing::debug!("about to enter raw mode for long pasted input");

        // In raw mode, the input is not mirrored into the terminal, so we need
        // to read char-by-char and echo it back.
        let mut stdout = std::io::stdout().into_raw_mode()?;

        let mut bytes = Vec::with_capacity(8192);
        for b in std::io::stdin().bytes() {
            let b = b?;
            // In raw mode, the enter key might generate \r or \n, check either.
            if b == b'\n' || b == b'\r' {
                break;
            }
            bytes.push(b);
            stdout.write(&[b]).unwrap();
            // Flushing may not be the most efficient but performance isn't critical here.
            stdout.flush()?;
        }
        // Drop _stdout to restore the terminal to normal mode
        std::mem::drop(stdout);
        // We consumed a newline of some kind but didn't echo it, now print
        // one out so subsequent output is guaranteed to be on a new line.
        println!("");

        tracing::debug!("exited raw mode and returned to cooked mode");

        let line = String::from_utf8(bytes)?;
        tracing::debug!(?line, "read response line");

        if line.is_empty() {
            return Ok(None);
        }
        Ok(Some(line))
    }

    async fn get_password(&self) -> Result<String> {
        fn get_possibly_empty_string() -> Result<String> {
            // The `rpassword` crate doesn't support reading from stdin, so we check
            // for an interactive session. We must support non-interactive use cases,
            // for integration with other tooling.
            if std::io::stdin().is_terminal() {
                Ok(rpassword::prompt_password("Enter password:")?)
            } else {
                Ok(std::io::stdin().lock().read_line()?.unwrap_or_default())
            }
        }

        let mut string: String = Default::default();
        while string.is_empty() {
            string = get_possibly_empty_string()?;
        }
        Ok(string)
    }
}
