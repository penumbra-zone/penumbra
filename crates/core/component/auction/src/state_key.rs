pub mod parameters {
    pub fn key() -> &'static str {
        "auction/parameters"
    }

    pub fn updated_flag() -> &'static str {
        "auction/parameters/updated"
    }
}

pub mod store {
    pub fn prefix() -> &'static str {
        "auction/store/"
    }

    // TODO: change this to `AuctionId`
    pub fn by_id(auction_id: u64) -> String {
        format!("{}{auction_id}", prefix())
    }
}

pub mod dutch {
    pub mod trigger {
        pub fn prefix() -> &'static str {
            "auction/dutch/trigger/"
        }

        pub fn by_height(trigger_height: u64) -> String {
            format!("{}{trigger_height:020}/", prefix())
        }

        // TODO: change to `auction_id: AuctionId` when we define it
        pub fn auction_at_height(auction_id: u64, trigger_height: u64) -> String {
            format!("{}{}", by_height(trigger_height), auction_id)
        }
    }
}

#[cfg(test)]
mod tests {}
