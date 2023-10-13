use axsync::BootOnceCell;
use handler_table::HandlerTable;
use riscv::register::{sstatus, sie};

pub const MAX_IRQ_COUNT: usize = 1024;
pub(super) const INTC_IRQ_BASE: usize = 1 << (usize::BITS - 1);

#[allow(unused)]
pub(super) const S_SOFT: usize = INTC_IRQ_BASE + 1;
pub(super) const S_TIMER: usize = INTC_IRQ_BASE + 5;
pub(super) const S_EXT: usize = INTC_IRQ_BASE + 9;

pub const TIMER_IRQ_NUM: usize = S_TIMER;

pub type IrqHandler = handler_table::Handler;

static IRQ_HANDLER_TABLE: HandlerTable<MAX_IRQ_COUNT> = HandlerTable::new();

static TIMER_HANDLER: BootOnceCell<IrqHandler> = unsafe {
    BootOnceCell::new()
};

pub fn dispatch_irq(scause: usize) {
    match scause {
        S_TIMER => {
            log::trace!("IRQ: timer");
            TIMER_HANDLER.get()();
        },
        S_EXT => {
            crate::irq::dispatch_irq_common(0);
        },
        _ => panic!("invalid trap cause: {:#x}", scause),
    }
}

#[allow(dead_code)]
pub(crate) fn dispatch_irq_common(irq_num: usize) {
    log::trace!("IRQ {}", irq_num);
    if !IRQ_HANDLER_TABLE.handle(irq_num) {
        log::warn!("Unhandled IRQ {}", irq_num);
    }
}

#[allow(dead_code)]
pub(crate) fn register_handler_common(irq_num: usize, handler: IrqHandler) -> bool {
    if irq_num < MAX_IRQ_COUNT && IRQ_HANDLER_TABLE.register_handler(irq_num, handler) {
        return true;
    }
    log::warn!("register handler for IRQ {} failed", irq_num);
    false
}

pub fn register_handler(scause: usize, handler: IrqHandler) -> bool {
    match scause {
        S_TIMER => {
            if !TIMER_HANDLER.is_init() {
                TIMER_HANDLER.init(handler);
                true
            } else {
                false
            }
        },
        S_EXT => {
            crate::irq::register_handler_common(scause & !INTC_IRQ_BASE, handler)
        },
        _ => panic!("invalid trap cause: {:#x}", scause),
    }
}

#[inline]
pub fn enable_irqs() {
    unsafe { sstatus::set_sie() }
}

pub(super) fn init_percpu() {
    unsafe {
        sie::set_ssoft();
        sie::set_stimer();
        sie::set_sext();
    }
}
