# Run all pre-publish checks (fmt, clippy, tests, docs)
pre_publish:
	cargo fmt --all -- --check
	cargo clippy --all-targets -- -D warnings
	cargo clippy --all-targets --all-features -- -D warnings
	cargo clippy --lib --target wasm32-unknown-unknown --all-features -- -D warnings
	cargo test --all-targets
	cargo test --lib --all-features
	cargo doc --no-deps

VERSION := $(shell cargo metadata --no-deps --format-version 1 | python3 -c "import sys,json; print(json.load(sys.stdin)['packages'][0]['version'])")
NOTES := target/release-notes.md

# Write this version's CHANGELOG section to $(NOTES); fails if it's missing or empty.
release_notes:
	@mkdir -p target
	@awk '/^## \[$(VERSION)\]/ {found=1; next} /^## \[/ {found=0} found' CHANGELOG.md > $(NOTES)
	@grep -q '[^[:space:]]' $(NOTES) || (echo "No CHANGELOG section for $(VERSION)" && exit 1)
	@echo "Release notes for v$(VERSION) written to $(NOTES)"

# Fails unless HEAD is pushed and its CI run passed, so releases only come from green commits.
ci_green:
	@git fetch -q origin
	@git merge-base --is-ancestor HEAD @{u} || (echo "HEAD isn't pushed yet" && exit 1)
	@result=$$(gh run list --commit $$(git rev-parse HEAD) --workflow ci.yml --json status,conclusion -q '.[0] | "\(.status) \(.conclusion)"'); \
	if [ "$$result" != "completed success" ]; then echo "CI for HEAD is not green (got: '$$result')"; exit 1; fi
	@echo "CI is green for $$(git rev-parse --short HEAD)"

# Publish to crates.io, tag the commit and create the GitHub release.
# Before running: bump the version in Cargo.toml, rename [Unreleased] in CHANGELOG.md,
# then push and wait for CI to pass.
publish: ci_green pre_publish release_notes
	cargo publish
	git tag v$(VERSION)
	git push origin v$(VERSION)
	gh release create v$(VERSION) --title v$(VERSION) --notes-file $(NOTES) --verify-tag

# Regenerate the README screenshots. Needs a display and Pillow; keep the window visible.
screenshots:
	cargo run --release --example showcase --features backdrop-blur -- --screenshot target/screenshots
	python3 docs/screenshots/encode.py target/screenshots docs/screenshots
