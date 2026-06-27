# Contributing a Plugin to the Catalog

## Before you PR

1. Build your `.aikpkg` (your plugin's cdylib + `plugin.toml` + `ui/` + `plugin.sig`).
2. Sign it with `aik-sign sign <plugin-dir> --key <your-ed25519-key>` — see the
   AllInKit repo's `docs/dev-keys/README.md`. The **public key** (hex) is the
   `signing_pubkey` you put in your catalog entry.
3. Publish a GitHub Release on your plugin's repo and upload the `.aikpkg` as a
   release asset. The asset's URL is your entry's `download_url`.

## catalog.json entry

```json
{
  "id": "json-tool",
  "name": "JSON Tool",
  "description": "Format and validate JSON",
  "author": "Your Name",
  "version": "1.0.0",
  "tags": ["encoding", "utility"],
  "capabilities": ["kv"],
  "download_url": "https://github.com/<you>/<plugin>/releases/download/v1.0.0/json-tool.aikpkg",
  "signing_pubkey": "<64-char-hex-ed25519-public-key>",
  "icon_url": "https://raw.githubusercontent.com/<you>/<plugin>/main/icon.png"
}
```

## Field rules (enforced by CI)

| Field | Required | Rule |
|-------|----------|------|
| `id` | yes | `^[A-Za-z0-9_-]+$`, unique across catalog, must match your `plugin.toml` id |
| `name` | yes | non-empty |
| `description`, `author` | yes | may be empty |
| `version` | yes | valid semver (e.g. `1.0.0`) |
| `download_url` | yes | `https://` URL to your `.aikpkg` release asset; must be reachable (HEAD 2xx) |
| `signing_pubkey` | yes | non-empty, valid hex (your ed25519 public key) |
| `tags`, `capabilities` | optional | string arrays; `capabilities` are display-only — real permissions come from your `plugin.toml` |
| `icon_url` | optional | `https://` URL if present |

## Signing pubkey policy

You **may** change `signing_pubkey` in a later PR (e.g. key rotation). Existing
users are protected by client-side TOFU: a changed pubkey makes their upgrade
fail with `PubkeyMismatch`, forcing an explicit uninstall + reinstall — the
client never silently re-trusts a new key. New users accept whatever pubkey is
in the catalog at install time.

**Residual risk (acknowledged):** a malicious maintainer of this catalog repo
could swap a `signing_pubkey` and publish a malicious `.aikpkg` signed by the
new key, attacking *new* users. This is mitigated by maintainer trust and a
future catalog-wide signature (not yet implemented), not by re-verifying the
`.aikpkg` here (CI re-verification cannot stop an attacker who controls both
the catalog entry and a freshly-signed package).

## PR flow

1. Edit `catalog.json` (add your entry, or bump `version` + `download_url` for an update).
2. Open a PR. CI runs `cargo run -p catalog-validator -- ../catalog.json`.
3. A maintainer reviews and merges. The catalog is live immediately (the client
   fetches `raw.githubusercontent.com/.../main/catalog.json` on next open).