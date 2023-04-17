-- the shape information about the sct
CREATE TABLE sct_position ( position BIGINT );
INSERT INTO sct_position VALUES ( 0 ); -- starting position is 0

CREATE TABLE sct_forgotten ( forgotten BIGINT NOT NULL );
INSERT INTO sct_forgotten VALUES ( 0 ); -- starting forgotten version is 0

-- the hashes for nodes in the sct
CREATE TABLE sct_hashes (
    position BIGINT NOT NULL,
    height   TINYINT NOT NULL,
    hash     BLOB NOT NULL
);

-- these indices may help with 2-dimensional range deletion
CREATE INDEX hash_position_idx ON sct_hashes ( position );
--CREATE INDEX hash_height_idx ON sct_hashes ( height );

-- all the commitments stored in the sct
CREATE TABLE sct_commitments (
    position BIGINT NOT NULL,
    commitment BLOB NOT NULL
);