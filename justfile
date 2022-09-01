set export

MSW_API_KEY := 'op://Private/MSW_API_KEY/credential'

# run tests
test:
  op run -- cargo test

# watch server
watch-server:
  op run -- cargo watch -x clippy -x 'run --bin server'

# watch test endpoint
watch-test:
  watchexec --exts rs -- curl 127.0.0.1:8080/test

# run server
run-server:
  op run -- cargo run --bin server

# provision data/spots.json
provision-spots-json:
  mkdir -p ./data
  cargo run --bin cli -- -u -p ./data/spots.json
