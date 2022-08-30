-- Add migration script here

CREATE TABLE `batch_swap_output_data` (
  `height` BIGINT NOT NULL,
  `asset_1` BLOB NOT NULL,
  `asset_2` BLOB NOT NULL,
  `delta_1` BIGINT NOT NULL,
  `delta_2` BIGINT NOT NULL,
  `lambda_1` BIGINT NOT NULL,
  `lambda_2` BIGINT NOT NULL,
  `success` BOOLEAN NOT NULL,
  PRIMARY KEY (`height`, `asset_1`, `asset_2`)
);

CREATE INDEX `height_assets` ON `batch_swap_output_data` (`height`, `asset_1`, `asset_2`);