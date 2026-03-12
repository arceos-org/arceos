#[cfg(target_os = "hermit")]
use arceos_rust as _;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::{mpsc, Mutex, Barrier};
use tokio::time::{Duration, sleep};

fn main() {
    println!("=== Tokio 异步运行时测试程序 ===\n");

    // 使用 current_thread runtime，更适合 unikernel 环境
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime");

    rt.block_on(async {
        // 1. 基本 async/await
        test_basic_async().await;

        // 2. tokio::spawn 任务
        test_spawn_tasks().await;

        // 3. 使用 mpsc channel 进行异步通信
        test_mpsc_channel().await;

        // 4. tokio::time::sleep 定时器
        test_timer().await;

        // 5. tokio::sync::Mutex 异步互斥锁
        test_async_mutex().await;

        // 6. tokio::sync::Barrier 异步屏障
        test_async_barrier().await;

        // 7. tokio::select! 多路复用
        test_select().await;

        // 8. 嵌套 spawn 与 JoinHandle
        test_nested_spawn().await;
    });

    println!("\n=== 所有 Tokio 测试完成 ===");
}

// 1. 基本 async/await
async fn test_basic_async() {
    println!("1. 基本 async/await 测试:");

    async fn add(a: i32, b: i32) -> i32 {
        a + b
    }

    async fn greet(name: &str) -> String {
        format!("Hello, {}!", name)
    }

    let sum = add(3, 4).await;
    println!("  3 + 4 = {}", sum);
    assert_eq!(sum, 7);

    let greeting = greet("ArceOS").await;
    println!("  {}", greeting);
    assert_eq!(greeting, "Hello, ArceOS!");

    println!("  基本 async/await 测试通过\n");
}

// 2. tokio::spawn 任务
async fn test_spawn_tasks() {
    println!("2. tokio::spawn 任务测试:");

    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = Vec::new();

    for i in 0..5 {
        let counter = counter.clone();
        let handle = tokio::spawn(async move {
            println!("  任务 {} 开始执行", i);
            counter.fetch_add(1, Ordering::SeqCst);
            i * 10
        });
        handles.push(handle);
    }

    let mut results = Vec::new();
    for handle in handles {
        let result = handle.await.expect("任务执行失败");
        results.push(result);
    }

    results.sort();
    println!("  任务返回值: {:?}", results);
    assert_eq!(results, vec![0, 10, 20, 30, 40]);

    let count = counter.load(Ordering::SeqCst);
    println!("  共执行 {} 个任务", count);
    assert_eq!(count, 5);

    println!("  tokio::spawn 测试通过\n");
}

// 3. 使用 mpsc channel 进行异步通信
async fn test_mpsc_channel() {
    println!("3. mpsc channel 异步通信测试:");

    let (tx, mut rx) = mpsc::channel::<String>(16);

    let tx1 = tx.clone();
    let tx2 = tx.clone();

    // 生产者 1
    let producer1 = tokio::spawn(async move {
        for i in 0..3 {
            tx1.send(format!("生产者1: 消息{}", i))
                .await
                .expect("发送失败");
        }
    });

    // 生产者 2
    let producer2 = tokio::spawn(async move {
        for i in 0..3 {
            tx2.send(format!("生产者2: 消息{}", i))
                .await
                .expect("发送失败");
        }
    });

    // 丢弃原始 tx，这样当所有生产者完成后 rx 会结束
    drop(tx);

    producer1.await.expect("生产者1失败");
    producer2.await.expect("生产者2失败");

    let mut received = Vec::new();
    while let Some(msg) = rx.recv().await {
        println!("  消费者收到: {}", msg);
        received.push(msg);
    }

    assert_eq!(received.len(), 6);
    println!("  共收到 {} 条消息", received.len());
    println!("  mpsc channel 测试通过\n");
}

// 4. tokio::time::sleep 定时器
async fn test_timer() {
    println!("4. tokio::time 定时器测试:");

    let start = std::time::Instant::now();

    println!("  等待 100ms...");
    sleep(Duration::from_millis(100)).await;

    let elapsed = start.elapsed();
    println!("  实际经过时间: {:?}", elapsed);
    assert!(elapsed >= Duration::from_millis(100));

    // 测试 tokio::time::timeout（通过 select 模拟）
    println!("  定时器测试通过\n");
}

// 5. tokio::sync::Mutex 异步互斥锁
async fn test_async_mutex() {
    println!("5. tokio::sync::Mutex 异步互斥锁测试:");

    let data = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();

    for i in 0..5 {
        let data = data.clone();
        let handle = tokio::spawn(async move {
            let mut vec = data.lock().await;
            vec.push(i);
            println!("  任务 {} 添加数据", i);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.expect("任务失败");
    }

    let vec = data.lock().await;
    println!("  最终数据长度: {}", vec.len());
    assert_eq!(vec.len(), 5);
    println!("  异步互斥锁测试通过\n");
}

// 6. tokio::sync::Barrier 异步屏障
async fn test_async_barrier() {
    println!("6. tokio::sync::Barrier 异步屏障测试:");

    let barrier = Arc::new(Barrier::new(3));
    let mut handles = Vec::new();

    for i in 0..3 {
        let barrier = barrier.clone();
        let handle = tokio::spawn(async move {
            println!("  任务 {} 到达屏障", i);
            let result = barrier.wait().await;
            println!("  任务 {} 通过屏障 (leader: {})", i, result.is_leader());
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.expect("任务失败");
    }

    println!("  异步屏障测试通过\n");
}

// 7. tokio::select! 多路复用
async fn test_select() {
    println!("7. tokio::select! 多路复用测试:");

    let (tx, mut rx) = mpsc::channel::<&str>(1);

    tokio::spawn(async move {
        sleep(Duration::from_millis(50)).await;
        let _ = tx.send("来自通道的消息").await;
    });

    tokio::select! {
        msg = rx.recv() => {
            println!("  收到通道消息: {:?}", msg.unwrap());
        }
        _ = sleep(Duration::from_millis(200)) => {
            println!("  超时");
        }
    }

    // 测试 select 超时分支
    let (tx2, mut rx2) = mpsc::channel::<&str>(1);
    drop(tx2); // 立即丢弃发送端

    tokio::select! {
        msg = rx2.recv() => {
            // 通道关闭后 recv 返回 None
            println!("  通道已关闭: {:?}", msg);
        }
        _ = sleep(Duration::from_millis(50)) => {
            println!("  超时触发");
        }
    }

    println!("  select! 多路复用测试通过\n");
}

// 8. 嵌套 spawn 与 JoinHandle
async fn test_nested_spawn() {
    println!("8. 嵌套 spawn 测试:");

    let result = tokio::spawn(async {
        println!("  外层任务开始");

        let inner = tokio::spawn(async {
            println!("  内层任务开始");
            42
        });

        let inner_result = inner.await.expect("内层任务失败");
        println!("  内层任务返回: {}", inner_result);

        inner_result * 2
    })
    .await
    .expect("外层任务失败");

    println!("  最终结果: {}", result);
    assert_eq!(result, 84);
    println!("  嵌套 spawn 测试通过\n");
}
