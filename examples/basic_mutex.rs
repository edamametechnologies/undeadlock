//! Example demonstrating basic CustomMutex usage
//!
//! This example shows:
//! - Creating a CustomMutex
//! - Using mutex guards (exclusive access)
//! - Concurrent access patterns
//! - Deadlock detection in debug mode

use std::sync::Arc;
use std::thread;
use std::time::Duration;
use undeadlock::CustomMutex;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("=== CustomMutex Basic Example ===\n");

    // Example 1: Basic exclusive access
    println!("1. Basic exclusive access:");
    {
        let data = Arc::new(CustomMutex::new(vec![1, 2, 3, 4, 5]));

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut guard = data.lock().await;
            println!("  Current data: {:?}", *guard);
            guard.push(6);
            println!("  After push: {:?}", *guard);
        });
    }

    // Example 2: Concurrent access with contention
    println!("\n2. Concurrent access with contention:");
    {
        let counter = Arc::new(CustomMutex::new(0i32));
        let mut handles = vec![];

        // Spawn multiple threads that increment the counter
        for i in 0..5 {
            let counter_clone = Arc::clone(&counter);
            let handle = thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    for j in 0..3 {
                        let mut guard = counter_clone.lock().await;
                        let old_value = *guard;
                        *guard += 1;
                        println!(
                            "  Thread {} iteration {}: {} -> {}",
                            i, j, old_value, *guard
                        );

                        // Simulate some work while holding the lock
                        tokio::time::sleep(Duration::from_millis(50)).await;
                    }
                });
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Final value
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let guard = counter.lock().await;
            println!("  Final counter value: {}", *guard);
        });
    }

    // Example 3: Protecting complex data structures
    println!("\n3. Protecting complex data structures:");
    {
        #[derive(Debug)]
        struct SharedResource {
            name: String,
            connections: Vec<String>,
            active: bool,
        }

        let resource = Arc::new(CustomMutex::new(SharedResource {
            name: "DatabasePool".to_string(),
            connections: vec![],
            active: true,
        }));

        let mut handles = vec![];

        // Simulate multiple clients connecting
        for i in 0..3 {
            let resource_clone = Arc::clone(&resource);
            let handle = thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let mut guard = resource_clone.lock().await;

                    if guard.active {
                        let client_id = format!("client_{}", i);
                        guard.connections.push(client_id.clone());
                        println!("  {} connected to {}", client_id, guard.name);
                        println!("  Active connections: {:?}", guard.connections);
                    }

                    // Simulate connection work
                    tokio::time::sleep(Duration::from_millis(100)).await;
                });
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Final state
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let guard = resource.lock().await;
            println!("  Final resource state: {:?}", *guard);
        });
    }

    // Example 4: Async usage patterns
    println!("\n4. Async usage patterns:");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let shared_state = Arc::new(CustomMutex::new(String::from("Initial")));

        // Spawn async tasks
        let mut tasks = vec![];

        for i in 0..3 {
            let state_clone = Arc::clone(&shared_state);
            let task = tokio::spawn(async move {
                let mut guard = state_clone.lock().await;
                let old_value = guard.clone();
                guard.push_str(&format!(" -> Task{}", i));
                println!("  Task {}: '{}' -> '{}'", i, old_value, *guard);

                // Simulate async work
                tokio::time::sleep(Duration::from_millis(30)).await;
            });
            tasks.push(task);
        }

        // Wait for all tasks
        for task in tasks {
            task.await.unwrap();
        }

        // Final state
        let guard = shared_state.lock().await;
        println!("  Final state: '{}'", *guard);
    });

    // Example 5: Queue operations
    println!("\n5. Queue operations:");
    {
        let queue = Arc::new(CustomMutex::new(Vec::<String>::new()));
        let mut handles = vec![];

        // Producer threads
        for i in 0..2 {
            let queue_clone = Arc::clone(&queue);
            let handle = thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    for j in 0..3 {
                        let mut guard = queue_clone.lock().await;
                        let item = format!("item_{}_{}", i, j);
                        guard.push(item.clone());
                        println!("  Producer {}: Added {}", i, item);

                        // Small delay to show interleaving
                        tokio::time::sleep(Duration::from_millis(20)).await;
                    }
                });
            });
            handles.push(handle);
        }

        // Consumer thread
        let queue_clone = Arc::clone(&queue);
        let consumer_handle = thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // Wait a bit for producers to add items
                tokio::time::sleep(Duration::from_millis(50)).await;

                for _ in 0..6 {
                    let mut guard = queue_clone.lock().await;
                    if let Some(item) = guard.pop() {
                        println!("  Consumer: Processed {}", item);
                    }

                    // Small delay between processing
                    tokio::time::sleep(Duration::from_millis(30)).await;
                }
            });
        });

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
        consumer_handle.join().unwrap();

        // Check remaining items
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let guard = queue.lock().await;
            println!("  Remaining items in queue: {:?}", *guard);
        });
    }

    // Example 6: Error handling
    println!("\n6. Error handling:");
    {
        let result_holder = Arc::new(CustomMutex::new(Ok::<i32, String>(42)));

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // Simulate an operation that might fail
            let mut guard = result_holder.lock().await;
            match &mut *guard {
                Ok(value) => {
                    println!("  Current value: {}", value);
                    *value += 1;
                    println!("  Updated value: {}", value);
                }
                Err(e) => {
                    println!("  Error state: {}", e);
                }
            }

            // Simulate setting an error
            *guard = Err("Something went wrong".to_string());
        });

        // Check error state
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let guard = result_holder.lock().await;
            match &*guard {
                Ok(value) => println!("  Final value: {}", value),
                Err(e) => println!("  Final error: {}", e),
            }
        });
    }

    println!("\n=== Example completed successfully ===");
}
