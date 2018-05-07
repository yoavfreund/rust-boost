# RUST_BACKTRACE=1 RUST_LOG=DEBUG cargo run single 0 564 500
# RUST_BACKTRACE=1 RUST_LOG=DEBUG cargo run --release single 0 564 1500
RUST_BACKTRACE=1 RUST_LOG=DEBUG cargo run --release single 0 564 1500 2> error.log

