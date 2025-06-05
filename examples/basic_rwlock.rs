//! Example demonstrating basic CustomRwLock usage
//!
//! This example shows:
//! - Creating a CustomRwLock
//! - Using read guards (multiple concurrent readers)
//! - Using write guards (exclusive access)
//! - Deadlock detection in debug mode

use std::sync::Arc;
use std::thread;
use std::time::Duration;
use undeadlock::CustomRwLock;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("=== CustomRwLock Basic Example ===\n");

    // Create a shared RwLock with a vector
    let data = Arc::new(CustomRwLock::new(vec![1, 2, 3, 4, 5]));

    // Example 1: Multiple concurrent readers
    println!("1. Multiple concurrent readers:");
    {
        let data_clone1 = Arc::clone(&data);
        let data_clone2 = Arc::clone(&data);
        let data_clone3 = Arc::clone(&data);

        let reader1 = thread::spawn(move || {
            let guard = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(data_clone1.read());
            println!("  Reader 1: {:?}", *guard);
            thread::sleep(Duration::from_millis(100));
            println!("  Reader 1: done");
        });

        let reader2 = thread::spawn(move || {
            let guard = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(data_clone2.read());
            println!("  Reader 2: {:?}", *guard);
            thread::sleep(Duration::from_millis(100));
            println!("  Reader 2: done");
        });

        let reader3 = thread::spawn(move || {
            let guard = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(data_clone3.read());
            println!("  Reader 3: {:?}", *guard);
            thread::sleep(Duration::from_millis(100));
            println!("  Reader 3: done");
        });

        reader1.join().unwrap();
        reader2.join().unwrap();
        reader3.join().unwrap();
    }

    println!("\n2. Exclusive writer:");
    {
        let data_clone = Arc::clone(&data);
        let writer = thread::spawn(move || {
            let mut guard = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(data_clone.write());
            println!("  Writer: Current data = {:?}", *guard);
            guard.push(6);
            println!("  Writer: After push = {:?}", *guard);
        });

        writer.join().unwrap();
    }

    println!("\n3. Reader after write:");
    {
        let guard = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(data.read());
        println!("  Final data: {:?}", *guard);
    }

    // Example 2: Async usage
    println!("\n4. Async usage example:");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let async_data = Arc::new(CustomRwLock::new(String::from("Hello")));

        // Spawn async readers
        let data1 = Arc::clone(&async_data);
        let data2 = Arc::clone(&async_data);

        let task1 = tokio::spawn(async move {
            let guard = data1.read().await;
            println!("  Async reader 1: {}", *guard);
        });

        let task2 = tokio::spawn(async move {
            let guard = data2.read().await;
            println!("  Async reader 2: {}", *guard);
        });

        task1.await.unwrap();
        task2.await.unwrap();

        // Async writer
        {
            let mut guard = async_data.write().await;
            guard.push_str(", World!");
            println!("  Async writer: Modified to '{}'", *guard);
        }

        // Final read
        let guard = async_data.read().await;
        println!("  Final async read: {}", *guard);
    });

    // Example 3: Nested data structures
    println!("\n5. Nested data structure example:");
    #[derive(Debug)]
    struct Counter {
        value: i32,
        name: String,
    }

    let counter = Arc::new(CustomRwLock::new(Counter {
        value: 0,
        name: String::from("Main Counter"),
    }));

    // Multiple threads incrementing the counter
    let mut handles = vec![];
    for i in 0..5 {
        let counter_clone = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // Read current value
                let current = {
                    let guard = counter_clone.read().await;
                    guard.value
                };
                println!("  Thread {}: Read value = {}", i, current);

                // Small delay to simulate work
                tokio::time::sleep(Duration::from_millis(10)).await;

                // Write new value
                let mut guard = counter_clone.write().await;
                guard.value += 1;
                println!("  Thread {}: Incremented to {}", i, guard.value);
            });
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Final counter value
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let guard = counter.read().await;
        println!("\n  Final counter: {} = {}", guard.name, guard.value);
    });

    println!("\n=== Example completed successfully ===");
}
