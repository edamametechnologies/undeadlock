# Undeadlock

Light-weight diagnostic wrappers around `tokio::sync::RwLock` and `dashmap::DashMap` that help you discover lock contention and potential dead-locks while developing.

In **debug** builds the wrappers gather rich runtime information (back-traces, timestamps, thread-ids, locked keys, …) and emit `tracing` warnings or errors whenever:

* acquiring a read lock takes longer than **10 s**;
* acquiring a write lock takes longer than **15 s**;
* a `CustomDashMap` key is still held after **10 s** by another writer;
* an instrumented map/lock operation itself runs for more than **1 s**.

The code never panics or aborts – it merely logs – so program semantics are unchanged.  You get actionable diagnostics without risking new failures.

In **release** builds all of the additional bookkeeping is compiled out and the types collapse to the plain primitives:

```rust
// release mode
pub type CustomRwLock<T> = tokio::sync::RwLock<T>;

pub struct CustomDashMap<K, V>(dashmap::DashMap<K, V>);
```

The overhead is therefore virtually zero.

---

## Getting started

Add the crate and (optionally) enable the `tokio-console` feature to propagate caller-location information into console spans:

```toml
[dependencies]
undeadlock = { version = "0.1", features = ["tokio-console"] }
```

Replace your existing locks:

```rust
use tokio::sync::RwLock;        // ❌
use dashmap::DashMap;           // ❌

use undeadlock::{               // ✅
    CustomRwLock as RwLock,
    CustomDashMap as DashMap,
};
```

Everything else keeps compiling unchanged.

---

## Quick example

```rust
use undeadlock::{CustomDashMap, CustomRwLock};
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // emit logs to stdout
    tracing_subscriber::fmt::init();

    // keyed concurrent structure
    let users = Arc::new(CustomDashMap::new("users"));
    users.insert("alice", 0);

    // shared counter
    let counter = Arc::new(CustomRwLock::new(0u64));

    // spawn some tasks that stress the locks
    // your application code here
}
```

Run one of the shipped examples in **debug** mode to see the diagnostics:

```bash
cargo run --example basic_rwlock
cargo run --example dashmap_usage
cargo run --example mutex_ordering
```

Switch to **release** to benchmark with almost no overhead:

```bash
cargo run --release --example basic_rwlock
```

---

## Customising thresholds

`CustomDashMap` offers an extra constructor allowing you to choose the timeout (in seconds) that is considered "too long" for a writer:

```rust
let map = CustomDashMap::new_with_timeout("my_map", 30); // 30-second threshold
```

For `CustomRwLock` thresholds, modify the constants in `src/debug.rs` and recompile.

---

## What this crate **does not** do

* It does **not** enforce a global lock-ordering discipline.
* It does **not** build a dependency graph to prove the absence of dead-locks.
* It does **not** terminate your program – it only logs.

The purpose is to surface the most common issues (holding a lock for too long, double-locking the same key, etc.) early during development without affecting production performance.

---

## License

Licensed under the Apache License, Version 2.0.  See the `LICENSE` file for details.
