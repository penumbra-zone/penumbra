#!/usr/bin/env bash
cargo run --release --bin pcli -- --home localnet_config tx lp order sell 183.070325gn@1.000501gm
cargo run --release --bin pcli -- --home localnet_config tx lp order sell 200gn@1.000501gm
cargo run --release --bin pcli -- --home localnet_config tx lp order sell 2.013014gn@1.00502gm
cargo run --release --bin pcli -- --home localnet_config tx lp order sell 2.013014gn@3.19999penumbra
cargo run --release --bin pcli -- --home localnet_config tx lp order buy 2.013014gn@3.19999penumbra
