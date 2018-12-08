use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};

use log::trace;

/// A wrapper allocator that logs messages on allocation.
pub struct LoggingAllocator<A = System> {
    enabled: AtomicBool,
    allocator: A,
}

impl LoggingAllocator<System> {
    pub const fn new() -> Self {
        LoggingAllocator::with_allocator(System)
    }
}

impl<A> LoggingAllocator<A> {
    pub const fn with_allocator(allocator: A) -> Self {
        LoggingAllocator {
            enabled: AtomicBool::new(false),
            allocator,
        }
    }

    pub fn enable_logging(&self) {
        self.enabled.store(true, Ordering::SeqCst)
    }

    pub fn disable_logging(&self) {
        self.enabled.store(false, Ordering::SeqCst)
    }

    pub fn logging_enabled(&self) -> bool {
        self.enabled.load(Ordering::SeqCst)
    }
}

/// Execute a closure without logging on allocations.
pub fn run_guarded<F>(f: F)
where
    F: FnOnce(),
{
    thread_local! {
        static GUARD: Cell<bool> = Cell::new(false);
    }

    GUARD.with(|guard| {
        if !guard.replace(true) {
            f();
            guard.set(false)
        }
    })
}

unsafe impl<A> GlobalAlloc for LoggingAllocator<A>
where
    A: GlobalAlloc,
{
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.allocator.alloc(layout);
        if self.logging_enabled() {
            run_guarded(|| {
                trace!("alloc {}", Fmt(ptr, layout.size(), layout.align()));
            });
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.allocator.dealloc(ptr, layout);
        if self.logging_enabled() {
            run_guarded(|| trace!("dealloc {}", Fmt(ptr, layout.size(), layout.align()),));
        }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let ptr = self.allocator.alloc_zeroed(layout);
        if self.logging_enabled() {
            run_guarded(|| {
                trace!("alloc_zeroed {}", Fmt(ptr, layout.size(), layout.align()));
            });
        }
        ptr
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = self.allocator.realloc(ptr, layout, new_size);
        if self.logging_enabled() {
            run_guarded(|| {
                trace!(
                    "realloc {} to {}",
                    Fmt(ptr, layout.size(), layout.align()),
                    Fmt(new_ptr, new_size, layout.align())
                );
            });
        }
        new_ptr
    }
}

struct Fmt(*mut u8, usize, usize);

impl fmt::Display for Fmt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[address={:p}, size={}, align={}]",
            self.0, self.1, self.2
        )
    }
}
