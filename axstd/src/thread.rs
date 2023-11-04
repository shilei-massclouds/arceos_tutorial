use core::num::NonZeroU64;
use core::cell::UnsafeCell;
use alloc::sync::Arc;
use crate::String;
use crate::io::{self, Result, IoError};

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

#[derive(Debug)]
pub struct Builder {
    // A name for the thread-to-be, for identification in panic messages
    name: Option<String>,
    // The size of the stack for the spawned thread in bytes
    stack_size: Option<usize>,
}

impl Builder {
    pub const fn new() -> Builder {
        Builder {
            name: None,
            stack_size: None,
        }
    }
    /*

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
    */

    /// Spawns a new thread by taking ownership of the `Builder`, and returns an
    /// [`Result`] to its [`JoinHandle`].
    ///
    /// The spawned thread may outlive the caller (unless the caller thread
    /// is the main thread; the whole process is terminated when the main
    /// thread finishes). The join handle can be used to block on
    /// termination of the spawned thread.
    pub fn spawn<F, T>(self, f: F) -> io::Result<JoinHandle<T>>
    where
        F: FnOnce() -> T + 'static,
        F: 'static,
        T: 'static,
    {
        unsafe { self.spawn_unchecked(f) }
    }

    unsafe fn spawn_unchecked<F, T>(self, f: F) -> Result<JoinHandle<T>>
    where
        F: FnOnce() -> T + 'static,
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

pub struct JoinHandle<T> {
    native: AxTaskHandle,
    thread: Thread,
    packet: Arc<Packet<T>>,
}

unsafe impl<T> Send for JoinHandle<T> {}
unsafe impl<T> Sync for JoinHandle<T> {}

impl<T> JoinHandle<T> {
    pub fn thread(&self) -> &Thread {
        &self.thread
    }

    pub fn join(mut self) -> Result<T> {
        Self::wait_for_exit(self.native).ok_or_else(|| IoError::BadState)?;
        Arc::get_mut(&mut self.packet)
            .unwrap()
            .result
            .get_mut()
            .take()
            .ok_or_else(|| IoError::BadState)
    }

    fn wait_for_exit(task: AxTaskHandle) -> Option<i32> {
        task.inner.join()
    }
}
