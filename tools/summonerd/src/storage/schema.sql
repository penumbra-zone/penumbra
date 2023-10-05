-- used for storing phase 2 contribution information
CREATE TABLE phase2_contributions (
    slot    INTEGER PRIMARY KEY,
    address BLOB NOT NULL
);
