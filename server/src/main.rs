//! app:v4: build major registry.

#![no_std]
#![no_main]

extern crate alloc;

use axstd::println;
use axstd::thread;
use axstd::sync::Mutex;
use alloc::sync::Arc;

#[no_mangle]
fn main() {
    //raise_break_exception();
    //cooperative_preemptive();

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
        let mut lock = flag.lock();
        if *lock >= 2 {
            break;
        }
    }
    println!("Mutex test run OK!");
}

/*
fn raise_break_exception() {
    unsafe {
        core::arch::asm!("ebreak");
        core::arch::asm!("nop");
        core::arch::asm!("nop");
    }
}

fn cooperative_preemptive() {
    use core::sync::atomic::{AtomicUsize, Ordering};

    static FLAG: AtomicUsize = AtomicUsize::new(0);

    thread::spawn(move || {
        println!("Spawned-thread is waiting ...");
        while FLAG.load(Ordering::Relaxed) < 1 {
            // For cooperative scheduler, we must yield here!
            // For preemptive scheduler, just relaxed! Leave it for scheduler.
        }

        let _ = FLAG.fetch_add(1, Ordering::Relaxed);
    });

    // Give spawned thread a chance to start.
    thread::yield_now();

    println!("Main thread set FLAG to notify spawned-thread to continue.");
    let _ = FLAG.fetch_add(1, Ordering::Relaxed);
    println!("Main thread waits spawned-thread to respond ...");
    while FLAG.load(Ordering::Relaxed) < 2 {
        thread::yield_now();
    }
    println!("Preempt test run OK!");
}
*/
