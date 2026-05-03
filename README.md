# IBKR Rust CLI

`ibkrctl` is an OAuth-only IBKR CLI for Airflow tasks. Airflow invokes one command, reads exit status, and handles scheduling or retries.

Success writes JSON to stdout and exits `0`. Failures write diagnostics to stderr and exit nonzero.
Airflow should own persistence: use stdout or `--output`, then let the DAG write results to storage.

## Explicit Non-Goals

- No Client Portal Gateway.
- No HTTP job API.
- No Redis queue. Redis is used only as an optional OAuth Live Session Token cache.
- No Kafka.
- No WebSocket streaming.
- No watchlist, scanner, or broad contract lookup modules. Stock symbol-to-conid lookup is supported.
- No automatic fallback auth path.

## Example

For local development, copy `.env.example` to `.env` and fill in the IBKR values. `ibkrctl` loads `.env` automatically at startup. On a server, provide the same `IBKR_*` variables through the process environment or your scheduler's secret manager.

### 1. Generate OAuth materials

This creates the local OpenSSL materials required by the IBKR OAuth setup. Send only `public_signature.pem`, `public_encryption.pem`, and `dhparam.pem` to IBKR. Never send or commit the generated private key files.

```bash
# Local Development
cargo run -p worker --bin ibkrctl -- oauth generate-materials --out-dir ./secrets/ibkr-oauth

# Server Usage
ibkrctl oauth generate-materials --out-dir ./secrets/ibkr-oauth
```

Use `--force` only when you intentionally want to replace existing files:

```bash
# Local Development
cargo run -p worker --bin ibkrctl -- oauth generate-materials --out-dir ./secrets/ibkr-oauth --force

# Server Usage
ibkrctl oauth generate-materials --out-dir ./secrets/ibkr-oauth --force
```

### 2. Initialize the brokerage session

This asks IBKR to initialize the authenticated brokerage session before protected account, order, or market-data calls. It uses the configured OAuth credentials and writes IBKR's JSON response to stdout.

```bash
# Local Development
cargo run -p worker --bin ibkrctl -- init-session

# Server Usage
ibkrctl init-session
```

### 3. Look up a stock conid

This calls IBKR's stock lookup endpoint and maps a stock symbol to one contract id using the current filters. If multiple contracts match, the command fails so the caller can provide a more specific `--exchange` or filter choice.

```bash
# Local Development
cargo run -p worker --bin ibkrctl -- stock-conid --symbol AAPL --exchange NASDAQ

# Server Usage
ibkrctl stock-conid --symbol AAPL --exchange NASDAQ
```

### 4. Fetch historical bars

This requests historical market-data bars for a known IBKR conid. By default the JSON response goes to stdout; use `--output` when Airflow should hand a file to a downstream task.

```bash
# Local Development
cargo run -p worker --bin ibkrctl -- fetch-history --conid 265598 --period 1d --bar 1min

# Server Usage
ibkrctl fetch-history --conid 265598 --period 1d --bar 1min --output /tmp/ibkr-history.json
```

### 5. Place an order

Order placement is non-interactive. The order file contains one order object or an array of order objects. If IBKR returns warning prompts, the answers file must explicitly accept them by message substring or message id.

```json
{
  "conid": 265598,
  "side": "BUY",
  "quantity": 1,
  "order_type": "MKT",
  "acct_id": "DU123456",
  "coid": "example-20260503-0001"
}
```

```json
{
  "price exceeds the Percentage constraint": true,
  "o354": true
}
```

```bash
ibkrctl order place --account-id DU123456 --orders-file ./order.json --answers-file ./answers.json
```

To inspect IB Algo strategies available for a contract, query the Web API algo endpoint:

```bash
ibkrctl order algos --conid 265598 --algo Adaptive --algo Vwap --add-description --add-params --pretty
```

Algo orders use the same order placement command with `strategy` and `strategy_parameters` in the order JSON:

```json
{
  "conid": 265598,
  "side": "BUY",
  "quantity": 100,
  "order_type": "LMT",
  "price": 185.5,
  "acct_id": "DU123456",
  "tif": "DAY",
  "strategy": "Vwap",
  "strategy_parameters": {
    "maxPctVol": 0.1,
    "startTime": "09:30:00 EST",
    "endTime": "15:30:00 EST",
    "allowPastEndTime": true
  }
}
```

## Configuration

Configuration comes only from `.env` and process environment variables. Local development can use a `.env` file in the current directory. Server and Airflow usage should provide the same variables through the runtime environment or a secret manager.

```sh
cp .env.example .env
```

```text
IBKR_BASE_URL=https://api.ibkr.com/v1/api
IBKR_CONSUMER_KEY=<from IBKR>
IBKR_REALM=limited_poa
IBKR_ACCESS_TOKEN=<from IBKR self-service portal>
IBKR_ACCESS_TOKEN_SECRET=<from IBKR self-service portal>
IBKR_SIGNATURE_KEY_PATH=/secure/path/private_signature.pem
IBKR_ENCRYPTION_KEY_PATH=/secure/path/private_encryption.pem
IBKR_DH_PARAM_PATH=/secure/path/dhparam.pem
IBKR_TIMEOUT_SECONDS=30
IBKR_LST_CACHE_MODE=redis
IBKR_REDIS_URL=redis://user:password@redis.example.internal:6379/0
IBKR_REDIS_KEY_PREFIX=ibkr:oauth:lst
IBKR_LST_REFRESH_SKEW_SECONDS=60
IBKR_LST_LOCK_TTL_SECONDS=15
```

## Live Session Token Cache

`ibkrctl` routes all Live Session Token lookup through one OAuth cache provider. Cache modes are:

- `redis`: share validated LSTs across short-lived CLI processes. If Redis is unavailable, the command fails instead of silently requesting an uncached token.
- `memory`: reuse the LST only inside the current CLI process. This is the default.
- `disabled`: request a fresh LST for each protected REST call.

Redis stores the validated LST payload with a TTL ending before IBKR expiry. The cache key uses hashed fingerprints of the base URL, consumer key, access token, and realm, and never stores raw identifiers in the key. Treat Redis as a secrets-adjacent system: use ACLs, private networking or TLS, restricted database access, and avoid broad shared Redis instances.

## Airflow Example

```python
BashOperator(
    task_id="lookup_aapl_conid",
    bash_command="ibkrctl stock-conid --symbol AAPL --output /tmp/aapl-conid.json",
    env={
        "IBKR_BASE_URL": "{{ var.value.ibkr_base_url }}",
        "IBKR_CONSUMER_KEY": "{{ var.value.ibkr_consumer_key }}",
        "IBKR_REALM": "limited_poa",
        "IBKR_ACCESS_TOKEN": "{{ var.value.ibkr_access_token }}",
        "IBKR_ACCESS_TOKEN_SECRET": "{{ var.value.ibkr_access_token_secret }}",
        "IBKR_SIGNATURE_KEY_PATH": "/opt/airflow/secrets/private_signature.pem",
        "IBKR_ENCRYPTION_KEY_PATH": "/opt/airflow/secrets/private_encryption.pem",
        "IBKR_DH_PARAM_PATH": "/opt/airflow/secrets/dhparam.pem",
        "IBKR_LST_CACHE_MODE": "redis",
        "IBKR_REDIS_URL": "{{ var.value.ibkr_redis_url }}",
    },
)
```

```python
BashOperator(
    task_id="fetch_aapl_history",
    bash_command=(
        "ibkrctl fetch-history --conid 265598 --period 1d --bar 1min "
        "--output /tmp/ibkr-history.json"
    ),
    env={
        "IBKR_BASE_URL": "{{ var.value.ibkr_base_url }}",
        "IBKR_CONSUMER_KEY": "{{ var.value.ibkr_consumer_key }}",
        "IBKR_REALM": "limited_poa",
        "IBKR_ACCESS_TOKEN": "{{ var.value.ibkr_access_token }}",
        "IBKR_ACCESS_TOKEN_SECRET": "{{ var.value.ibkr_access_token_secret }}",
        "IBKR_SIGNATURE_KEY_PATH": "/opt/airflow/secrets/private_signature.pem",
        "IBKR_ENCRYPTION_KEY_PATH": "/opt/airflow/secrets/private_encryption.pem",
        "IBKR_DH_PARAM_PATH": "/opt/airflow/secrets/dhparam.pem",
        "IBKR_LST_CACHE_MODE": "redis",
        "IBKR_REDIS_URL": "{{ var.value.ibkr_redis_url }}",
    },
)
```

Airflow should treat exit code `0` as success and any nonzero exit code as task failure.
Run `init-session` before protected IBKR calls. If IBKR returns `Bad Request: no bridge`, run `init-session` again; Redis caches OAuth Live Session Tokens, not brokerage bridge state.

## API Surface

Implemented REST/CLI commands:

- `auth-status`: `iserver/auth/status`
- `init-session`: `iserver/auth/ssodh/init`
- `fetch-history`: `iserver/marketdata/history`
- `stock-conid`: `trsrv/stocks`
- `accounts`: `portfolio/accounts`
- `account-summary`: `iserver/account/{account_id}/summary`
- `portfolio-summary`: `portfolio/{account_id}/summary`
- `ledger`: `portfolio/{account_id}/ledger`
- `positions`: `portfolio/{account_id}/positions/{page}`
- `live-orders`: `iserver/account/orders`
- `order algos`: `iserver/contract/{conid}/algos`
- `order place`: `iserver/account/{account_id}/orders`
- `order whatif`: `iserver/account/{account_id}/orders/whatif`
- `order reply`: `iserver/reply/{reply_id}`
- `order cancel`: `iserver/account/{account_id}/order/{order_id}`
- `order modify`: `iserver/account/{account_id}/order/{order_id}`
- `order status`: `iserver/account/order/status/{order_id}`

Missing but relevant functions:

- Account PnL endpoints.
- Typed request/response models for accounts, positions, live orders, auth, and session calls.

Historical data, stock conid lookup, and order placement have non-trivial typed request shapes today. The other implemented endpoints are still thin JSON passthroughs with little or no request structure.
