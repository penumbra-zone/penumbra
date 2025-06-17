-- Version 0
DROP TABLE IF EXISTS dex_ex_price_charts CASCADE;
DROP TABLE IF EXISTS dex_ex_pairs_block_snapshot CASCADE;
DROP TABLE IF EXISTS dex_ex_pairs_summary CASCADE;
DROP TABLE IF EXISTS dex_ex_aggregate_summary CASCADE;
DROP TABLE IF EXISTS dex_ex_metadata CASCADE;
DROP TABLE IF EXISTS dex_ex_position_state CASCADE;
DROP TABLE IF EXISTS dex_ex_position_reserves CASCADE;
DROP TABLE IF EXISTS dex_ex_position_executions CASCADE;
DROP TABLE IF EXISTS dex_ex_position_withdrawals CASCADE;
DROP TABLE IF EXISTS dex_ex_batch_swap_traces CASCADE;
DROP TABLE IF EXISTS dex_ex_block_summary CASCADE;
DROP TABLE IF EXISTS dex_ex_transactions CASCADE;
-- Version 1
DROP SCHEMA IF EXISTS dex_ex CASCADE;
