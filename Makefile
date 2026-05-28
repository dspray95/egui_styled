# Run all pre-publish checks (fmt, clippy, tests, docs)
pre_publish:
	cargo fmt --all -- --check
	cargo clippy --all-targets -- -D warnings
	cargo test --all-targets
	cargo doc --no-deps

# Publish a new release to crates.io and tag the commit.
# Before running: bump the version in Cargo.toml and update CHANGELOG.md.
publish: pre_publish
	cargo publish
	git tag v$(shell cargo metadata --no-deps --format-version 1 | python3 -c "import sys,json; print(json.load(sys.stdin)['packages'][0]['version'])")
	git push origin --tags
