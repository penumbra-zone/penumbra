pub mod parameters {
    pub fn key() -> &'static str {
        "auction/parameters"
    }

    pub fn updated_flag() -> &'static str {
        "auction/parameters/updated"
    }
}

pub mod store {
    use crate::auction::id::AuctionId;

    pub fn prefix() -> &'static str {
        "auction/store/"
    }

    pub fn by_id(auction_id: AuctionId) -> String {
        format!("{}{auction_id}", prefix())
    }
}

pub mod dutch {
    pub mod trigger {
        use crate::auction::id::AuctionId;

        pub fn prefix() -> &'static str {
            "auction/dutch/trigger/"
        }

        pub fn by_height(trigger_height: u64) -> String {
            format!("{}{trigger_height:020}/", prefix())
        }

        pub fn auction_at_height(auction_id: AuctionId, trigger_height: u64) -> String {
            format!("{}{auction_id}", by_height(trigger_height))
        }
    }
}

#[cfg(test)]
mod tests {}
