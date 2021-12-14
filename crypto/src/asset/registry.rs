use std::sync::Arc;

use once_cell::sync::Lazy;
use regex::{Regex, RegexSet};

use crate::asset::{denom, BaseDenom, DisplayDenom};

// penumbra
// upenumbra
// dpenumbra-penumbravaloper1tsehasteuheasouthea

pub struct Registry {
    // Set of regexes for base denominations
    base_set: RegexSet,
    // Individual regexes for base denominations
    base_regexes: Vec<Regex>,
    // List of constructors, indexed by base denom regex indices
    // Each constructor maps a base denom string to the denom metadata.
    constructors: Vec<fn(&str) -> denom::Inner>,
    // Set of regexes for display denominations
    display_set: RegexSet,
    // Mapping from display regex indices to base indices
    display_to_base: Vec<usize>,
    // Display regexes, grouped by base index
    display_regexes: Vec<Vec<Regex>>,
}

#[derive(Default)]
struct Builder {
    base_regexes: Vec<&'static str>,
    constructors: Vec<fn(&str) -> denom::Inner>,
    display_regexes: Vec<Vec<&'static str>>,
}

impl Builder {
    fn add_asset(
        mut self,
        base_regex: &'static str,
        display_regexes: &[&'static str],
        constructor: fn(&str) -> denom::Inner,
    ) -> Self {
        self.base_regexes.push(base_regex);
        self.constructors.push(constructor);
        self.display_regexes.push(display_regexes.to_vec());

        self
    }

    fn build(self) -> Registry {
        let mut display_to_base = Vec::new();
        let mut display_regexes = Vec::new();
        for (base_index, displays) in self.display_regexes.iter().enumerate() {
            for _d in displays.iter() {
                display_to_base.push(base_index);
            }
            display_regexes.push(displays.iter().map(|d| Regex::new(d).unwrap()).collect());
        }

        Registry {
            base_set: RegexSet::new(self.base_regexes.iter()).unwrap(),
            base_regexes: self
                .base_regexes
                .iter()
                .map(|r| Regex::new(r).unwrap())
                .collect(),
            constructors: self.constructors,
            display_set: RegexSet::new(
                self.display_regexes
                    .iter()
                    .flat_map(|displays| displays.iter()),
            )
            .unwrap(),
            display_to_base,
            display_regexes,
        }
    }
}

impl Registry {
    pub fn parse_base(&self, raw_denom: &str) -> Option<BaseDenom> {
        // We hope that our regexes are disjoint (TODO: add code to test this)
        // so that there will only ever be one match from the RegexSet.

        if let Some(base_index) = self.base_set.matches(raw_denom).iter().next() {
            // We've matched a base denomination.

            // Rematch with the specific pattern to obtain captured denomination data.
            let data = self.base_regexes[base_index]
                .captures(raw_denom)
                .expect("already checked this regex matches")
                .name("data")
                .map(|m| m.as_str())
                .unwrap_or("");

            Some(BaseDenom {
                inner: Arc::new(self.constructors[base_index](data)),
            })
        } else if let Some(_) = self.display_set.matches(raw_denom).iter().next() {
            // 2. This denom isn't a base denom, it's a display denom
            None
        } else {
            // 3. Fallthrough: create default base denom
            Some(BaseDenom {
                inner: Arc::new(denom::Inner::new(raw_denom.to_string(), Vec::new())),
            })
        }
    }

    pub fn parse_display(&self, raw_denom: &str) -> Option<DisplayDenom> {
        self.display_set
            .matches(raw_denom)
            .iter()
            .next()
            .map(|display_index| {
                let base_index = self.display_to_base[display_index];
                // We need to determine which unit we matched
                for (unit_index, regex) in self.display_regexes[base_index].iter().enumerate() {
                    if let Some(capture) = regex.captures(raw_denom) {
                        let data = capture.name("data").map(|m| m.as_str()).unwrap_or("");
                        return DisplayDenom {
                            inner: Arc::new(self.constructors[base_index](data)),
                            unit_index,
                        };
                    }
                }
                unreachable!("we matched one of the display regexes");
            })
    }
}

pub static REGISTRY: Lazy<Registry> = Lazy::new(|| {
    Builder::default()
        .add_asset(
            "upenumbra",
            &["penumbra", "mpenumbra"],
            (|data: &str| {
                assert!(data.is_empty());
                denom::Inner::new(
                    "upenumbra".to_string(),
                    vec![
                        denom::Unit {
                            exponent: 6,
                            denom: "penumbra".to_string(),
                        },
                        denom::Unit {
                            exponent: 3,
                            denom: "mpenumbra".to_string(),
                        },
                    ],
                )
            }) as for<'r> fn(&'r str) -> _,
        )
        .add_asset(
            // TODO: this doesn't restrict the length of the bech32 encoding
            "udpenumbra-(?P<data>penumbravaloper1[a-zA-HJ-NP-Z0-9]+)",
            &[
                "dpenumbra-(?P<data>penumbravaloper1[a-zA-HJ-NP-Z0-9]+)",
                "mdpenumbra-(?P<data>penumbravaloper1[a-zA-HJ-NP-Z0-9]+)",
            ],
            (|data: &str| {
                assert!(!data.is_empty());
                denom::Inner::new(
                    format!("udpenumbra-{}", data),
                    vec![
                        denom::Unit {
                            exponent: 6,
                            denom: format!("dpenumbra-{}", data),
                        },
                        denom::Unit {
                            exponent: 3,
                            denom: format!("mdpenumbra-{}", data),
                        },
                    ],
                )
            }) as for<'r> fn(&'r str) -> _,
        )
        .build()
});
