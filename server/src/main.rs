//! app:v4: build major registry.

#![no_std]
#![no_main]

extern crate alloc;

use axstd::println;
use axstd::thread;
use core::sync::atomic::{AtomicUsize, Ordering};

const PAGE_SIZE: usize = 4096;

static FLAG: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
fn main() {
    raise_break_exception();

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

fn raise_break_exception() {
    unsafe {
        core::arch::asm!("ebreak");
        core::arch::asm!("nop");
        core::arch::asm!("nop");
    }
}
