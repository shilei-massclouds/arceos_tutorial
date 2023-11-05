#![no_std]
#![no_main]

use axstd::{println, String, time, Vec, thread};
use axstd::sync::Mutex;

#[no_mangle]
pub fn main() {
    let now = time::Instant::now();

    let s = String::from("Hello, ArceOS!");
    println!("{s} Now axstd is okay!");

    try_alloc_bulk();

    try_multitask();

    raise_break_exception();

    test_mutex();

    let d = now.elapsed();
    println!("Elapsed: {}.{:06}", d.as_secs(), d.subsec_micros());
}

fn try_alloc_bulk() {
    println!("\nTry alloc bulk memory ...\n");
    let mut v = Vec::new();
    for i in 0..0x2000 {
        v.push(i);
    }
    println!("Alloc bulk memory ok!\n");
}

fn try_multitask() {
    println!("Start task...");

    let computation = thread::spawn(|| {
        42
    });

    let result = computation.join().unwrap();
    println!("Task gets result: {result}\n");
}

fn raise_break_exception() {
    unsafe {
        core::arch::asm!("ebreak");
        core::arch::asm!("nop");
        core::arch::asm!("nop");
    }
}

fn test_mutex() {
    extern crate alloc;
    use alloc::sync::Arc;
    let flag = Arc::new(Mutex::new(0));
    let flag2 = flag.clone();

    thread::spawn(move || {
        println!("Spawned-thread starts ...");
        let mut lock = flag2.lock();
        *lock += 1;
        println!("Spawned-thread starts ok! {}", *lock);
    });

    {
        println!("Main thread set FLAG to notify spawned-thread to continue.");
        let mut lock = flag.lock();
        *lock += 1;
    }

    println!("Main thread waits spawned-thread to respond ...");
    loop {
        let lock = flag.lock();
        if *lock >= 2 {
            break;
        }
    }
    println!("Mutex test run OK!");
}
