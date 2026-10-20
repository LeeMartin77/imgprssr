# imgprssr — Productionisation Plan

> **Status:** Draft — not committed to remote
> **Branch:** `prod/imgprssr`

---

## 1. Out-of-Date Libraries

| Dependency | Current | Latest | Notes |
|---|---|---|---|
| **hyper** | `0.14` | `1.12.0` | **Critical** — 0.14 reached EOL Dec 2024. 1.x has breaking API changes (Server/Client removed, new builder pattern). Major migration required. |
| **hyper-tls** | `0.5.0` | `0.6.0` | 0.5.0 is the last 0.x release; 0.6.0 requires hyper 1.x. Must upgrade together with hyper. |
| **image** | `0.24.5` | `0.25.10` | Minor bump. Should be mostly compatible but check API changes. |
| **config** | `0.13.1` | `0.15.27` | MSRV bumped to 1.88. Should work with 1.98 target. |
| **signal-hook** | `0.3.14` | `0.4.1` | signal-hook-tokio also needs upgrade from `0.3.1` → `0.4.0`. |
| **tokio** | `1` | `1.54.0` | Latest patches; `cargo update` handles this. |
| **futures** | `0.3.25` | `0.3.31` | Minor bump, likely safe. |

**Priority:** Hyper 0.14 → 1.x is the biggest lift. It's a full API rewrite — the `Server`, `Client`, and request/response builders all changed. This should be done first, as hyper 0.14 is no longer maintained.

---

## 2. Security Issues

### 🔴 Critical — Path Traversal (Local File Source)

**File:** `src/source.rs:36-38`

```rust
let full_path = format!("{}{}", img_source, target_path);
let path = Path::new(&full_path);
```

`target_path` comes directly from the request URI with no sanitisation. A request to `/../../etc/passwd` will escape the image directory. **No validation of `..` or absolute paths.**

### 🟡 Medium — SSRF (HTTP/HTTPS Source)

**File:** `src/source.rs:26-33`

The HTTP/HTTPS source concatenates user-supplied `target_path` to the configured base URL without validation. An attacker could request `http://localhost:2379` (Docker API), `http://169.254.169.254` (cloud metadata), etc. No allowlist or denylist of internal/private IPs.

### 🟡 Medium — Unhandled Panics

Across `main.rs`, `source.rs`, and `process.rs`, there are **26 `.unwrap()` calls**. Any unexpected input, missing file, or parse failure will panic the entire server. Production services should use proper error propagation and return HTTP error responses.

### 🟢 Low — Dockerfile Issues

- Runs as root (no `USER` directive)
- Uses `alpine:3.24.2` — **should pin exact versions** rather than using floating tags. Pinned images make diagnosing regressions far easier.
- No `.dockerignore` coverage for `target/` or `Cargo.lock` exclusions

### 🟢 Low — No Content Security / Rate Limiting / CORS

The service has no request rate limiting, no Content-Security-Policy headers, and no CORS configuration. These are **intentional design choices** (the service is lean by design) but should be documented as such.

---

## 3. Test Coverage Gaps

### Missing Unit Tests

| Module | Coverage | Gap |
|---|---|---|
| `parameters.rs` | ✅ Good | — |
| `process.rs` | ⚠️ Partial | No oversized image tests, no filter type tests, no combined param tests |
| `source.rs` | ⚠️ Fragile | Tests hit the real internet (flaky); no path traversal tests; no HTTP handler tests |
| `appconfig.rs` | ❌ None | No config parsing, validation, or default tests |
| `main.rs` | ❌ None | No handler tests, no config generation tests |

### Missing E2E / Integration Tests

- No tests for **error responses** (400 Bad Request, 404 Not Found)
- No tests for **invalid parameters** (malformed query strings)
- No tests for **greyscale** feature
- No tests for **oversized image handling** (Clamp mode)
- No tests for **different image formats** (WebP, GIF, etc.)
- No tests for **edge cases** (zero dimensions, very large dimensions)
- No tests for **path traversal rejection**
- No concurrent / load tests

### Missing Test Infrastructure

- No **security fuzzing** (e.g. honggfuzz or cargo-fuzz for parameter parsing)
- No **benchmarks** (no `criterion` setup)
- No **snapshot management** for e2e image tests (baseline images not checked into version control)
- CI has no **security scanning** (no `cargo audit`, no `trivy` for container images)
- CI has no **dependency vulnerability scanning**
- CI **does not run on PRs for test file changes** — the `paths` filter in `.github/workflows/test.yml` excludes `e2e_tests/` and does not include test source files, so adding tests silently skips CI

---

## Tasks

> **Approach:** TDD — write a failing (red) test first, make the minimal change to get it green, then move on. This ensures every change is covered by a test and prevents regression.

- [x] **1. Fix CI `paths` filter** so tests run on PRs for test file changes and e2e test changes
- [x] **2. Add tests for test infrastructure** (verify CI discovers and runs tests, e2e suite bootstraps correctly)
- [x] **3. Add unit tests for `appconfig.rs`** and `main.rs` handler (TDD)
- [x] **4. Add unit tests for `process.rs`** (oversized handling, filters, combined params) (TDD)
- [x] **5. Write path traversal test** (TDD — red)
- [x] **6. Fix path traversal vulnerability** (TDD — green)
- [x] **7. Write SSRF test** (TDD — red)
- [x] **8. Fix SSRF vulnerability** on HTTP/HTTPS source (TDD — green)
- [ ] **9. Add `cargo audit` to CI** (security baseline)
- [ ] **10. Add `cargo fuzz` / property-based tests** for parameter parsing
- [ ] **11. Add e2e tests** for error cases, greyscale, formats, edge cases — serves as regression guard before upgrades
- [ ] **12. Upgrade `image`, `config`, `signal-hook` families** (low risk)
- [ ] **13. Migrate hyper 0.14 → 1.x** (high effort, high impact)
- [ ] **14. Dockerfile hardening** (non-root user, pin exact versions, `.dockerignore`)
- [ ] **15. Document intentional design choices** (no CORS, no rate limiting — by design)
