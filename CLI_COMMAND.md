# ibkrctl CLI Command Reference

`ibkrctl` is an OAuth-only Interactive Brokers CLI intended primarily for non-interactive jobs such as Airflow tasks. It also provides a separately listed interactive utility for placing a manual VWAP order. The CLI wraps selected IBKR REST endpoints, signs requests with OAuth credentials, prints successful responses as JSON, and exits nonzero on failures.

This document describes every command currently exposed by the CLI.

## Basic Invocation

Installed binary:

```bash
ibkrctl [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS]
```

Examples:

```bash
ibkrctl auth-status --pretty
ibkrctl stock-conid --symbol AAPL --exchange NASDAQ
ibkrctl fetch-history --conid 265598 --period 1d --bar 1min --output /tmp/history.json
ibkrctl trades --account-id DU123456 --days 7 --pretty
```

## Exit And Output Contract

Successful commands exit with status `0`.

Most commands write JSON to stdout by default. If `--output <PATH>` is supplied, JSON is written to that file instead.

Failures write diagnostics to stderr and exit with a nonzero status. The CLI does not persist results itself; callers should capture stdout or use `--output`.

Two commands are exceptions to the JSON-output rule:

- `env` writes plain text containing supported `IBKR_*` variables and their effective values.
- `oauth generate-materials` writes status notices to stderr and creates files on disk.

`quick-vwap-order` prompts through the controlling terminal and writes its review information to stderr. If the order is submitted, its final IBKR response follows the normal JSON stdout or `--output` contract.

## Global Options

Global options can be placed before or after the subcommand because they are marked as global Clap arguments.

### `--env-file <PATH>`

Loads environment variables from the specified dotenv file before running the command.

```bash
ibkrctl --env-file /secure/path/ibkr.env auth-status
```

Without `--env-file`, the CLI attempts to load `.env` from the current directory or its parents. Missing default `.env` files are ignored.

### `--output <PATH>`

Writes command output to a file instead of stdout.

```bash
ibkrctl accounts --output /tmp/accounts.json
```

This applies to JSON commands and also to `env` plain-text output.

### `--pretty`

Pretty-prints JSON output.

```bash
ibkrctl live-orders --pretty
```

This only affects JSON output. It has no effect on `env` or `oauth generate-materials`.

### `--timeout-seconds <SECONDS>`

Overrides the HTTP timeout configured by `IBKR_TIMEOUT_SECONDS`.

```bash
ibkrctl --timeout-seconds 60 fetch-history --conid 265598 --period 1d --bar 1min
```

The default timeout is 30 seconds.

### `--version` / `-V`

Prints the CLI, worker crate, and core crate versions without loading dotenv files or configuration.

```bash
ibkrctl --version
```

Output format:

```text
ibkrctl <worker-version>
worker <worker-version>
core <core-version>
```

## Configuration

All protected IBKR API commands require OAuth configuration. Configuration is read from dotenv files and process environment variables.

Supported variables:

| Variable | Required | Default | Description |
| --- | --- | --- | --- |
| `IBKR_BASE_URL` | No | `https://api.ibkr.com/v1/api` | Base URL for IBKR Web API calls. |
| `IBKR_CONSUMER_KEY` | Yes | None | OAuth consumer key from IBKR. |
| `IBKR_REALM` | No | `limited_poa` | OAuth realm. |
| `IBKR_ACCESS_TOKEN` | Yes | None | OAuth access token from IBKR. |
| `IBKR_ACCESS_TOKEN_SECRET` | Yes | None | OAuth access token secret from IBKR. |
| `IBKR_SIGNATURE_KEY_PATH` | Yes | None | Path to private signature key PEM. |
| `IBKR_ENCRYPTION_KEY_PATH` | Yes | None | Path to private encryption key PEM. |
| `IBKR_DH_PARAM_PATH` | Yes | None | Path to DH parameter PEM. |
| `IBKR_TIMEOUT_SECONDS` | No | `30` | HTTP timeout in seconds. Can be overridden by `--timeout-seconds`. |
| `IBKR_ACCOUNT_ID` | No | None | Default account id offered by the interactive `quick-vwap-order` form. |
| `IBKR_QUICK_ORDER_PREFIX` | No | `quick-vwap` | Default client-order-id prefix offered by `quick-vwap-order`. |
| `IBKR_DATABASE` | No | None | PostgreSQL URL used by `stock-conid` for local conid cache lookup/upsert and by `fetch-history` for best-effort bar persistence. |
| `IBKR_LST_CACHE_MODE` | No | `memory` | Live Session Token cache mode: `memory`, `redis`, or `disabled`. |
| `IBKR_REDIS_URL` | Required when Redis cache is used | None | Redis URL for shared Live Session Token cache. |
| `IBKR_REDIS_KEY_PREFIX` | No | `ibkr:oauth:lst` | Redis key prefix for cached Live Session Tokens. |
| `IBKR_LST_REFRESH_SKEW_SECONDS` | No | `60` | Seconds before expiry when a cached Live Session Token should be refreshed. |
| `IBKR_LST_LOCK_TTL_SECONDS` | No | `15` | Redis lock TTL used while refreshing a Live Session Token. |

Commands that do not require OAuth client configuration:

- `ibkrctl env`
- `ibkrctl oauth generate-materials`
- `ibkrctl --version`

All other commands require the OAuth variables listed above.

## Live Session Token Cache

Protected REST calls use an OAuth Live Session Token. Cache behavior is controlled by `IBKR_LST_CACHE_MODE`.

`memory` is the default. It reuses a token only within the current process. Because `ibkrctl` usually runs as a short-lived process, this mostly helps commands that make more than one protected request.

`redis` shares validated tokens across CLI processes. Use this for schedulers that invoke many short-lived `ibkrctl` processes. If Redis is unavailable, the command fails instead of silently bypassing the configured shared cache.

`disabled` requests a fresh token for each protected REST call.

The cache stores Live Session Token payloads, not brokerage bridge state. If IBKR returns a bridge/session error, run `init-session` again.

## Command Overview

Top-level commands:

| Command | Purpose |
| --- | --- |
| `oauth generate-materials` | Generate local OAuth key and DH parameter materials. |
| `auth-status` | Return IBKR authentication status. |
| `env` | Print supported environment variables with secrets masked. |
| `init-session` | Initialize the authenticated brokerage session. |
| `tickle` | Keep an already-initialized brokerage session alive. |
| `fetch-history` | Fetch historical market-data bars for a conid. |
| `stock-conid` | Resolve a stock symbol to an active IBKR conid, optionally using a PostgreSQL cache. |
| `accounts` | Initialize and list accounts available to portfolio endpoints. |
| `brokerage-accounts` | Initialize and list accounts available to IServer trading endpoints. |
| `account-pnl` | Fetch P&L for the currently selected account and its models. |
| `account-summary` | Fetch IServer account summary for one account. |
| `portfolio-summary` | Fetch portfolio summary for one account. |
| `ledger` | Fetch portfolio ledger for one account. |
| `positions` | Fetch portfolio positions for one account and page. |
| `positions-live` | Fetch uncached, near-real-time portfolio positions through REST. |
| `trades` | Fetch recent trade executions. |
| `live-orders` | Fetch live orders. |
| `order algos` | Fetch available IB Algo strategies and parameters for a contract. |
| `order place` | Submit one or more orders and automatically handle configured confirmations. |
| `order whatif` | Submit one or more orders to the IBKR what-if endpoint. |
| `order reply` | Reply to an IBKR order confirmation prompt. |
| `order cancel` | Cancel an order. |
| `order modify` | Modify an order and automatically handle configured confirmations. |
| `order status` | Fetch status for one order. |
| `order fee-plan` | Compute the local Tiered/Fixed fee-plan decision for one or more limit orders. |

Interactive utilities are separated from the normal command list in `ibkrctl --help`:

| Command | Purpose |
| --- | --- |
| `quick-vwap-order` | Prompt for, review, and optionally submit one manual VWAP limit order. |

## `oauth generate-materials`

Generates the OpenSSL materials needed for IBKR OAuth setup.

```bash
ibkrctl oauth generate-materials --out-dir <DIR> [--force]
```

Options:

| Option | Required | Description |
| --- | --- | --- |
| `--out-dir <DIR>` | Yes | Directory where generated OAuth files will be written. Created if missing. |
| `--force` | No | Allows overwriting generated files if they already exist. |

Generated files:

| File | Share With IBKR | Description |
| --- | --- | --- |
| `dhparam.pem` | Yes | Diffie-Hellman parameters. |
| `public_signature.pem` | Yes | Public signature key. |
| `public_encryption.pem` | Yes | Public encryption key. |
| `private_signature.pem` | No | Private signature key used by the CLI. |
| `private_encryption.pem` | No | Private encryption key used by the CLI. |
| `private_signature.pk8` | No | PKCS#8 version of the private signature key. |
| `private_encryption.pk8` | No | PKCS#8 version of the private encryption key. |

On Unix systems, private files are set to mode `0600`.

The command refuses to overwrite any generated file unless `--force` is passed.

Examples:

```bash
ibkrctl oauth generate-materials --out-dir ./secrets/ibkr-oauth
ibkrctl oauth generate-materials --out-dir ./secrets/ibkr-oauth --force
```

## `env`

Prints the supported `IBKR_*` configuration variables and their current effective values.

```bash
ibkrctl env
```

This command does not validate OAuth credentials and does not call IBKR.

Secret-like values are truncated. Values with eight characters or fewer are displayed as `...`; longer values are displayed as the first four and last four characters separated by `...`.

Secret-like variables:

- `IBKR_CONSUMER_KEY`
- `IBKR_ACCESS_TOKEN`
- `IBKR_ACCESS_TOKEN_SECRET`
- `IBKR_DATABASE`
- `IBKR_REDIS_URL`

Example:

```bash
ibkrctl --env-file /secure/path/ibkr.env env
```

Example output:

```text
IBKR_BASE_URL=https://api.ibkr.com/v1/api
IBKR_CONSUMER_KEY=abcd...wxyz
IBKR_REALM=limited_poa
IBKR_ACCESS_TOKEN=tokn...1234
IBKR_ACCESS_TOKEN_SECRET=secr...5678
IBKR_SIGNATURE_KEY_PATH=/secure/path/private_signature.pem
IBKR_ENCRYPTION_KEY_PATH=/secure/path/private_encryption.pem
IBKR_DH_PARAM_PATH=/secure/path/dhparam.pem
IBKR_TIMEOUT_SECONDS=30
IBKR_ACCOUNT_ID=DU123456
IBKR_QUICK_ORDER_PREFIX=quick-vwap
IBKR_DATABASE=post...name
IBKR_LST_CACHE_MODE=redis
IBKR_REDIS_URL=redi.../0
IBKR_REDIS_KEY_PREFIX=ibkr:oauth:lst
IBKR_LST_REFRESH_SKEW_SECONDS=60
IBKR_LST_LOCK_TTL_SECONDS=15
```

## `auth-status`

Fetches IBKR authentication status.

```bash
ibkrctl auth-status
```

Endpoint:

```text
GET iserver/auth/status
```

Output:

The raw JSON response from IBKR.

Example:

```bash
ibkrctl auth-status --pretty
```

Use this command to inspect the current authentication/session state before making protected account, market-data, or order calls.

## `init-session`

Initializes the authenticated brokerage session.

```bash
ibkrctl init-session [--compete]
```

Options:

| Option | Required | Default | Description |
| --- | --- | --- | --- |
| `--compete` | No | `true` | Passed to IBKR session initialization. This is currently a flag and the parsed value defaults to `true`, so there is no CLI form for setting it to `false`. |

Endpoint:

```text
POST iserver/auth/ssodh/init
```

Output:

The raw JSON response from IBKR.

Examples:

```bash
ibkrctl init-session
ibkrctl init-session --compete
```

Run this before protected IServer account, market-data, or order commands, especially if IBKR reports that the brokerage bridge is not initialized.

## `tickle`

Pings IBKR to keep an already-initialized brokerage session alive. This does not reauthenticate or initialize a missing brokerage session; use `init-session` for recovery.

```bash
ibkrctl tickle
```

Endpoint:

```text
POST tickle
```

Output:

The raw JSON response from IBKR.

For long-running jobs, call this command about once every 60 seconds rather than before every API request.

## `fetch-history`

Fetches historical market-data bars for a known conid.

```bash
ibkrctl fetch-history --conid <CONID> --period <PERIOD> --bar <BAR> [--exchange <EXCHANGE>] [--outside-rth <true|false>] [--start-time <START_TIME>]
```

Options:

| Option | Required | Description |
| --- | --- | --- |
| `--conid <CONID>` | Yes | IBKR contract id. |
| `--period <PERIOD>` | Yes | IBKR history period, such as `1d`, `1w`, `1m`, depending on IBKR endpoint support. |
| `--bar <BAR>` | Yes | Bar size, such as `1min`, depending on IBKR endpoint support. |
| `--exchange <EXCHANGE>` | No | Exchange parameter forwarded to IBKR. |
| `--outside-rth <true|false>` | No | Whether to include data outside regular trading hours. |
| `--start-time <START_TIME>` | No | Start time string forwarded to IBKR. |

Endpoint:

```text
GET iserver/marketdata/history
```

Output:

The raw JSON response from IBKR.

Database behavior:

If `IBKR_DATABASE` is set and connectable, returned bars are upserted into `warehouse.ibkr_bars` with `(conid, bar_start, bar_end)` as the conflict key. The command still returns the raw IBKR JSON. If the database is unavailable or persistence fails, the command writes a warning to stderr and still returns the IBKR response.

The expected table is:

```sql
CREATE TABLE IF NOT EXISTS warehouse.ibkr_bars (
    conid     bigint NOT NULL,
    bar_start timestamptz NOT NULL,
    bar_end   timestamptz NOT NULL,
    open      numeric NOT NULL,
    high      numeric NOT NULL,
    low       numeric NOT NULL,
    close     numeric NOT NULL,
    volume    bigint,
    CONSTRAINT ibkr_bars_pk PRIMARY KEY (conid, bar_start, bar_end)
);

CREATE INDEX IF NOT EXISTS ibkr_bars_conid_day_idx
    ON warehouse.ibkr_bars (conid, bar_start, bar_end);
```

Examples:

```bash
ibkrctl fetch-history --conid 265598 --period 1d --bar 1min
ibkrctl fetch-history --conid 265598 --period 1w --bar 1h --outside-rth true --pretty
ibkrctl fetch-history --conid 265598 --period 1d --bar 1min --output /tmp/ibkr-history.json
```

## `stock-conid`

Resolves a stock symbol to an IBKR conid.

```bash
ibkrctl stock-conid --symbol <SYMBOL> [--exchange <EXCHANGE>] [--default-filtering <true|false>]
```

Options:

| Option | Required | Default | Description |
| --- | --- | --- | --- |
| `--symbol <SYMBOL>` | Yes | Stock symbol. The CLI uppercases this value before lookup. |
| `--exchange <EXCHANGE>` | No | Exchange filter. Use this when multiple active contracts may match a symbol. |
| `--default-filtering <true|false>` | No | `true` | Enables the stock lookup model's default filtering behavior. |

Endpoint:

```text
GET trsrv/stocks
```

Database behavior:

If `IBKR_DATABASE` is set, the command first searches a local PostgreSQL conid cache. If an active cached match is found, that cached result is returned without calling IBKR.

If no cached match is found, or if the database is unavailable, the command falls back to the IBKR API. When a database connection exists and the IBKR lookup succeeds, the result is upserted into the cache.

If the local conid lookup finds an ambiguous or invalid local result, the command can fail instead of falling back.

Output:

A selected stock lookup result serialized as JSON. When a cached database result is used, the output is normalized from the cached row.

Examples:

```bash
ibkrctl stock-conid --symbol AAPL
ibkrctl stock-conid --symbol AAPL --exchange NASDAQ
ibkrctl stock-conid --symbol BRK.B --default-filtering false --pretty
```

## `accounts`

Initializes and lists accounts available to portfolio endpoints.

```bash
ibkrctl accounts
```

Endpoint:

```text
GET portfolio/accounts
```

Output:

The raw JSON response from IBKR.

Example:

```bash
ibkrctl accounts --pretty
```

Call this command before account-scoped `portfolio-summary`, `ledger`, `positions`, or `positions-live` requests. It is different from `brokerage-accounts`.

## `brokerage-accounts`

Initializes and lists accounts available to IServer trading endpoints.

```bash
ibkrctl brokerage-accounts
```

Endpoint:

```text
GET iserver/accounts
```

Output:

The raw JSON response from IBKR.

Example:

```bash
ibkrctl brokerage-accounts --pretty
```

Use this as the account-context preflight for `account-summary`, `account-pnl`, and the trade-execution warm-up workflow. It does not replace the `accounts` preflight required by portfolio endpoints.

## `account-pnl`

Fetches P&L for the currently selected account and its models.

```bash
ibkrctl account-pnl
```

Endpoint:

```text
GET iserver/account/pnl/partitioned
```

Output:

The raw JSON response from IBKR.

Example:

```bash
ibkrctl brokerage-accounts
ibkrctl account-pnl --pretty
```

## `account-summary`

Fetches IServer account summary for one account.

```bash
ibkrctl account-summary --account-id <ACCOUNT_ID>
```

Options:

| Option | Required | Description |
| --- | --- | --- |
| `--account-id <ACCOUNT_ID>` | Yes | IBKR account id, such as `DU123456`. |

Endpoint:

```text
GET iserver/account/{account_id}/summary
```

Output:

The raw JSON response from IBKR.

Example:

```bash
ibkrctl brokerage-accounts
ibkrctl account-summary --account-id DU123456 --pretty
```

## `portfolio-summary`

Fetches portfolio summary for one account.

```bash
ibkrctl portfolio-summary --account-id <ACCOUNT_ID>
```

Options:

| Option | Required | Description |
| --- | --- | --- |
| `--account-id <ACCOUNT_ID>` | Yes | IBKR account id. |

Endpoint:

```text
GET portfolio/{account_id}/summary
```

Output:

The raw JSON response from IBKR.

Example:

```bash
ibkrctl portfolio-summary --account-id DU123456 --pretty
```

## `ledger`

Fetches portfolio ledger data for one account.

```bash
ibkrctl ledger --account-id <ACCOUNT_ID>
```

Options:

| Option | Required | Description |
| --- | --- | --- |
| `--account-id <ACCOUNT_ID>` | Yes | IBKR account id. |

Endpoint:

```text
GET portfolio/{account_id}/ledger
```

Output:

The raw JSON response from IBKR.

Example:

```bash
ibkrctl ledger --account-id DU123456 --pretty
```

## `positions`

Fetches portfolio positions for one account and page.

```bash
ibkrctl positions --account-id <ACCOUNT_ID> [--page <PAGE>]
```

Options:

| Option | Required | Default | Description |
| --- | --- | --- | --- |
| `--account-id <ACCOUNT_ID>` | Yes | None | IBKR account id. |
| `--page <PAGE>` | No | `0` | Positions page number. |

Endpoint:

```text
GET portfolio/{account_id}/positions/{page}
```

Output:

The raw JSON response from IBKR.

Examples:

```bash
ibkrctl positions --account-id DU123456
ibkrctl positions --account-id DU123456 --page 1 --pretty
```

## `positions-live`

Fetches uncached, near-real-time positions for one account through the REST API. This command does not open a WebSocket.

```bash
ibkrctl positions-live --account-id <ACCOUNT_ID> [--model <MODEL>] [--sort <FIELD>] [--direction a|d]
```

Options:

| Option | Required | Default | Description |
| --- | --- | --- | --- |
| `--account-id <ACCOUNT_ID>` | Yes | None | IBKR account id. |
| `--model <MODEL>` | No | None | Optional model filter. |
| `--sort <FIELD>` | No | None | Optional field used to sort positions. |
| `--direction <a|d>` | No | None | Optional ascending (`a`) or descending (`d`) sort direction. |

Endpoint:

```text
GET portfolio2/{account_id}/positions
```

Output:

The raw JSON response from IBKR.

Examples:

```bash
ibkrctl accounts
ibkrctl positions-live --account-id DU123456 --pretty
ibkrctl positions-live --account-id DU123456 --model Growth --sort position --direction d --pretty
```

## `live-orders`

Fetches live orders.

```bash
ibkrctl live-orders [--account-id <ACCOUNT_ID>] [--force]
```

Options:

| Option | Required | Default | Description |
| --- | --- | --- | --- |
| `--account-id <ACCOUNT_ID>` | No | None | Optional account id filter. |
| `--force` | No | `true` | Forwarded to the IBKR live orders endpoint. This is currently a flag and the parsed value defaults to `true`, so there is no CLI form for setting it to `false`. |

Endpoint:

```text
GET iserver/account/orders
```

Output:

The raw JSON response from IBKR.

Examples:

```bash
ibkrctl live-orders
ibkrctl live-orders --account-id DU123456 --force --pretty
```

## `trades`

Fetches recent trade executions for the current day and prior days. This is trade execution history, not market-data bar history; use `fetch-history` for historical market-data bars.

IBKR supports up to 7 days through this endpoint and advises calling it once per session.

IBKR may return an empty trade list until account context has been loaded for the session. For scripts and Airflow jobs, warm the session in this order:

```bash
ibkrctl init-session
ibkrctl brokerage-accounts
ibkrctl trades
sleep 5
ibkrctl trades --account-id DU123456 --days 7 --pretty
```

The five-second wait is explicit orchestration policy; the CLI does not sleep, retry, or silently fall back. The initial unscoped `trades` request and the later scoped request therefore remain separately observable.

```bash
ibkrctl trades [--account-id <ACCOUNT_ID>] [--days <DAYS>]
```

Options:

| Option | Required | Default | Description |
| --- | --- | --- | --- |
| `--account-id <ACCOUNT_ID>` | No | None | Optional account id or allocation group filter. |
| `--days <DAYS>` | No | None | Number of days of executions to request, up to IBKR's maximum of 7. If omitted, IBKR returns the current day. |

Endpoint:

```text
GET iserver/account/trades/
```

Output:

The raw JSON response from IBKR.

Examples:

```bash
ibkrctl trades
ibkrctl trades --account-id DU123456 --days 7 --pretty
```

## `quick-vwap-order`

Interactively builds, reviews, and optionally submits one manual VWAP limit order. This is an operator utility, so it is listed under `INTERACTIVE UTILITIES` rather than the normal command list in top-level help.

```bash
ibkrctl quick-vwap-order [OPTIONS]
```

Options prefill the form; omitted values are prompted:

| Option | Required | Default | Description |
| --- | --- | --- | --- |
| `--account-id <ACCOUNT_ID>` | No | `IBKR_ACCOUNT_ID`, then `IBKRCTL_ACCOUNT_ID` | Account used to place the order. |
| `--ticker <TICKER>` | No | None | Stock ticker resolved to a conid through IBKR. |
| `--exchange <EXCHANGE>` | No | Automatic | Optional exchange used to disambiguate ticker resolution. |
| `--side <buy|sell>` | No | Interactive selection | Order side. |
| `--quantity <QUANTITY>` | No | None | Positive order quantity. |
| `--limit-price <PRICE>` | No | None | Positive limit price. |
| `--start-time <TIME>` | No | `15:30:00 US/Eastern` | Editable VWAP start time. |
| `--end-time <TIME>` | No | `16:00:00 US/Eastern` | Editable VWAP end time. |
| `--max-percent-volume <VALUE>` | No | `0.1` | VWAP maximum percentage of market volume. |
| `--client-order-id-prefix <PREFIX>` | No | `IBKR_QUICK_ORDER_PREFIX` or `quick-vwap` | Prefix used to construct the client order id. |
| `--max-replies <COUNT>` | No | `20` | Maximum IBKR confirmation prompts handled after submission. |

The generated order always uses these fixed fields:

```json
{
  "orderType": "LMT",
  "tif": "DAY",
  "strategy": "Vwap",
  "strategyParameters": {
    "allowPastEndTime": "0",
    "noTakeLiq": "0",
    "speedUp": "0"
  }
}
```

The utility resolves the ticker directly through `trsrv/stocks`, prints the complete order payload to stderr, and asks for final confirmation with **no** as the default. After submission, every warning from IBKR is presented as a separate confirmation that also defaults to **no**.

It does not connect to PostgreSQL and does not persist the order locally. The final IBKR JSON response follows the normal stdout or `--output` behavior.

Recommended workflow:

```bash
ibkrctl --env-file /secure/path/ibkr.env init-session
ibkrctl --env-file /secure/path/ibkr.env brokerage-accounts
ibkrctl --env-file /secure/path/ibkr.env quick-vwap-order
```

Values can be prefixed while retaining the review and confirmation steps:

```bash
ibkrctl quick-vwap-order \
  --account-id DU123456 \
  --ticker TQQQ \
  --side buy \
  --quantity 10 \
  --limit-price 70 \
  --start-time "15:30:00 US/Eastern" \
  --end-time "16:00:00 US/Eastern" \
  --max-percent-volume 0.1
```

## Order Command Overview

All order commands are nested under `order`.

```bash
ibkrctl order <ORDER_COMMAND> [OPTIONS]
```

Order subcommands:

| Command | Purpose |
| --- | --- |
| `order algos` | Inspect available IB Algo strategies for a contract. |
| `order place` | Place one or more orders from a JSON file. |
| `order whatif` | Submit one or more orders to what-if preview. |
| `order reply` | Confirm or reject a prompt reply id directly. |
| `order cancel` | Cancel an existing order. |
| `order modify` | Modify an existing order from a JSON file. |
| `order status` | Fetch status for an order id. |
| `order fee-plan` | Compute the local Tiered/Fixed fee-plan decision for one or more limit orders. |

Run `init-session` before protected order commands if IBKR has not already initialized the brokerage session.

## `order algos`

Fetches available IB Algo strategies and optional strategy metadata for a contract.

```bash
ibkrctl order algos --conid <CONID> [--algo <ALGO>]... [--add-description] [--add-params]
```

Options:

| Option | Required | Description |
| --- | --- | --- |
| `--conid <CONID>` | Yes | IBKR contract id. |
| `--algo <ALGO>` | No | Case-sensitive algo id. Repeat the flag to request multiple algos. |
| `--add-description` | No | Ask IBKR to include algo descriptions. |
| `--add-params` | No | Ask IBKR to include algo parameters. |

Endpoint:

```text
GET iserver/contract/{conid}/algos
```

Output:

The raw JSON response from IBKR.

Examples:

```bash
ibkrctl order algos --conid 265598
ibkrctl order algos --conid 265598 --algo Adaptive --algo Vwap --add-description --add-params --pretty
```

## `order place`

Places one or more orders from an inline JSON argument and automatically answers selected IBKR confirmation prompts using built-in defaults plus optional answer-file and inline JSON layers.

```bash
ibkrctl order place --account-id <ACCOUNT_ID> --orders-json <JSON> [--answers-file <PATH>] [--answers-json <JSON>] [--max-replies <N>]
```

Options:

| Option | Required | Default | Description |
| --- | --- | --- | --- |
| `--account-id <ACCOUNT_ID>` | Yes | None | IBKR account id used in the endpoint path. |
| `--orders-json <JSON>` | Yes | None | Inline JSON containing one order object or an array of order objects. |
| `--answers-file <PATH>` | No | Built-in defaults plus optional `IBKR_ORDERS_ANSWER_JSON` file | JSON map used to override or extend lower-priority answers for IBKR confirmation prompts. |
| `--answers-json <JSON>` | No | Built-in defaults plus optional answer files | Inline JSON map used to override or extend lower-priority answers for IBKR confirmation prompts. |
| `--max-replies <N>` | No | `20` | Maximum number of automatic confirmation reply loops before failing. |

Endpoint:

```text
POST iserver/account/{account_id}/orders
```

Order JSON format:

The JSON may contain either a single order object:

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

Or an array of order objects:

```json
[
  {
    "conid": 265598,
    "side": "BUY",
    "quantity": 1,
    "order_type": "MKT",
    "acct_id": "DU123456"
  },
  {
    "conid": 265598,
    "side": "SELL",
    "quantity": 1,
    "order_type": "LMT",
    "price": 220,
    "acct_id": "DU123456"
  }
]
```

Accepted order fields are listed in [Order JSON Fields](#order-json-fields).

Answers format:

The answers JSON is a JSON object whose keys are either IBKR `messageId` values or substrings expected in prompt messages. Values are booleans.

```json
{
  "o354": true,
  "price exceeds the Percentage constraint": true,
  "missing market data": false
}
```

Built-in answers:

The CLI accepts these known prompts by default, based on the `QuestionType` constants used by `ibind`'s `rest_04_place_order.py` and `rest_07_bracket_orders.py` examples:

| Constant | Keys | Default |
| --- | --- | --- |
| `PRICE_PERCENTAGE_CONSTRAINT` | `o163`, `price exceeds the Percentage constraint` | `true` |
| `ORDER_VALUE_LIMIT` | `o451`, `exceeds the Total Value Limit` | `true` |
| `MISSING_MARKET_DATA` | `o354`, `You are submitting an order without market data` | `true` |
| `STOP_ORDER_RISKS` | `o10331`, `You are about to submit a stop order` | `true` |

Answer sources are merged in this order: built-in defaults, optional base file path from `IBKR_ORDERS_ANSWER_JSON`, optional `--answers-file`, then optional `--answers-json`. Later sources override duplicate keys. If `IBKR_ORDERS_ANSWER_JSON` is unset, empty, or points to a missing file, it is treated as `{}`. If that file exists but is unreadable or contains invalid JSON, the command fails. Unknown prompts still fail unless supplied explicitly.

Prompt matching:

1. If IBKR returns a `messageId` or first `messageIds` entry, that id is checked first.
2. If no id match is found, each answer key is checked as a substring of the cleaned prompt message.
3. A matching value of `true` sends `order reply --confirmed true` internally.
4. A matching value of `false` rejects the order and the command fails.
5. If no answer matches, the command fails.
6. If more than `--max-replies` confirmation loops are required, the command fails.

Output:

If no prompt remains, the final IBKR JSON response is printed. If IBKR returns a single-item array as the final order response, the CLI unwraps that single item.

Examples:

```bash
ibkrctl order place --account-id DU123456 --orders-json '{"conid":265598,"side":"BUY","quantity":1,"order_type":"MKT","acct_id":"DU123456"}' --answers-file ./answers.json
ibkrctl order place --account-id DU123456 --orders-json '[{"conid":265598,"side":"BUY","quantity":1,"order_type":"MKT","acct_id":"DU123456"}]' --answers-file ./answers.json --max-replies 5 --pretty
ibkrctl order place --account-id DU123456 --orders-json '{"conid":265598,"side":"BUY","quantity":1,"order_type":"LMT","price":185.5,"acct_id":"DU123456"}' --answers-json '{"o354":true}' --pretty
ibkrctl order place --account-id DU123456 --orders-json '{"conid":265598,"side":"BUY","quantity":1,"order_type":"MKT","acct_id":"DU123456"}' --answers-json '{"o354":false}'
```

## `order whatif`

Submits one or more orders to the IBKR what-if endpoint.

```bash
ibkrctl order whatif --account-id <ACCOUNT_ID> --orders-json <JSON>
```

Options:

| Option | Required | Description |
| --- | --- | --- |
| `--account-id <ACCOUNT_ID>` | Yes | IBKR account id used in the endpoint path. |
| `--orders-json <JSON>` | Yes | Inline JSON containing one order object or an array of order objects. |

Endpoint:

```text
POST iserver/account/{account_id}/orders/whatif
```

Output:

The raw JSON response from IBKR.

Example:

```bash
ibkrctl order whatif --account-id DU123456 --orders-json '{"conid":265598,"side":"BUY","quantity":1,"order_type":"LMT","price":185.5,"acct_id":"DU123456"}' --pretty
```

## `order reply`

Replies directly to an IBKR order confirmation prompt.

```bash
ibkrctl order reply --reply-id <REPLY_ID> [--confirmed]
```

Options:

| Option | Required | Description |
| --- | --- | --- |
| `--reply-id <REPLY_ID>` | Yes | Reply id returned by IBKR in an order confirmation prompt. |
| `--confirmed` | No | Sends `confirmed: true`. If omitted, sends `confirmed: false`. |

Endpoint:

```text
POST iserver/reply/{reply_id}
```

Output:

The raw JSON response from IBKR.

Examples:

```bash
ibkrctl order reply --reply-id abc123 --confirmed
ibkrctl order reply --reply-id abc123
```

## `order cancel`

Cancels an existing order.

```bash
ibkrctl order cancel --account-id <ACCOUNT_ID> --order-id <ORDER_ID>
```

Options:

| Option | Required | Description |
| --- | --- | --- |
| `--account-id <ACCOUNT_ID>` | Yes | IBKR account id used in the endpoint path. |
| `--order-id <ORDER_ID>` | Yes | Order id to cancel. |

Endpoint:

```text
DELETE iserver/account/{account_id}/order/{order_id}
```

Output:

The raw JSON response from IBKR.

Example:

```bash
ibkrctl order cancel --account-id DU123456 --order-id 987654321 --pretty
```

## `order modify`

Modifies an existing order from an inline JSON argument and automatically answers selected IBKR confirmation prompts using built-in defaults plus optional answer-file and inline JSON layers.

```bash
ibkrctl order modify --account-id <ACCOUNT_ID> --order-id <ORDER_ID> --order-json <JSON> [--answers-file <PATH>] [--answers-json <JSON>] [--max-replies <N>]
```

Options:

| Option | Required | Default | Description |
| --- | --- | --- | --- |
| `--account-id <ACCOUNT_ID>` | Yes | None | IBKR account id used in the endpoint path. |
| `--order-id <ORDER_ID>` | Yes | None | Order id to modify. |
| `--order-json <JSON>` | Yes | None | Inline JSON containing one order object. |
| `--answers-file <PATH>` | No | Built-in defaults plus optional `IBKR_ORDERS_ANSWER_JSON` file | JSON map used to override or extend lower-priority answers for IBKR confirmation prompts. |
| `--answers-json <JSON>` | No | Built-in defaults plus optional answer files | Inline JSON map used to override or extend lower-priority answers for IBKR confirmation prompts. |
| `--max-replies <N>` | No | `20` | Maximum number of automatic confirmation reply loops before failing. |

Endpoint:

```text
POST iserver/account/{account_id}/order/{order_id}
```

The `--order-json` uses the same order object schema described in [Order JSON Fields](#order-json-fields), but it must contain one object rather than an array.

The answers file uses the same format and matching behavior as `order place`.

Output:

If no prompt remains, the final IBKR JSON response is printed. If IBKR returns a single-item array as the final order response, the CLI unwraps that single item.

Example:

```bash
ibkrctl order modify --account-id DU123456 --order-id 987654321 --order-json '{"conid":265598,"side":"BUY","quantity":1,"order_type":"LMT","price":185.5,"acct_id":"DU123456"}' --answers-file ./answers.json --pretty
ibkrctl order modify --account-id DU123456 --order-id 987654321 --order-json '{"conid":265598,"side":"BUY","quantity":1,"order_type":"LMT","price":185.5,"acct_id":"DU123456"}' --answers-json '{"o354":true}' --pretty
```

## `order status`

Fetches status for one order id.

```bash
ibkrctl order status --order-id <ORDER_ID>
```

Options:

| Option | Required | Description |
| --- | --- | --- |
| `--order-id <ORDER_ID>` | Yes | Order id to inspect. |

Endpoint:

```text
GET iserver/account/order/status/{order_id}
```

Output:

The raw JSON response from IBKR.

Example:

```bash
ibkrctl order status --order-id 987654321 --pretty
```

## `order fee-plan`

Computes the local Tiered/Fixed fee-plan decision for one or more limit orders. This command does not call IBKR and does not require OAuth configuration.

IBKR's Web API order documentation lists the order fields accepted by `/iserver/account/{accountId}/orders`, while IBKR's pricing-plan documentation describes Fixed/Tiered as an account pricing structure. The Web API documentation does not currently document a per-order field for changing the account commission plan. For that reason, this command reports the local decision without injecting an unsupported order field into the submitted order body.

Decision rule:

- If `quantity * price <= 10000`, the fee plan is `Tiered`.
- If `quantity * price > 10000`, the fee plan is `Fixed`.

```bash
ibkrctl order fee-plan --orders-json <JSON>
```

Options:

| Option | Required | Description |
| --- | --- | --- |
| `--orders-json <JSON>` | Yes | Inline JSON containing one order object or an array of order objects. |

Input requirements:

Each order must include numeric `quantity` and `price` fields. `price` is treated as the limit price for the notional calculation.

Output:

```json
{
  "orders": [
    {
      "index": 0,
      "notional": 10000.0,
      "threshold": 10000.0,
      "feePlan": "Tiered"
    }
  ]
}
```

Examples:

```bash
ibkrctl order fee-plan --orders-json '{"conid":265598,"side":"BUY","quantity":100,"order_type":"LMT","price":100,"acct_id":"DU123456"}' --pretty
```

## Order JSON Fields

Order input files are parsed into the CLI's `OrderRequest` model. Fields are serialized to IBKR using IBKR's expected camelCase names. Several snake_case aliases are accepted in input for convenience.

Any unrecognized JSON fields are preserved and forwarded to IBKR.

Supported modeled fields:

| Input field | IBKR field | Type | Notes |
| --- | --- | --- | --- |
| `conid` | `conid` | JSON value | Mutually exclusive with `conidex`. |
| `side` | `side` | String | Typical values are `BUY` or `SELL`. |
| `quantity` | `quantity` | JSON value | Mutually exclusive with `cashQty` and `fxQty`. |
| `order_type` or `orderType` | `orderType` | String | Example: `MKT`, `LMT`, `STP`. |
| `acct_id` or `acctId` | `acctId` | String | Account id inside the order body. |
| `price` | `price` | JSON value | Commonly used with limit orders. |
| `conidex` | `conidex` | String | Mutually exclusive with `conid`. |
| `manual_indicator` or `manualIndicator` | `manualIndicator` | Boolean | Forwarded to IBKR. |
| `ext_operator` or `extOperator` | `extOperator` | String | Forwarded to IBKR. |
| `sec_type` or `secType` | `secType` | String | Forwarded to IBKR. |
| `coid` or `cOID` | `cOID` | String | Client order id. |
| `parent_id` or `parentId` | `parentId` | String | Parent order id for related orders. |
| `listing_exchange` or `listingExchange` | `listingExchange` | String | Listing exchange. |
| `is_single_group` or `isSingleGroup` | `isSingleGroup` | Boolean | Forwarded to IBKR. |
| `outside_rth` or `outsideRTH` | `outsideRTH` | Boolean | Outside regular trading hours. |
| `aux_price` or `auxPrice` | `auxPrice` | JSON value | Auxiliary price for order types that require it. |
| `ticker` | `ticker` | String | Forwarded to IBKR. |
| `tif` | `tif` | String | Time in force, such as `DAY` or `GTC`. |
| `trailing_amt` or `trailingAmt` | `trailingAmt` | JSON value | Trailing amount. |
| `trailing_type` or `trailingType` | `trailingType` | String | Trailing type. |
| `customer_account` or `customerAccount` | `customerAccount` | String | Forwarded to IBKR. |
| `is_pro_customer` or `isProCustomer` | `isProCustomer` | Boolean | Forwarded to IBKR. |
| `referrer` | `referrer` | String | Forwarded to IBKR. |
| `cash_qty` or `cashQty` | `cashQty` | JSON value | Mutually exclusive with `quantity` and `fxQty`. |
| `fx_qty` or `fxQty` | `fxQty` | JSON value | Mutually exclusive with `quantity` and `cashQty`. |
| `use_adaptive` or `useAdaptive` | `useAdaptive` | Boolean | Forwarded to IBKR. |
| `is_ccy_conv` or `isCcyConv` | `isCcyConv` | Boolean | Forwarded to IBKR. |
| `allocation_method` or `allocationMethod` | `allocationMethod` | String | Forwarded to IBKR. |
| `manual_order_time` or `manualOrderTime` | `manualOrderTime` | JSON value | Forwarded to IBKR. |
| `deactivated` | `deactivated` | Boolean | Forwarded to IBKR. |
| `strategy` | `strategy` | String | IB Algo strategy id. Required when `strategyParameters` is present. |
| `strategy_parameters` or `strategyParameters` | `strategyParameters` | JSON value | IB Algo strategy parameters. Cannot be provided without `strategy`. |
| `is_close` or `isClose` | `isClose` | Boolean | Forwarded to IBKR. |

Validation performed before sending:

- `conid` and `conidex` cannot both be present.
- `quantity` and `cashQty` cannot both be present.
- `quantity` and `fxQty` cannot both be present.
- `cashQty` and `fxQty` cannot both be present.
- `strategyParameters` cannot be provided without `strategy`.
- `order place` and `order whatif` require at least one order in the parsed order list.

Example adaptive algo order:

```json
{
  "conid": 265598,
  "side": "BUY",
  "quantity": 100,
  "order_type": "LMT",
  "price": 185.5,
  "acct_id": "DU123456",
  "tif": "DAY",
  "strategy": "Adaptive",
  "strategy_parameters": {
    "adaptivePriority": "Normal"
  }
}
```

## Endpoint Mapping

| CLI command | IBKR endpoint |
| --- | --- |
| `auth-status` | `iserver/auth/status` |
| `init-session` | `iserver/auth/ssodh/init` |
| `tickle` | `tickle` |
| `fetch-history` | `iserver/marketdata/history` |
| `stock-conid` | `trsrv/stocks` |
| `accounts` | `portfolio/accounts` |
| `brokerage-accounts` | `iserver/accounts` |
| `account-pnl` | `iserver/account/pnl/partitioned` |
| `account-summary` | `iserver/account/{account_id}/summary` |
| `portfolio-summary` | `portfolio/{account_id}/summary` |
| `ledger` | `portfolio/{account_id}/ledger` |
| `positions` | `portfolio/{account_id}/positions/{page}` |
| `positions-live` | `portfolio2/{account_id}/positions` |
| `trades` | `iserver/account/trades/` |
| `live-orders` | `iserver/account/orders` |
| `quick-vwap-order` | `trsrv/stocks`, `iserver/account/{account_id}/orders`, and confirmation replies when required |
| `order algos` | `iserver/contract/{conid}/algos` |
| `order place` | `iserver/account/{account_id}/orders` |
| `order whatif` | `iserver/account/{account_id}/orders/whatif` |
| `order reply` | `iserver/reply/{reply_id}` |
| `order cancel` | `iserver/account/{account_id}/order/{order_id}` |
| `order modify` | `iserver/account/{account_id}/order/{order_id}` |
| `order status` | `iserver/account/order/status/{order_id}` |

## Practical Workflows

### Initial OAuth Setup

```bash
ibkrctl oauth generate-materials --out-dir /secure/ibkr/oauth
```

Send these files to IBKR:

- `/secure/ibkr/oauth/public_signature.pem`
- `/secure/ibkr/oauth/public_encryption.pem`
- `/secure/ibkr/oauth/dhparam.pem`

Keep these files private:

- `/secure/ibkr/oauth/private_signature.pem`
- `/secure/ibkr/oauth/private_encryption.pem`
- `/secure/ibkr/oauth/private_signature.pk8`
- `/secure/ibkr/oauth/private_encryption.pk8`

Set the runtime environment:

```text
IBKR_CONSUMER_KEY=<from IBKR>
IBKR_ACCESS_TOKEN=<from IBKR>
IBKR_ACCESS_TOKEN_SECRET=<from IBKR>
IBKR_SIGNATURE_KEY_PATH=/secure/ibkr/oauth/private_signature.pem
IBKR_ENCRYPTION_KEY_PATH=/secure/ibkr/oauth/private_encryption.pem
IBKR_DH_PARAM_PATH=/secure/ibkr/oauth/dhparam.pem
```

Verify configuration shape:

```bash
ibkrctl env
```

Initialize brokerage session:

```bash
ibkrctl init-session
```

### Initialize Account Contexts

IServer trading endpoints and portfolio endpoints have different account-context preflights.

For IServer account summary and P&L:

```bash
ibkrctl brokerage-accounts
ibkrctl account-summary --account-id DU123456 --pretty
ibkrctl account-pnl --pretty
```

For portfolio summary, ledger, and positions:

```bash
ibkrctl accounts
ibkrctl portfolio-summary --account-id DU123456 --pretty
ibkrctl ledger --account-id DU123456 --pretty
ibkrctl positions --account-id DU123456 --pretty
ibkrctl positions-live --account-id DU123456 --pretty
```

For scoped trade executions:

```bash
ibkrctl brokerage-accounts
ibkrctl trades
sleep 5
ibkrctl trades --account-id DU123456 --days 7 --pretty
```

### Resolve Symbol Then Fetch History

```bash
ibkrctl stock-conid --symbol AAPL --exchange NASDAQ --output /tmp/aapl-conid.json
ibkrctl fetch-history --conid 265598 --period 1d --bar 1min --output /tmp/aapl-history.json
```

### Place A Non-Interactive Order

`order.json`:

```json
{
  "conid": 265598,
  "side": "BUY",
  "quantity": 1,
  "order_type": "MKT",
  "acct_id": "DU123456",
  "coid": "airflow-20260505-0001"
}
```

`answers.json`:

```json
{
  "price exceeds the Percentage constraint": true,
  "o354": true
}
```

Command:

```bash
ibkrctl init-session
ibkrctl order place --account-id DU123456 --orders-json '{"conid":265598,"side":"BUY","quantity":1,"order_type":"MKT","acct_id":"DU123456"}' --answers-file ./answers.json --output /tmp/place-order-response.json
```

### Preview An Order Without Placing It

```bash
ibkrctl order whatif --account-id DU123456 --orders-json '{"conid":265598,"side":"BUY","quantity":1,"order_type":"LMT","price":185.5,"acct_id":"DU123456"}' --pretty
```

### Inspect And Cancel Live Orders

```bash
ibkrctl live-orders --account-id DU123456 --pretty
ibkrctl order status --order-id 987654321 --pretty
ibkrctl order cancel --account-id DU123456 --order-id 987654321
```

## Notes For Schedulers

For Airflow or similar systems:

- Treat exit code `0` as success.
- Treat any nonzero exit code as task failure.
- Prefer `--output` when a downstream task expects a file artifact.
- Keep OAuth private key files and dotenv files outside the repository.
- Use `IBKR_LST_CACHE_MODE=redis` when many short-lived tasks need to share OAuth Live Session Tokens.
- Run `init-session` before protected account, market-data, and order workflows.
- Re-run `init-session` if IBKR reports that bridge state is missing or stale.
- Keep `brokerage-accounts`, the initial `trades` request, and any required wait as explicit scheduler tasks; the CLI does not hide these steps.
- Do not run `quick-vwap-order` as an unattended scheduler task; it requires a controlling terminal and operator confirmations.
