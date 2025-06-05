# Undeadlock

Custom synchronization primitives for Rust with built-in deadlock detection and prevention.

## Overview

Undeadlock provides drop-in replacements for standard Rust synchronization primitives that automatically detect and prevent deadlocks during development. In release builds, they compile down to efficient wrappers with minimal overhead.

## Features

- **Deadlock Detection**: Automatically detects potential deadlocks in debug builds
- **Lock Ordering**: Enforces consistent lock ordering to prevent deadlocks
- **Debug Visibility**: Enhanced debugging output to trace lock acquisition
- **Minimal Overhead**: Near-zero overhead in release builds
- **Drop-in Replacement**: Compatible with standard library APIs

## Primitives

### CustomRwLock
A RwLock implementation with read/write deadlock detection.

```rust
use undeadlock::CustomRwLock;

let lock = CustomRwLock::new(vec![1, 2, 3]);

// Multiple readers
{
    let r1 = lock.read();
    let r2 = lock.read();
    println!("Values: {:?} and {:?}", *r1, *r2);
}

// Exclusive writer
{
    let mut w = lock.write();
    w.push(4);
}
```

### CustomDashMap
A concurrent HashMap with deadlock-aware locking.

```rust
use undeadlock::CustomDashMap;

let map = CustomDashMap::new();

// Concurrent access
map.insert("key", "value");
if let Some(v) = map.get(&"key") {
    println!("Found: {}", *v);
}

// Entry API
map.entry("counter")
    .and_modify(|v| *v += 1)
    .or_insert(0);
```

## Examples

Run the examples with:

```bash
# Basic RwLock usage
cargo run --example basic_rwlock --features examples

# DashMap concurrent operations  
cargo run --example dashmap_usage --features examples

# Deadlock prevention examples
cargo run --example mutex_ordering --features examples
```

## How It Works

### Debug Mode
In debug builds, Undeadlock:
1. Tracks lock acquisition order per thread
2. Detects circular dependencies between locks
3. Assigns unique IDs to each lock instance
4. Panics with detailed diagnostics when deadlock is detected

### Release Mode  
In release builds:
- Deadlock detection is compiled out
- Minimal wrapper overhead (usually just a pointer indirection)
- Operations compile down to near-native performance

## Example: Concurrent Operations

Here's how Undeadlock helps detect potential deadlocks:

```rust
use undeadlock::{CustomRwLock, CustomDashMap};

async fn concurrent_updates(shared_data: &CustomDashMap<String, i32>) {
    // Multiple threads can safely update different keys
    let mut handle1 = shared_data.get_mut(&"key1".to_string());
    let mut handle2 = shared_data.get_mut(&"key2".to_string());
    
    // In debug mode, holding locks too long triggers warnings
    *handle1 += 1;
    *handle2 += 1;
}

// Thread-safe read/write access
async fn read_write_example(data: &CustomRwLock<Vec<i32>>) {
    // Multiple concurrent readers
    let reader1 = data.read().await;
    let reader2 = data.read().await;
    
    drop(reader1);
    drop(reader2);
    
    // Exclusive writer
    let mut writer = data.write().await;
    writer.push(42);
}
```

## Performance

Benchmarks show minimal overhead:
- **Debug mode**: ~15-20% overhead for deadlock detection
- **Release mode**: <1% overhead compared to standard library

See benchmarks in `benches/` for detailed performance analysis.

## Integration

Undeadlock is designed to integrate seamlessly:

```rust
// Before
use tokio::sync::RwLock;
use dashmap::DashMap;

// After  
use undeadlock::{CustomRwLock as RwLock, CustomDashMap as DashMap};
```

## Limitations

- Deadlock detection only works within a single process
- Cannot detect deadlocks involving external resources
- Async code requires special handling (see async examples)
- Some edge cases in complex lock hierarchies may not be detected