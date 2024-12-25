pub mod parameters {
    pub fn key() -> &'static str {
        "auction/parameters"
    }

    pub fn updated_flag() -> &'static str {
        "auction/parameters/updated"
    }
}

pub mod value_balance {
    use penumbra_sdk_asset::asset;

    #[allow(dead_code)] // For some reason, this shows up as unused
    pub fn prefix() -> &'static str {
        "auction/value_breaker/"
    }

    #[allow(dead_code)] // For some reason, this shows up as unused
    pub fn for_asset(asset_id: &asset::Id) -> String {
        format!("{}{asset_id}", prefix())
    }
}

pub mod auction_store {
    use crate::auction::id::AuctionId;

    pub fn prefix() -> &'static str {
        "auction/auction_store/"
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
