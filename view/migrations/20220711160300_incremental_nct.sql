-- don't use the old nct single-blob table
DROP TABLE IF EXISTS note_commitment_tree;

-- the shape information about the nct
CREATE TABLE nct_position ( position BIGINT );
INSERT INTO nct_position VALUES ( 0 ); -- starting position is 0

CREATE TABLE nct_forgotten ( forgotten BIGINT NOT NULL );
INSERT INTO nct_forgotten VALUES ( 0 ); -- starting forgotten version is 0

-- the hashes for nodes in the nct
CREATE TABLE nct_hashes (
    position BIGINT NOT NULL,
    height   TINYINT NOT NULL,
    hash     BLOB NOT NULL
);

-- these indices may help with 2-dimensional range deletion
CREATE INDEX hash_position_idx ON nct_hashes ( position );
--CREATE INDEX hash_height_idx ON nct_hashes ( height );

-- all the commitments stored in the nct
CREATE TABLE nct_commitments (
    position BIGINT NOT NULL,
    commitment BLOB NOT NULL
);