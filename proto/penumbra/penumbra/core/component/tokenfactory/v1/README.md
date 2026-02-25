## Token Factory Implementation Idea

### User story

The token factory will works as following:

#### Action TokenFactoryCreateWithBondingCurve

##### Input to TokenFactoryCreateWithBondingCurve

1. The user will submit the token metadata for the token they want to create (token name, token decimals, token max supply, image, whatever other metadata)
(Optional) 2. The user may specify the base token for the bonding curve. This may also just be hardcoded as UM (this has some benefits for liquidity composition later down the line - and is simpler to understand)
(Optional) 3. The user may specify the bonding curve max size. What this means is the total base token in 2 that is necessary to be deposited for the curve to be "fully bonded".

##### Output of TokenFactoryCreateWithBondingCurve

This action will output the following:

1. A BondingCurveNFT
(Optional) 2. A MintTokenNFT (this can be ingrained into 1 potentially)

The BondingCurveNFT will specify the price to mint a token. As more tokens get minted, the BondingCurveNFT is eventually consumed and a new one is created (with a higher price). This can be done once per token mint, or it can be done for set limits of the token. When a token is minted, the supply of that token goes up, and to mint a token the user must call the action `TokenFactoryMint`.

#### Action TokenFactoryMint

##### Input to TokenFactoryMint

1. The user specifies the asset of the token from the tokenFactory to mint, and the amount to mint, as well as the corresponding correct amount of base token that corresponds to this mint.

##### Output of TokenFactoryMint

1. The base_supply in the BondingCurveNFT is increased corresponding to how much of the base token was submitted. The nft is burnt depending on if the supply has surpassed a certain threshold depending on the curve.
2. The minted tokens are given to the user.

#### Action TokenFactoryBurn

Directly opposite to `Action TokenFactoryMint`. This should be able to consume nfts and make their sequence number go backwards and price decrease.

### Objects

#### BondingCurveNFT

```rust
pub struct BondingCurveNFT {
    pub sequence: u8,
    pub base_supplied: u64,
    pub base_supply_cap: u64
    pub current_price: f64 // Not a float in practice, only for conceptualisation purposes
    pub exponential_rate: f64
}

impl BondingCurveNFT {
    // Not actually a float, just for conceptualisation purposes
    /// Figure out how many tokens that are minted from a certain collateral deposit
    pub fn get_mint_amount(&self, input_base: u64) -> u64 {
        // Geometric sum formula = a(1-r^n)/(1-r)
        // where:
        // a = first term
        // r = common ratio 
        // n = number of terms
        (self.exponential_rate).log2() * (self.sequence as i32) = (input_base * (self.exponential_rate - 1) + 1).log2()
    }

    pub fn try_mint(&self) -> Option<Self> {
        let new_base_supplied = self.base_supplied + self.current_price
        if new_base_supplied >= 
        Self {
            sequence = self.sequence + 1,
            base_supplied = self.base_supplied + self.current_price,
            base_supply_cap = self.base_supply_cap,
            current_price = self.get_next_price(),
            exponential_rate = self.exponential_rate
        }
    }

    /// Assuming the exponential factor is 2
    pub fn get_next_price(&self) -> u64 {
        self.exponential_rate * current_price // Or exponential_rate ** sequence
    }

    /// Find sequence for the mint
    pub fn get_sequence_for_mint(&self) -> u8 {
        // If log is cheaply implemented:
        // Use log2 to determine sequence number based on supply
        ((self.base_supply as f64).log2()  as u8)
    }
}
```

### BondingCurve Completion

The bonding curve is completed when the max supply is reached. The max base supply must be set to a number such that the geometric sum is solvable for an integer n. I.e

$$ (r^n - 1)/(r-1) = \text{base_supply_cap} $$

has a solution for $n \in \mathbb{Z}^+$

## Privacy implications

Privacy implication of this bonding curve.

The total supply should always be observable, in the same way that an auction is observable to a user on Penumbra. When the user purchases or mints a new token, there should be no need to reveal any data, but if there ends up being a need for this, they can always shield the tokens afterward.

## Fairness implications

By keeping the same structure as the autction nft, it should be possible to guarantee the same fairness implications as for dutch auctions on Penumbra, and because it is an automated market maker in this case, there is no need for doing any matchmaking at all. Instead, price is predetermined by supply. Before the bonding period is complete, there may be users wanting to sell or buy tokens outside of the bonding curve pricing. This would have to be done on the auction interface rather than the minting/burning interface.

### Frontrunning protection

Because I don't fully understand frontrunning protection on penumbra, I'm not entirely sure what gurantees are carried over to this model. This will become more clear with time.

## Considerations

It needs to be thought about as to what flexibility we want to have for this token factory on penumbra.

The more flexibility will mean more complications and development time. We also need to consider simplicity and its effect on the longer term improvements, such as liquidity fungibility across other factories.