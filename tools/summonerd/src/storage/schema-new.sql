-- New table for storing contribution_or_crs data for phase 1
CREATE TABLE phase1_contribution_data (
    slot INTEGER PRIMARY KEY,
    contribution_or_crs BLOB NOT NULL
);

-- Updated phase1_contributions table without the large BLOB column
CREATE TABLE phase1_contributions (
    slot INTEGER PRIMARY KEY,
    is_root INTEGER NOT NULL,
    hash BLOB,
    address BLOB,
    time INTEGER NOT NULL,
    FOREIGN KEY (slot) REFERENCES phase1_contribution_data(slot)
);

-- New table for storing contribution_or_crs data for phase 2
CREATE TABLE phase2_contribution_data (
    slot INTEGER PRIMARY KEY,
    contribution_or_crs BLOB NOT NULL
);

-- Updated phase2_contributions table without the large BLOB column
CREATE TABLE phase2_contributions (
    slot INTEGER PRIMARY KEY,
    is_root INTEGER NOT NULL,
    hash BLOB,
    address BLOB,
    time INTEGER NOT NULL,
    FOREIGN KEY (slot) REFERENCES phase2_contribution_data(slot)
);

-- participant_metadata remains unchanged
CREATE TABLE participant_metadata (
  address BLOB PRIMARY KEY NOT NULL,
  strikes INTEGER NOT NULL
);

-- transition_aux remains unchanged
CREATE TABLE transition_aux (
  id INTEGER PRIMARY KEY,
  data BLOB NOT NULL
);
