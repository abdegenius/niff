# Requirements Document

## Introduction

This feature introduces a dedicated, injectable `SorobanRpcService` for the niffyInsure NestJS backend. The service encapsulates all Soroban RPC / Stellar SDK interactions behind a single, well-defined boundary. It handles configurable timeouts, exponential-backoff retries for transient failures, and maps low-level Stellar SDK errors to structured NestJS HTTP exceptions so callers never need to reason about raw RPC error codes. Network selection (Futurenet / Testnet / Mainnet) is driven entirely by environment variables. The service emits Prometheus-compatible metrics and structured logs so operations teams can distinguish client faults, server faults, and provider degradation. RPC API keys are kept server-side only and never reach browser bundles.

> **Note on current state**: The backend currently uses Express. Migrating to NestJS (or wrapping the service in a NestJS module) is a prerequisite for the injectable pattern described here. Requirements are written against the target NestJS architecture.

---

## Glossary

- **SorobanRpcService**: The injectable NestJS service that is the subject of this document.
- **RPC_Provider**: The remote Soroban JSON-RPC endpoint (public horizon-soroban, Quickstart, or a dedicated provider such as Ankr/Blockdaemon).
- **Stellar_SDK**: The `@stellar/stellar-sdk` npm package used to construct and submit transactions.
- **Simulation**: A `simulateTransaction` RPC call that dry-runs a contract invocation and returns resource footprint and result without committing to the ledger.
- **Submission**: A `sendTransaction` RPC call that broadcasts a signed transaction to the network.
- **Network_Config**: The tuple of `{ rpcUrl, networkPassphrase }` that identifies a Stellar network.
- **Retry_Budget**: The maximum number of retry attempts allowed for a single RPC call before the service gives up and raises an error.
- **Backoff_Policy**: The exponential-backoff algorithm (base delay, multiplier, jitter, max delay) applied between retries.
- **Error_Classifier**: The internal component that maps raw Stellar/RPC error codes to NestJS HTTP exceptions or structured API error codes.
- **Metrics_Emitter**: The component responsible for recording Prometheus counters and histograms (or equivalent structured log entries).
- **Finality_Window**: The number of ledgers after which a submitted transaction is considered irreversibly confirmed or failed on Stellar (~5–7 seconds per ledger; practical finality within 1–2 ledgers).

---

## Requirements

### Requirement 1: NestJS Module and Injectable Service

**User Story:** As a backend developer, I want a NestJS module that exports `SorobanRpcService`, so that any feature module (quote, policy, claim) can inject it without managing RPC connections directly.

#### Acceptance Criteria

1. THE SorobanRpcService SHALL be decorated with `@Injectable()` and registered in a `SorobanRpcModule` that can be imported by other NestJS feature modules.
2. THE SorobanRpcModule SHALL export `SorobanRpcService` so that importing modules can inject it via constructor injection.
3. WHEN `SorobanRpcModule` is imported more than once in the application module graph, THE SorobanRpcModule SHALL behave as a global singleton (using `@Global()` or `isGlobal: true`) to avoid duplicate RPC connections.

---

### Requirement 2: Network Configuration via Environment Variables

**User Story:** As a DevOps engineer, I want to select the Stellar network and RPC endpoint through environment variables, so that the same build artifact can target Futurenet, Testnet, or Mainnet without code changes.

#### Acceptance Criteria

1. THE SorobanRpcService SHALL read `STELLAR_NETWORK` (accepted values: `futurenet`, `testnet`, `mainnet`) from the process environment at startup.
2. WHEN `STELLAR_NETWORK` is `futurenet`, THE SorobanRpcService SHALL use the RPC URL from `STELLAR_RPC_URL` and the network passphrase from `STELLAR_NETWORK_PASSPHRASE`, falling back to the public Futurenet defaults if those variables are absent.
3. WHEN `STELLAR_NETWORK` is `testnet`, THE SorobanRpcService SHALL use the RPC URL from `STELLAR_RPC_URL` and the network passphrase from `STELLAR_NETWORK_PASSPHRASE`, falling back to the public Testnet defaults if those variables are absent.
4. WHEN `STELLAR_NETWORK` is `mainnet`, THE SorobanRpcService SHALL require both `STELLAR_RPC_URL` and `STELLAR_NETWORK_PASSPHRASE` to be explicitly set and SHALL throw a configuration error at startup if either is absent.
5. IF `STELLAR_NETWORK` is set to a value other than `futurenet`, `testnet`, or `mainnet`, THEN THE SorobanRpcService SHALL throw a configuration error at application startup with a message identifying the invalid value.
6. THE SorobanRpcService SHALL accept an optional `STELLAR_RPC_API_KEY` environment variable and, WHEN present, SHALL attach it as a server-side HTTP header on every RPC request.
7. THE SorobanRpcService SHALL never include `STELLAR_RPC_API_KEY` in any response body, log line, or client-facing error message.

---

### Requirement 3: Configurable Timeouts

**User Story:** As a backend developer, I want every RPC call to have a bounded timeout, so that a slow or unresponsive provider does not block request processing indefinitely.

#### Acceptance Criteria

1. THE SorobanRpcService SHALL apply a per-call timeout to every outbound RPC request, configurable via the `STELLAR_RPC_TIMEOUT_MS` environment variable (default: 10 000 ms).
2. WHEN an RPC call does not complete within the configured timeout, THE SorobanRpcService SHALL cancel the in-flight request and treat the outcome as a transient failure eligible for retry.
3. THE SorobanRpcService SHALL enforce a maximum total wall-clock budget per logical operation (simulation or submission) via `STELLAR_RPC_TOTAL_TIMEOUT_MS` (default: 30 000 ms), after which no further retries SHALL be attempted regardless of remaining retry count.

---

### Requirement 4: Exponential Backoff and Retry

**User Story:** As a backend developer, I want transient RPC failures to be retried automatically with exponential backoff, so that brief provider hiccups do not surface as errors to callers.

#### Acceptance Criteria

1. THE SorobanRpcService SHALL retry RPC calls that fail with transient errors (network timeout, HTTP 429, HTTP 503, Soroban `tryAgainLater` status) up to a maximum of `STELLAR_RPC_MAX_RETRIES` attempts (default: 3).
2. WHEN retrying, THE SorobanRpcService SHALL wait for a delay calculated as `min(baseDelay * 2^attempt + jitter, maxDelay)` where `baseDelay` defaults to 200 ms, `maxDelay` defaults to 5 000 ms, and `jitter` is a random value in `[0, baseDelay]`.
3. THE SorobanRpcService SHALL NOT retry RPC calls that fail with non-transient errors (HTTP 400, invalid XDR, contract execution errors, insufficient funds).
4. WHEN the Retry_Budget is exhausted without a successful response, THE SorobanRpcService SHALL stop retrying and propagate a mapped HTTP exception to the caller.
5. THE SorobanRpcService SHALL log each retry attempt at `warn` level, including the attempt number, error code, and delay applied.

---

### Requirement 5: Error Mapping to NestJS HTTP Exceptions

**User Story:** As a backend developer, I want low-level Stellar errors mapped to standard NestJS HTTP exceptions, so that callers receive actionable, structured error responses without leaking infrastructure details.

#### Acceptance Criteria

1. WHEN an RPC call fails due to provider unavailability or exhausted retries on transient errors, THE Error_Classifier SHALL throw `HttpException` with status 502 (Bad Gateway) and a structured body containing `{ code, message }`.
2. WHEN an RPC call fails because the provider is rate-limiting the service, THE Error_Classifier SHALL throw `HttpException` with status 503 (Service Unavailable) and include a `Retry-After` header when the provider supplies one.
3. WHEN an RPC call fails due to a client mistake (invalid contract ID, malformed XDR, insufficient balance, bad sequence number), THE Error_Classifier SHALL throw `HttpException` with status 400 (Bad Request) and a structured body containing `{ code, message, field? }` where `code` is a stable, documented API error code.
4. THE Error_Classifier SHALL map the following Soroban result codes to stable API error codes:

   | Soroban / Stellar condition | HTTP status | Stable `code` |
   |---|---|---|
   | `tryAgainLater` (after retries) | 503 | `RPC_UNAVAILABLE` |
   | Provider timeout (after retries) | 502 | `RPC_TIMEOUT` |
   | `txBAD_SEQ` | 400 | `TX_BAD_SEQUENCE` |
   | `txINSUFFICIENT_BALANCE` | 400 | `TX_INSUFFICIENT_BALANCE` |
   | `txINSUFFICIENT_FEE` | 400 | `TX_INSUFFICIENT_FEE` |
   | Invalid contract ID / XDR | 400 | `TX_INVALID_INPUT` |
   | Contract execution failure | 400 | `CONTRACT_EXECUTION_FAILED` |
   | Unknown / unclassified | 502 | `RPC_UNKNOWN_ERROR` |

5. THE Error_Classifier SHALL never include raw RPC response bodies, stack traces, or internal hostnames in client-facing error responses.
6. THE Error_Classifier SHALL log the full raw error at `error` level internally before throwing the mapped exception, so that operators can diagnose issues without exposing details to clients.

---

### Requirement 6: Simulation Helper

**User Story:** As a backend developer building the quote and tx-build endpoints, I want a centralized simulation helper, so that I do not duplicate `simulateTransaction` boilerplate across feature modules.

#### Acceptance Criteria

1. THE SorobanRpcService SHALL expose a `simulate(transaction: Transaction): Promise<SimulateTransactionResponse>` method that wraps `Stellar_SDK.rpc.Server.simulateTransaction`.
2. WHEN simulation succeeds, THE SorobanRpcService SHALL return the raw `SimulateTransactionResponse` to the caller.
3. WHEN simulation returns a `SimulateTransactionResponse` with `error` set, THE SorobanRpcService SHALL classify and throw the appropriate HTTP exception per Requirement 5.
4. THE SorobanRpcService SHALL apply timeout and retry logic (Requirements 3 and 4) to simulation calls.

---

### Requirement 7: Submission Helper

**User Story:** As a backend developer building the tx-build endpoint, I want a centralized submission helper, so that I do not duplicate `sendTransaction` and polling boilerplate across feature modules.

#### Acceptance Criteria

1. THE SorobanRpcService SHALL expose a `submit(signedXdr: string): Promise<GetTransactionResponse>` method that sends a signed transaction and polls for its final status.
2. WHEN `sendTransaction` returns status `PENDING`, THE SorobanRpcService SHALL poll `getTransaction` at configurable intervals (`STELLAR_RPC_POLL_INTERVAL_MS`, default: 2 000 ms) until the transaction reaches a terminal status (`SUCCESS`, `FAILED`, `NOT_FOUND`) or the total timeout is exceeded.
3. WHEN the transaction reaches `SUCCESS`, THE SorobanRpcService SHALL return the `GetTransactionResponse` to the caller.
4. WHEN the transaction reaches `FAILED` or `NOT_FOUND` after the polling window, THE SorobanRpcService SHALL throw the appropriate mapped HTTP exception per Requirement 5.
5. THE SorobanRpcService SHALL apply the total wall-clock budget (Requirement 3.3) across the combined send + poll cycle.

---

### Requirement 8: Metrics and Observability

**User Story:** As an operations engineer, I want Prometheus-compatible metrics and structured logs from the RPC service, so that I can detect provider degradation and distinguish client, server, and provider faults.

#### Acceptance Criteria

1. THE Metrics_Emitter SHALL increment a counter `soroban_rpc_requests_total` labelled with `{ method, network, status }` on every RPC call completion, where `status` is one of `success`, `client_error`, `server_error`, `provider_error`.
2. THE Metrics_Emitter SHALL record a histogram `soroban_rpc_duration_seconds` labelled with `{ method, network }` measuring the wall-clock time of each RPC call (excluding retry delays).
3. THE Metrics_Emitter SHALL increment a counter `soroban_rpc_retries_total` labelled with `{ method, network }` each time a retry is attempted.
4. WHEN Prometheus integration is not available, THE Metrics_Emitter SHALL emit equivalent structured JSON log entries at `info` level containing the same label/value pairs, so that log-based alerting remains possible.
5. THE Metrics_Emitter SHALL label every structured log entry with `{ service: "SorobanRpcService", network, method }` to enable log-based filtering.

---

### Requirement 9: Security — API Key Isolation

**User Story:** As a security engineer, I want RPC API keys to remain exclusively server-side, so that they are never exposed in browser bundles, client responses, or logs.

#### Acceptance Criteria

1. THE SorobanRpcService SHALL load `STELLAR_RPC_API_KEY` only from the server-side environment and SHALL never serialize it into any HTTP response, WebSocket message, or client-accessible endpoint.
2. THE SorobanRpcService SHALL redact `STELLAR_RPC_API_KEY` from all log output, replacing it with `[REDACTED]` if it would otherwise appear.
3. WHERE the NestJS application uses a configuration module (e.g., `@nestjs/config`), THE SorobanRpcModule SHALL declare `STELLAR_RPC_API_KEY` as a server-only variable and SHALL NOT expose it via any public configuration endpoint.

---

### Requirement 10: Stellar Finality Documentation

**User Story:** As a frontend developer, I want documented finality assumptions for Stellar transactions, so that I can display accurate UX messaging about confirmation times.

#### Acceptance Criteria

1. THE SorobanRpcService SHALL include inline code documentation (JSDoc) stating that Stellar ledgers close approximately every 5–7 seconds and that practical transaction finality is achieved within 1–2 ledger closes (~10–14 seconds) under normal network conditions.
2. THE SorobanRpcService SHALL document that `NOT_FOUND` status after the polling window does not guarantee the transaction was rejected — it may still be included in a future ledger — and callers SHOULD re-query before treating it as a definitive failure.
3. THE SorobanRpcService SHALL document the recommended UX messaging pattern: show "confirming…" until `SUCCESS` is received, and show "check back shortly" rather than "failed" when `NOT_FOUND` is returned after the polling window.

---

### Requirement 11: Rate Limit and Provider Documentation

**User Story:** As a backend developer, I want documented rate limits for public RPC endpoints and guidance on when dedicated providers are required, so that I can plan capacity and avoid unexpected throttling in production.

#### Acceptance Criteria

1. THE SorobanRpcService source file SHALL include a documentation block listing the known rate limits of public Soroban RPC endpoints (Horizon-Soroban Testnet: ~100 req/min per IP; Futurenet: lower, unstable; Mainnet public: not recommended for production).
2. THE SorobanRpcService documentation SHALL state that dedicated providers (Ankr, Blockdaemon, or self-hosted Quickstart) are mandatory for Mainnet production workloads exceeding 50 req/min.
3. THE SorobanRpcService documentation SHALL describe the Quickstart local node setup command for integration testing so that developers can run tests without hitting public rate limits.
