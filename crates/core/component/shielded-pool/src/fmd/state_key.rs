// use std::string::String;

pub mod parameters {
    pub fn current() -> &'static str {
        "shielded_pool/fmd_parameters/current"
    }

    pub fn previous() -> &'static str {
        "shielded_pool/fmd_parameters/previous"
    }
}

pub(super) mod clue_count {
    pub fn current() -> &'static str {
        "shielded_pool/fmd_clue_count/current"
    }

    pub fn previous() -> &'static str {
        "shielded_pool/fmd_clue_count/previous"
    }
}
