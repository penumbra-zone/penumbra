-- used for storing phase 2 contribution information
CREATE TABLE phase2_contributions (
    slot    INTEGER PRIMARY KEY,
    -- 1 if this is a root
    is_root INTEGER NOT NULL,
    -- If this is the root, will be just the elements, and not a full contribution
    contribution_or_crs BLOB NOT NULL,
    -- NULL in the specific case that this is the root 
    address BLOB
);
