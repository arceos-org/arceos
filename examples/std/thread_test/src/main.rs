#[cfg(target_os = "hermit")]
use arceos_rust as _;

use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Barrier;

fn main() {
    println!("=== Rust 线程功能测试程序 ===\n");

    // 1. 基本线程创建和等待
    test_basic_threads();

    // 2. 使用move闭包获取所有权
    test_move_closure();

    // 3. 使用通道进行线程间通信
    test_channel_communication();
    
    // 4. 使用Mutex进行线程安全的数据共享
    test_mutex_shared_data();
    
    // 5. 原子操作
    test_atomic_operations();
    
    // 6. 屏障同步
    test_barrier_synchronization();
    
    // 7. 线程恐慌处理
    test_thread_panic();

    println!("\n=== 所有测试完成 ===");
}

// 1. 基本线程创建和等待
fn test_basic_threads() {
    println!("1. 基本线程创建和等待:");

    let handle1 = thread::spawn(|| {
        for i in 1..=5 {
            println!("线程1: 计数 {}", i);
            thread::sleep(Duration::from_millis(100));
        }
    });

    let handle2 = thread::spawn(|| {
        for i in 1..=3 {
            println!("线程2: 工作 {}", i);
            thread::sleep(Duration::from_millis(150));
        }
    });

    // 等待线程完成
    handle1.join().unwrap();
    handle2.join().unwrap();
    println!("基本线程测试完成\n");
}

// 2. 使用move闭包获取所有权
fn test_move_closure() {
    println!("2. 使用move闭包获取所有权:");

    let data = vec![1, 2, 3, 4, 5];
    println!("主线程: 原始数据: {:?}", data);

    let handle = thread::spawn(move || {
        println!("子线程: 接收数据: {:?}", data);
        let sum: i32 = data.iter().sum();
        println!("子线程: 数据求和: {}", sum);
        sum
    });

    // 注意：这里不能再使用data，因为它已经被移动到线程中
    // println!("尝试使用data: {:?}", data); // 这会编译错误

    let result = handle.join().unwrap();
    println!("主线程: 子线程计算结果: {}\n", result);
}

// 3. 使用通道进行线程间通信
fn test_channel_communication() {
    println!("3. 使用通道进行线程间通信:");

    // 创建多生产者，单消费者通道
    let (tx, rx) = mpsc::channel();
    let tx1 = tx.clone();
    let tx2 = tx.clone();

    // 生产者线程1
    let producer1 = thread::spawn(move || {
        let messages = vec!["消息1", "消息2", "消息3"];
        for msg in messages {
            tx1.send(format!("生产者1: {}", msg)).unwrap();
            thread::sleep(Duration::from_millis(50));
        }
    });

    // 生产者线程2
    let producer2 = thread::spawn(move || {
        let messages = vec!["Hello", "World", "Rust"];
        for msg in messages {
            tx2.send(format!("生产者2: {}", msg)).unwrap();
            thread::sleep(Duration::from_millis(30));
        }
    });

    // 消费者线程
    let consumer = thread::spawn(move || {
        // 接收所有消息
        for received in rx {
            println!("消费者: 接收到 - {}", received);
        }
    });

    // 等待生产者完成
    producer1.join().unwrap();
    producer2.join().unwrap();

    // 需要丢弃原始tx，否则rx会一直等待
    drop(tx);

    consumer.join().unwrap();
    println!("通道通信测试完成\n");
}

// 4. 使用Mutex进行线程安全的数据共享
fn test_mutex_shared_data() {
    println!("4. 使用Mutex进行线程安全的数据共享:");

    // 使用Arc（原子引用计数）在多个线程间共享Mutex
    let counter = Arc::new(Mutex::new(0));
    let mut handles = vec![];

    println!("初始计数器: 0");

    for i in 0..10 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            thread::sleep(Duration::from_millis(i * 10));

            let mut num = counter.lock().unwrap();
            *num += 1;
            println!("线程{}: 增加计数器到 {}", i, *num);
        });
        handles.push(handle);
    }

    // 等待所有线程完成
    for handle in handles {
        handle.join().unwrap();
    }

    let final_value = *counter.lock().unwrap();
    println!("最终计数器: {}", final_value);
    assert_eq!(final_value, 10);
    println!("Mutex共享数据测试完成\n");
}

// 5. 原子操作
fn test_atomic_operations() {
    println!("5. 原子操作测试:");

    let counter = Arc::new(AtomicUsize::new(0));
    let mut handles = vec![];

    for _ in 0..100 {
        let counter = Arc::clone(&counter);
        let handle = thread::spawn(move || {
            // 原子增加
            counter.fetch_add(1, Ordering::SeqCst);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("原子计数器结果: {}", counter.load(Ordering::SeqCst));
    println!("原子操作测试完成\n");
}

// 6. 屏障同步
fn test_barrier_synchronization() {
    println!("6. 屏障同步测试:");

    let barrier = Arc::new(Barrier::new(3)); // 等待3个线程
    let mut handles = vec![];

    for i in 0..3 {
        let barrier = barrier.clone();
        let handle = thread::spawn(move || {
            println!("线程{}: 第一阶段工作", i);
            thread::sleep(Duration::from_millis(100 * (i + 1) as u64));

            // 等待所有线程到达屏障
            println!("线程{}: 到达屏障，等待其他线程...", i);
            barrier.wait();

            println!("线程{}: 所有线程已就绪，继续执行", i);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
    println!("屏障同步测试完成\n");
}

// 7. 线程恐慌处理
fn test_thread_panic() {
    println!("7. 线程恐慌处理:");
    println!("Unikernel环境下使用panic_abort模式，不支持恐慌捕获，因此会导致程序终止。");

    let handle = thread::spawn(|| {
        println!("恐慌线程: 即将恐慌...");
        panic!("测试恐慌！");
    });

    // 捕获线程恐慌
    match handle.join() {
        Ok(_) => println!("线程正常结束"),
        Err(e) => {
            println!("线程恐慌被捕获");
            if let Some(s) = e.downcast_ref::<&str>() {
                println!("恐慌信息: {}", s);
            }
        }
    }
    println!("恐慌处理测试完成\n");
}