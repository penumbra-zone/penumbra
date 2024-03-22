-- 1. Create new tables for storing large BLOB data
CREATE TABLE phase1_contribution_data (
    slot INTEGER PRIMARY KEY,
    contribution_or_crs BLOB NOT NULL
);

CREATE TABLE phase2_contribution_data (
    slot INTEGER PRIMARY KEY,
    contribution_or_crs BLOB NOT NULL
);

-- 2. Migrate BLOB data to the new tables
INSERT INTO phase1_contribution_data (slot, contribution_or_crs)
SELECT slot, contribution_or_crs FROM phase1_contributions;

INSERT INTO phase2_contribution_data (slot, contribution_or_crs)
SELECT slot, contribution_or_crs FROM phase2_contributions;

-- 3. Drop the BLOB column from the original contributions tables
-- We need to create a temporary table because SQLite does not support dropping a column directly
BEGIN TRANSACTION;

ALTER TABLE phase1_contributions RENAME TO phase1_contributions_old;
CREATE TABLE phase1_contributions (
    slot INTEGER PRIMARY KEY,
    is_root INTEGER NOT NULL,
    hash BLOB,
    address BLOB,
    time INTEGER NOT NULL,
    FOREIGN KEY (slot) REFERENCES phase1_contribution_data(slot)
);
INSERT INTO phase1_contributions (slot, is_root, hash, address, time)
SELECT slot, is_root, hash, address, time FROM phase1_contributions_old;

ALTER TABLE phase2_contributions RENAME TO phase2_contributions_old;
CREATE TABLE phase2_contributions (
    slot INTEGER PRIMARY KEY,
    is_root INTEGER NOT NULL,
    hash BLOB,
    address BLOB,
    time INTEGER NOT NULL,
    FOREIGN KEY (slot) REFERENCES phase2_contribution_data(slot)
);
INSERT INTO phase2_contributions (slot, is_root, hash, address, time)
SELECT slot, is_root, hash, address, time FROM phase2_contributions_old;

COMMIT;

-- 4. Drop the old tables
DROP TABLE phase1_contributions_old;
DROP TABLE phase2_contributions_old;

-- 5. Vacuum database
-- After all migration steps and dropping the old tables
VACUUM;