//! Example: Deadlock Prevention with CustomRwLock and CustomDashMap
//!
//! This example demonstrates how the custom synchronization primitives
//! help detect and prevent deadlocks in debug mode.

use std::sync::Arc;
use std::thread;
use std::time::Duration;
use undeadlock::{CustomDashMap, CustomRwLock};

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_target(false)
        .with_level(true)
        .init();

    println!("=== Undeadlock Deadlock Prevention Example ===\n");

    // Example 1: Demonstrate read/write lock behavior
    rwlock_behavior_demo();
    println!();

    // Example 2: Concurrent map operations
    concurrent_map_demo();
    println!();

    // Example 3: Complex nested locking scenario
    nested_locking_demo();
    println!();

    // Example 4: Performance with debug checks
    performance_demo();
}

fn rwlock_behavior_demo() {
    println!("Example 1: RwLock Behavior Demonstration\n");

    let data = Arc::new(CustomRwLock::new(vec![1, 2, 3, 4, 5]));
    let mut handles = vec![];

    // Spawn multiple readers
    for i in 0..3 {
        let data_clone = Arc::clone(&data);
        handles.push(thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                println!("Reader {} waiting for read lock...", i);
                let guard = data_clone.read().await;
                println!("Reader {} acquired read lock: {:?}", i, *guard);
                tokio::time::sleep(Duration::from_millis(100)).await;
                println!("Reader {} releasing read lock", i);
            });
        }));
    }

    // Give readers time to start
    thread::sleep(Duration::from_millis(50));

    // Spawn a writer
    let data_clone = Arc::clone(&data);
    let writer_handle = thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            println!("Writer waiting for write lock...");
            let mut guard = data_clone.write().await;
            println!("Writer acquired write lock");
            guard.push(6);
            println!("Writer modified data: {:?}", *guard);
            tokio::time::sleep(Duration::from_millis(50)).await;
            println!("Writer releasing write lock");
        });
    });

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    writer_handle.join().unwrap();
}

fn concurrent_map_demo() {
    println!("Example 2: Concurrent Map Operations\n");

    let map = Arc::new(CustomDashMap::new("shared_map"));

    // Initialize some data
    for i in 0..10 {
        map.insert(i, format!("value_{}", i));
    }

    let mut handles = vec![];

    // Thread 1: Updates even keys
    let map1 = Arc::clone(&map);
    handles.push(thread::spawn(move || {
        for i in (0..10).step_by(2) {
            if let Some(mut value) = map1.get_mut(&i) {
                println!("Thread 1: Updating key {}", i);
                *value = format!("updated_by_thread1_{}", i);
                thread::sleep(Duration::from_millis(50));
            }
        }
    }));

    // Thread 2: Updates odd keys
    let map2 = Arc::clone(&map);
    handles.push(thread::spawn(move || {
        for i in (1..10).step_by(2) {
            if let Some(mut value) = map2.get_mut(&i) {
                println!("Thread 2: Updating key {}", i);
                *value = format!("updated_by_thread2_{}", i);
                thread::sleep(Duration::from_millis(50));
            }
        }
    }));

    // Thread 3: Reads all keys
    let map3 = Arc::clone(&map);
    handles.push(thread::spawn(move || {
        thread::sleep(Duration::from_millis(100)); // Let updates start
        for i in 0..10 {
            if let Some(value) = map3.get(&i) {
                println!("Thread 3: Read key {} = {}", i, *value);
            }
        }
    }));

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    println!("\nFinal map contents:");
    for entry in map.iter() {
        let (key, value) = entry.pair();
        println!("  {}: {}", key, value);
    }
}

fn nested_locking_demo() {
    println!("Example 3: Nested Locking Scenario\n");

    #[derive(Debug)]
    struct Account {
        _id: u32,
        _balance: f64,
        lock: CustomRwLock<()>,
    }

    let accounts = CustomDashMap::new("accounts");

    // Create accounts
    for i in 0..5 {
        let account = Arc::new(Account {
            _id: i,
            _balance: 1000.0 * (i + 1) as f64,
            lock: CustomRwLock::new(()),
        });
        accounts.insert(i, account);
    }

    let accounts = Arc::new(accounts);
    let mut handles = vec![];

    // Simulate concurrent transfers
    for thread_id in 0..3 {
        let accounts_clone = Arc::clone(&accounts);
        handles.push(thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                for _ in 0..2 {
                    let from_id = thread_id % 5;
                    let to_id = (thread_id + 1) % 5;

                    println!(
                        "Thread {} attempting transfer from {} to {}",
                        thread_id, from_id, to_id
                    );

                    // Get accounts from map
                    if let (Some(from_ref), Some(to_ref)) =
                        (accounts_clone.get(&from_id), accounts_clone.get(&to_id))
                    {
                        let from_acc = Arc::clone(from_ref.value());
                        let to_acc = Arc::clone(to_ref.value());

                        // Drop map references before locking accounts
                        drop(from_ref);
                        drop(to_ref);

                        // Lock both accounts
                        let _from_lock = from_acc.lock.write().await;
                        let _to_lock = to_acc.lock.write().await;

                        // Simulate transfer (in real code, would modify balances)
                        println!(
                            "Thread {} completed transfer from {} to {}",
                            thread_id, from_id, to_id
                        );

                        tokio::time::sleep(Duration::from_millis(50)).await;
                    }
                }
            });
        }));
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    println!("\nAll transfers completed without deadlock!");
}

fn performance_demo() {
    println!("Example 4: Performance with Debug Checks\n");

    let iterations = 1000;
    let data = Arc::new(CustomRwLock::new(0u64));

    // Test write performance
    let write_data = Arc::clone(&data);
    let write_start = std::time::Instant::now();
    let write_handle = thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            for _ in 0..iterations {
                let mut guard = write_data.write().await;
                *guard += 1;
            }
        });
    });

    write_handle.join().unwrap();
    let write_duration = write_start.elapsed();

    // Test read performance
    let read_start = std::time::Instant::now();
    let mut read_handles = vec![];

    for _ in 0..4 {
        let read_data = Arc::clone(&data);
        read_handles.push(thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                for _ in 0..iterations {
                    let _guard = read_data.read().await;
                }
            });
        }));
    }

    for handle in read_handles {
        handle.join().unwrap();
    }
    let read_duration = read_start.elapsed();

    println!("Performance results:");
    println!(
        "  {} writes in {:?} ({:.0} writes/sec)",
        iterations,
        write_duration,
        iterations as f64 / write_duration.as_secs_f64()
    );
    println!(
        "  {} reads x 4 threads in {:?} ({:.0} reads/sec)",
        iterations,
        read_duration,
        (iterations * 4) as f64 / read_duration.as_secs_f64()
    );

    #[cfg(debug_assertions)]
    println!("\nNote: Running in debug mode with deadlock detection enabled.");
    #[cfg(not(debug_assertions))]
    println!("\nNote: Running in release mode with minimal overhead.");
}
