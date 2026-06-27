# AllInKit Plugin Catalog

The official index of third-party AllInKit plugins. The AllInKit desktop client
fetches `catalog.json` from this repo's `main` branch on every marketplace open
and renders the listed plugins for one-click install.

This repo holds **only the index** — the actual `.aikpkg` packages live in each
plugin author's own GitHub Releases; each catalog entry's `download_url` points
there. Signature verification (ed25519 + TOFU) happens on the client at install
time, not here.

## Add or update a plugin

See [CONTRIBUTING.md](CONTRIBUTING.md). In short: publish your signed `.aikpkg`
to a GitHub Release, then open a PR editing `catalog.json`. CI validates the
format and `download_url` reachability; a maintainer merges.

## Validate locally

```bash
cd validator
cargo run -p catalog-validator -- check ../catalog.json
```
Add `--no-reachability` to skip the `download_url` HEAD checks (offline).