# chatwoot-cli

Auto-generated Chatwoot CLI from the OpenAPI schema. Designed for LLM discovery and direct scripting.

## Install

### Install script (macOS arm64 + Linux x86_64)

```bash
curl -fsSL https://raw.githubusercontent.com/radjathaher/chatwoot-cli/main/scripts/install.sh | bash
```

### Homebrew (binary, macOS arm64 only)

```bash
brew tap radjathaher/tap
brew install chatwoot-cli
```

### Build from source

```bash
cargo build --release
./target/release/chatwoot --help
```

## Auth

Chatwoot uses the `api_access_token` header.

```bash
export CHATWOOT_API_TOKEN="<your_token>"
```

Alias:

```bash
export CHATWOOT_API_ACCESS_TOKEN="<your_token>"
```

Base URL (default `https://app.chatwoot.com`):

```bash
export CHATWOOT_BASE_URL="https://app.chatwoot.com"
```

## Discovery (LLM-friendly)

```bash
chatwoot list --json
chatwoot describe conversations conversation-list --json
chatwoot tree --json
```

Human help:

```bash
chatwoot --help
chatwoot conversations --help
chatwoot conversations conversation-list --help
```

## Examples

List agents:

```bash
chatwoot agents get-account-agents --account-id 123 --pretty
```

Create a contact (client API):

```bash
chatwoot contacts-api create-a-contact \
  --inbox-identifier "<inbox_identifier>" \
  --input-name "Jane Doe" \
  --input-email "jane@example.com" \
  --pretty
```

Update account (form-encoded):

```bash
chatwoot account update-account --id 1 --input-name "New Name" --pretty
```

## Update schema + command tree

```bash
tools/fetch_schema.py --out schemas/swagger.json
tools/gen_command_tree.py --schema schemas/swagger.json --out schemas/command_tree.json
cargo build
```

## Notes

- `--body` accepts raw JSON; `--input-*` maps top-level request fields.
- Endpoints with required auth will error if `CHATWOOT_API_TOKEN` is missing.
