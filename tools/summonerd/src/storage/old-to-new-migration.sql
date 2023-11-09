-- scratch migration from chatgpt, needs edits

-- 1. Create new tables for storing large BLOB data
CREATE TABLE phase1_contribution_data (
    slot INTEGER PRIMARY KEY,
    contribution_or_crs BLOB NOT NULL
);

CREATE TABLE phase2_contribution_data (
    slot INTEGER PRIMARY KEY,
    contribution_or_crs BLOB NOT NULL
);

-- 2. Create the updated contributions tables without the BLOB column
-- Note: It is assumed that the old tables are renamed to phase1_contributions_old and phase2_contributions_old for the purpose of migration

-- 3. Migrate the data
-- Insert BLOB data into the new tables
INSERT INTO phase1_contribution_data (slot, contribution_or_crs)
SELECT slot, contribution_or_crs FROM phase1_contributions_old;

INSERT INTO phase2_contribution_data (slot, contribution_or_crs)
SELECT slot, contribution_or_crs FROM phase2_contributions_old;

-- Insert the rest of the data into the updated contributions tables
INSERT INTO phase1_contributions (slot, is_root, hash, address, time)
SELECT slot, is_root, hash, address, time FROM phase1_contributions_old;

INSERT INTO phase2_contributions (slot, is_root, hash, address, time)
SELECT slot, is_root, hash, address, time FROM phase2_contributions_old;

-- 4. Confirm the migration, ensure application logic is updated to join with the new tables

-- 5. Drop the old tables if everything is confirmed to be working fine
DROP TABLE phase1_contributions_old;
DROP TABLE phase2_contributions_old;
