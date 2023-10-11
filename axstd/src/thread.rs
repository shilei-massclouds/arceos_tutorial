use core::num::NonZeroU64;
use core::cell::UnsafeCell;
use alloc::sync::Arc;
use crate::{Result, String};
use axerrno::ax_err_type;

/// A handle to a task.
pub struct AxTaskHandle {
    inner: axtask::AxTaskRef,
    id: u64,
}

/// A unique identifier for a running thread.
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub struct ThreadId(NonZeroU64);

/// A handle to a thread.
pub struct Thread {
    id: ThreadId,
}

impl Thread {
    fn from_id(id: u64) -> Self {
        Self {
            id: ThreadId(NonZeroU64::new(id).unwrap()),
        }
    }

    /// Gets the thread's unique identifier.
    pub fn id(&self) -> ThreadId {
        self.id
    }
}

/// Thread factory, which can be used in order to configure the properties of
/// a new thread.
///
/// Methods can be chained on it in order to configure it.
#[derive(Debug)]
pub struct Builder {
    // A name for the thread-to-be, for identification in panic messages
    name: Option<String>,
    // The size of the stack for the spawned thread in bytes
    stack_size: Option<usize>,
}

impl Builder {
    /// Generates the base configuration for spawning a thread, from which
    /// configuration methods can be chained.
    pub const fn new() -> Builder {
        Builder {
            name: None,
            stack_size: None,
        }
    }

    /// Names the thread-to-be.
    pub fn name(mut self, name: String) -> Builder {
        self.name = Some(name);
        self
    }

    /// Sets the size of the stack (in bytes) for the new thread.
    pub fn stack_size(mut self, size: usize) -> Builder {
        self.stack_size = Some(size);
        self
    }

    /// Spawns a new thread by taking ownership of the `Builder`, and returns an
    /// [`Result`] to its [`JoinHandle`].
    ///
    /// The spawned thread may outlive the caller (unless the caller thread
    /// is the main thread; the whole process is terminated when the main
    /// thread finishes). The join handle can be used to block on
    /// termination of the spawned thread.
    pub fn spawn<F, T>(self, f: F) -> Result<JoinHandle<T>>
    where
        F: FnOnce() -> T + 'static,
        //F: Send + 'static,
        //T: Send + 'static,
        F: 'static,
        T: 'static,
    {
        unsafe { self.spawn_unchecked(f) }
    }

    unsafe fn spawn_unchecked<F, T>(self, f: F) -> Result<JoinHandle<T>>
    where
        F: FnOnce() -> T + 'static,
        //F: Send + 'static,
        //T: Send + 'static,
        F: 'static,
        T: 'static,
    {
        let name = self.name.unwrap_or_default();
        let stack_size = self
            .stack_size
            .unwrap_or(axconfig::TASK_STACK_SIZE);

        let my_packet = Arc::new(Packet {
            result: UnsafeCell::new(None),
        });
        let their_packet = my_packet.clone();

        let main = move || {
            let ret = f();
            // SAFETY: `their_packet` as been built just above and moved by the
            // closure (it is an Arc<...>) and `my_packet` will be stored in the
            // same `JoinHandle` as this closure meaning the mutation will be
            // safe (not modify it and affect a value far away).
            unsafe { *their_packet.result.get() = Some(ret) };
            drop(their_packet);
        };

        let inner = axtask::spawn_raw(main, name, stack_size);
        let task = AxTaskHandle {
            id: inner.id().as_u64(),
            inner,
        };
        Ok(JoinHandle {
            thread: Thread::from_id(task.id),
            native: task,
            packet: my_packet,
        })
    }
}

/// Spawns a new thread, returning a [`JoinHandle`] for it.
///
/// The join handle provides a [`join`] method that can be used to join the
/// spawned thread.
///
/// The default task name is an empty string. The default thread stack size is
/// [`arceos_api::config::TASK_STACK_SIZE`].
///
/// [`join`]: JoinHandle::join
/*
pub fn spawn<T, F>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T + Send + 'static,
    T: Send + 'static,
    */
pub fn spawn<T, F>(f: F) -> JoinHandle<T>
where
    F: FnOnce() -> T + 'static,
    T: 'static,
{
    Builder::new().spawn(f).expect("failed to spawn thread")
}

struct Packet<T> {
    result: UnsafeCell<Option<T>>,
}

unsafe impl<T> Sync for Packet<T> {}

/// An owned permission to join on a thread (block on its termination).
///
/// A `JoinHandle` *detaches* the associated thread when it is dropped, which
/// means that there is no longer any handle to the thread and no way to `join`
/// on it.
pub struct JoinHandle<T> {
    native: AxTaskHandle,
    thread: Thread,
    packet: Arc<Packet<T>>,
}

unsafe impl<T> Send for JoinHandle<T> {}
unsafe impl<T> Sync for JoinHandle<T> {}

impl<T> JoinHandle<T> {
    /// Extracts a handle to the underlying thread.
    pub fn thread(&self) -> &Thread {
        &self.thread
    }

    /// Waits for the associated thread to finish.
    ///
    /// This function will return immediately if the associated thread has
    /// already finished.
    pub fn join(mut self) -> crate::Result<T> {
        Self::wait_for_exit(self.native).ok_or_else(|| ax_err_type!(BadState))?;
        Arc::get_mut(&mut self.packet)
            .unwrap()
            .result
            .get_mut()
            .take()
            .ok_or_else(|| ax_err_type!(BadState))
    }

    fn wait_for_exit(task: AxTaskHandle) -> Option<i32> {
        task.inner.join()
    }
}
