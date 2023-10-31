-- used for storing phase 1 contribution information
CREATE TABLE phase1_contributions (
    slot    INTEGER PRIMARY KEY,
    -- 1 if this is a root
    is_root INTEGER NOT NULL,
    -- If this is the root, will be just the elements, and not a full contribution
    contribution_or_crs BLOB NOT NULL,
    -- NULL in the specific case that this is the root
    hash BLOB,
    -- NULL in the specific case that this is the root 
    address BLOB,
    -- Unix secs timestamp for when this entry was inserted
    time INTEGER NOT NULL
);

-- used for storing phase 2 contribution information
CREATE TABLE phase2_contributions (
    slot    INTEGER PRIMARY KEY,
    -- 1 if this is a root
    is_root INTEGER NOT NULL,
    -- If this is the root, will be just the elements, and not a full contribution
    contribution_or_crs BLOB NOT NULL,
    -- NULL in the specific case that this is the root
    hash BLOB,
    -- NULL in the specific case that this is the root 
    address BLOB,
    -- Unix secs timestamp for when this entry was inserted
    time INTEGER NOT NULL
);

-- Used to store metadata about specific addresses that participated.
CREATE TABLE participant_metadata (
  address BLOB PRIMARY KEY NOT NULL,
  -- The number of strikes that have been received.
  strikes INTEGER NOT NULL
);

CREATE TABLE transition_aux (
  id INTEGER PRIMARY KEY,
  data BLOB NOT NULL
);

