//! Example demonstrating CustomDashMap usage
//!
//! This example shows:
//! - Creating and using a CustomDashMap
//! - Concurrent access patterns
//! - Entry API usage
//! - Iteration and filtering
//! - Deadlock detection in debug mode

use std::sync::Arc;
use std::thread;
use std::time::Duration;
use undeadlock::CustomDashMap;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("=== CustomDashMap Usage Example ===\n");

    // Example 1: Basic operations
    println!("1. Basic operations:");
    {
        let map = CustomDashMap::new("basic_map");

        // Insert some values
        map.insert("apple", 5);
        map.insert("banana", 3);
        map.insert("cherry", 8);

        // Get values
        if let Some(apple_count) = map.get(&"apple") {
            println!("  Apples: {}", *apple_count);
        }

        // Check if key exists
        println!("  Contains 'banana': {}", map.contains_key(&"banana"));
        println!("  Contains 'grape': {}", map.contains_key(&"grape"));

        // Remove a value
        if let Some((key, value)) = map.remove(&"banana") {
            println!("  Removed {} with count {}", key, value);
        }

        println!("  Map size: {}", map.len());
    }

    // Example 2: Concurrent access
    println!("\n2. Concurrent access:");
    {
        let shared_map = Arc::new(CustomDashMap::new("concurrent_map"));
        let mut handles = vec![];

        // Spawn multiple threads to insert values
        for i in 0..5 {
            let map_clone = Arc::clone(&shared_map);
            let handle = thread::spawn(move || {
                for j in 0..10 {
                    let key = format!("thread{}_{}", i, j);
                    map_clone.insert(key, i * 10 + j);
                }
                println!("  Thread {} inserted 10 values", i);
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        println!("  Total entries: {}", shared_map.len());
    }

    // Example 3: Entry API
    println!("\n3. Entry API usage:");
    {
        let counter_map = CustomDashMap::new("counter_map");

        // Count occurrences
        let words = vec!["hello", "world", "hello", "rust", "world", "hello"];
        for word in words {
            counter_map
                .entry(word)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }

        // Print counts
        println!("  Word counts:");
        for entry in counter_map.iter() {
            let (word, count) = entry.pair();
            println!("    {}: {}", word, count);
        }
    }

    // Example 4: Mutable access
    println!("\n4. Mutable access:");
    {
        let scores = CustomDashMap::new("scores");
        scores.insert("Alice", 100);
        scores.insert("Bob", 85);
        scores.insert("Charlie", 92);

        // Update Bob's score
        if let Some(mut bob_score) = scores.get_mut(&"Bob") {
            *bob_score += 10;
            println!("  Updated Bob's score to: {}", *bob_score);
        }

        // Double all scores
        for mut entry in scores.iter_mut() {
            *entry.value_mut() *= 2;
        }

        println!("  Doubled scores:");
        for entry in scores.iter() {
            let (name, score) = entry.pair();
            println!("    {}: {}", name, score);
        }
    }

    // Example 5: Filtering with retain
    println!("\n5. Filtering with retain:");
    {
        let inventory = CustomDashMap::new("inventory");
        inventory.insert("apples", 10);
        inventory.insert("bananas", 0);
        inventory.insert("oranges", 5);
        inventory.insert("grapes", 0);
        inventory.insert("pears", 3);

        println!("  Before filtering: {} items", inventory.len());

        // Remove items with zero count
        inventory.retain(|_, &mut count| count > 0);

        println!("  After filtering: {} items", inventory.len());
        for entry in inventory.iter() {
            let (item, count) = entry.pair();
            println!("    {}: {}", item, count);
        }
    }

    // Example 6: Complex data types
    println!("\n6. Complex data types:");
    {
        #[derive(Debug, Clone)]
        struct User {
            _id: u32,
            name: String,
            score: i32,
        }

        let users = CustomDashMap::new("users");

        // Insert some users
        users.insert(
            1001,
            User {
                _id: 1001,
                name: "Alice".to_string(),
                score: 0,
            },
        );

        users.insert(
            1002,
            User {
                _id: 1002,
                name: "Bob".to_string(),
                score: 0,
            },
        );

        // Update scores concurrently
        let users_arc = Arc::new(users);
        let mut handles = vec![];

        for i in 0..3 {
            let users_clone = Arc::clone(&users_arc);
            let handle = thread::spawn(move || {
                // Each thread updates all users' scores
                for entry in users_clone.iter() {
                    let user_id = *entry.key();
                    drop(entry); // Release read lock before getting write lock

                    if let Some(mut user_ref) = users_clone.get_mut(&user_id) {
                        user_ref.score += (i + 1) as i32;
                    }
                }
                println!("  Thread {} updated all scores", i);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Print final scores
        println!("  Final user scores:");
        for entry in users_arc.iter() {
            let user = entry.value();
            println!("    {}: {}", user.name, user.score);
        }
    }

    // Example 7: Async usage
    println!("\n7. Async usage:");
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let async_map = Arc::new(CustomDashMap::new("async_map"));

        // Spawn async tasks
        let map1 = Arc::clone(&async_map);
        let map2 = Arc::clone(&async_map);

        let task1 = tokio::spawn(async move {
            // Get write access
            let _guard = map1.write().await;
            map1.insert("task1", 100);
            println!("  Task 1 inserted value");
        });

        let task2 = tokio::spawn(async move {
            // Wait a bit then insert
            tokio::time::sleep(Duration::from_millis(50)).await;
            let _guard = map2.write().await;
            map2.insert("task2", 200);
            println!("  Task 2 inserted value");
        });

        task1.await.unwrap();
        task2.await.unwrap();

        // Read values
        println!("  Final map contents:");
        for entry in async_map.iter() {
            let (key, value) = entry.pair();
            println!("    {}: {}", key, value);
        }
    });

    // Example 8: Drain operation
    println!("\n8. Drain operation:");
    {
        let temp_map = CustomDashMap::new("temp_map");
        temp_map.insert("item1", 10);
        temp_map.insert("item2", 20);
        temp_map.insert("item3", 30);

        println!("  Before drain: {} items", temp_map.len());

        // Drain all items
        let drained: Vec<_> = temp_map.drain().collect();

        println!("  After drain: {} items", temp_map.len());
        println!("  Drained items:");
        for (key, value) in drained {
            println!("    {}: {}", key, value);
        }
    }

    println!("\n=== Example completed successfully ===");
}
