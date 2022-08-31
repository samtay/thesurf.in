set export

MSW_API_KEY := 'op://Private/MSW_API_KEY/credential'

# run tests
test:
  op run -- cargo test

# run server
run-server:
  op run -- cargo run --bin server

# provision data/spots.json
provision-spots-json:
  mkdir -p ./data
  cargo run --bin cli -- -u -p ./data/spots.json
