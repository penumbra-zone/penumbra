use regex::Regex;
use serenity::{
    async_trait,
    model::{channel::Message, id::GuildId, user::User},
    prelude::*,
};
use std::{collections::VecDeque, env};
use tokio::{sync::mpsc, time::Instant};

// Names of channels for which the bot is enabled
const ENABLED_CHANNELS: &[&str] = &["bot-stuff"];

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, message: Message) {
        // Don't trigger on messages we ourselves send, or when a message is in a channel we're not
        // supposed to interact within
        if message.author.id == ctx.cache.current_user().await.id
            || !ENABLED_CHANNELS.contains(
                &message
                    .channel_id
                    .name(&ctx)
                    .await
                    .unwrap_or_else(String::new)
                    .as_str(),
            )
        {
            return;
        }

        // Match penumbra testnet addresses (any version)
        let match_address =
            Regex::new(r"penumbrav\dt1[qpzry9x8gf2tvdw0s3jn54khce6mua7l]{126}").unwrap();

        // Collect all the matches into a struct, bundled with the original message
        let queue_message = AddressQueueMessage {
            addresses: match_address
                .find_iter(&message.content)
                .map(|m| m.as_str().to_string())
                .collect(),
            message,
        };

        // If no addresses were found, don't bother sending the message to the queue
        if queue_message.addresses.is_empty() {
            return;
        }

        // Send the message to the queue, to be processed asynchronously
        ctx.data
            .read()
            .await
            .get::<AddressQueue>()
            .expect("address queue exists")
            .send(queue_message)
            .await
            .expect("send to queue always succeeds");
    }

    async fn cache_ready(&self, ctx: Context, guilds: Vec<GuildId>) {
        println!("Connected to:");
        for guild_id in guilds {
            println!(
                "- {} (id: {})",
                guild_id
                    .name(&ctx.cache)
                    .await
                    .unwrap_or_else(|| "[unknown]".to_string()),
                guild_id
            );
        }
    }
}

/// `TypeMap` key for the address queue.
struct AddressQueue;

/// `TypeMap` value for the sender end of the address queue.
#[derive(Debug, Clone)]
struct AddressQueueMessage {
    /// The originating message that contained these addresses.
    message: Message,
    /// The addresses matched in the originating message.
    addresses: Vec<String>,
}

/// Associate the `AddressQueue` key with an `mpsc::Sender` for `AddressQueueMessage`s in the `TypeMap`.
impl TypeMapKey for AddressQueue {
    type Value = mpsc::Sender<AddressQueueMessage>;
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Make a new client using a token set by an environment variable, with our handlers
    let mut client = Client::builder(&env::var("DISCORD_TOKEN")?)
        .event_handler(Handler)
        .await?;

    // Get the cache and http part of the client, for use in dispatching replies
    let cache_and_http = client.cache_and_http.clone();

    // Put the sending end of the address queue into the global TypeMap
    let (tx, mut rx) = mpsc::channel(100);
    client.data.write().await.insert::<AddressQueue>(tx);

    // Spawn a task to handle the address queue
    tokio::spawn(async move {
        let mut send_history: VecDeque<(User, Instant)> = VecDeque::new();

        while let Some(AddressQueueMessage { message, addresses }) = rx.recv().await {
            for addr in addresses {
                // TODO: Keep track of addresses to which funds have been sent in a database, with a
                // last-sent timestamp. If the discord user or address has been sent to recently,
                // don't send coins to it again, instead replying to let the user know when more
                // will be available.

                // TODO: Invoke `pcli tx send` to dispense some random coins to the address, then
                // wait until `pcli balance` indicates that the transaction has been confirmed.
                // While this is happening, use the typing indicator API to show that something is
                // happening.

                // Reply to the originating message with the address
                message
                    .reply_ping(&cache_and_http, format!("Found address: {}", addr.as_str()))
                    .await
                    .unwrap();
            }
        }
    });

    // Start the client
    client.start().await?;

    Ok(())
}
