# provision data/spots.json
provision-spots-json:
  mkdir -p ./data
  cargo run --bin cli -- -u -p ./data/spots.json
