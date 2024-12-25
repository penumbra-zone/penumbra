use clap::ArgEnum;
use penumbra_sdk_asset::Value;
use penumbra_sdk_auction::auction::dutch::DutchAuctionDescription;
use rand::Rng;
use rand::RngCore;
use rand_core::OsRng;
use serde::Serialize;

#[derive(ArgEnum, Clone, Debug, Serialize)]
pub enum GdaRecipe {
    #[clap(name = "10m")]
    TenMinutes,
    #[clap(name = "30m")]
    ThirtyMinutes,
    #[clap(name = "1h")]
    OneHour,
    #[clap(name = "2h")]
    TwoHours,
    #[clap(name = "6h")]
    SixHours,
    #[clap(name = "12h")]
    TwelveHours,
    #[clap(name = "1d")]
    OneDay,
    #[clap(name = "2d")]
    TwoDays,
}

impl GdaRecipe {
    pub fn as_blocks(&self) -> u64 {
        match self {
            GdaRecipe::TenMinutes => 10 * 12,
            GdaRecipe::ThirtyMinutes => 30 * 12,
            GdaRecipe::OneHour => 60 * 12,
            GdaRecipe::TwoHours => 2 * 60 * 12,
            GdaRecipe::SixHours => 6 * 60 * 12,
            GdaRecipe::TwelveHours => 12 * 60 * 12,
            GdaRecipe::OneDay => 24 * 60 * 12,
            GdaRecipe::TwoDays => 2 * 24 * 60 * 12,
        }
    }

    pub fn poisson_intensity(&self) -> f64 {
        match &self {
            GdaRecipe::TenMinutes => 0.0645833333333,
            GdaRecipe::ThirtyMinutes => 0.05058333333,
            GdaRecipe::OneHour => 0.02525,
            GdaRecipe::TwoHours => 0.02266666666,
            GdaRecipe::SixHours => 0.01075,
            GdaRecipe::TwelveHours => 0.0053333333,
            GdaRecipe::OneDay => 0.035,
            GdaRecipe::TwoDays => 0.00175,
        }
    }

    pub fn num_auctions(&self) -> u64 {
        match self {
            GdaRecipe::TenMinutes => 4,
            GdaRecipe::ThirtyMinutes => 12,
            GdaRecipe::OneHour => 12,
            GdaRecipe::TwoHours => 24,
            GdaRecipe::SixHours => 36,
            GdaRecipe::TwelveHours => 36,
            GdaRecipe::OneDay => 48,
            GdaRecipe::TwoDays => 48,
        }
    }

    pub fn sub_auction_length(&self) -> u64 {
        match &self {
            GdaRecipe::TenMinutes => 60,
            GdaRecipe::ThirtyMinutes => 60,
            GdaRecipe::OneHour => 120,
            GdaRecipe::TwoHours => 120,
            GdaRecipe::SixHours => 240,
            GdaRecipe::TwelveHours => 480,
            GdaRecipe::OneDay => 720,
            GdaRecipe::TwoDays => 1440,
        }
    }

    pub fn step_count(&self) -> u64 {
        60
    }
}

#[derive(Debug, Serialize)]
pub struct GradualAuction {
    pub input: Value,
    pub max_output: Value,
    pub min_output: Value,
    pub recipe: GdaRecipe,
    pub start_height: u64,
}

impl GradualAuction {
    pub fn new(
        input: Value,
        max_output: Value,
        min_output: Value,
        recipe: GdaRecipe,
        start_height: u64,
    ) -> Self {
        GradualAuction {
            input,
            max_output,
            min_output,
            recipe,
            start_height,
        }
    }

    pub fn generate_start_heights(&self) -> Vec<u64> {
        let lambda = self.recipe.poisson_intensity();
        let num_auctions = self.recipe.num_auctions();
        let start_height = self.start_height;
        let sub_auction_length = self.recipe.sub_auction_length();
        tracing::debug!(
            lambda,
            num_auctions,
            start_height,
            sub_auction_length,
            num_blocks = self.recipe.as_blocks(),
            "generating auction starts"
        );

        let mut rng = rand::thread_rng();
        let mut current_height = start_height as f64;

        let mut auction_starts = Vec::with_capacity(num_auctions as usize);
        for _ in 0..num_auctions {
            // See https://en.wikipedia.org/wiki/Inverse_transform_sampling
            // aka. a smirnov transform that is a big workshopped with the abs, but it generates a
            // nice calendar of auctions.
            let ff_clock = (rng.gen::<f64>() / lambda).ln().abs();
            current_height += ff_clock;
            let height = current_height.ceil() as u64;
            tracing::debug!(height, arrival_time = ff_clock, "selected auction start");
            auction_starts.push(height)
        }

        auction_starts
    }

    pub fn generate_auctions(&self) -> Vec<DutchAuctionDescription> {
        let start_heights = self.generate_start_heights();
        let sub_auction_length = self.recipe.sub_auction_length();
        let step_count = self.recipe.step_count();
        let num_auctions = self.recipe.num_auctions();
        let mut auctions = Vec::with_capacity(num_auctions as usize);
        for start_height in start_heights {
            let amount_chunk = self.input.amount.value() / num_auctions as u128;
            let input_chunk = Value {
                asset_id: self.input.asset_id,
                amount: amount_chunk.into(),
            };

            let scaled_min_output = self.min_output.amount.value() / num_auctions as u128;
            let scaled_max_output = self.max_output.amount.value() / num_auctions as u128;

            let mut nonce = [0u8; 32];
            OsRng.fill_bytes(&mut nonce);

            let end_height = start_height + sub_auction_length;
            tracing::debug!(
                start_height,
                end_height,
                sub_auction_length,
                step_count,
                ?input_chunk,
                "generating auction"
            );

            let auction = DutchAuctionDescription {
                input: input_chunk,
                output_id: self.max_output.asset_id,
                max_output: scaled_max_output.into(),
                min_output: scaled_min_output.into(),
                start_height,
                end_height,
                step_count,
                nonce,
            };
            auctions.push(auction);
        }
        auctions
    }
}
