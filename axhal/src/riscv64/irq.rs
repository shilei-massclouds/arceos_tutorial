use axsync::BootOnceCell;
use handler_table::HandlerTable;

pub const MAX_IRQ_COUNT: usize = 1024;
pub(super) const INTC_IRQ_BASE: usize = 1 << (usize::BITS - 1);

#[allow(unused)]
pub(super) const S_SOFT: usize = INTC_IRQ_BASE + 1;
pub(super) const S_TIMER: usize = INTC_IRQ_BASE + 5;
pub(super) const S_EXT: usize = INTC_IRQ_BASE + 9;

pub type IrqHandler = handler_table::Handler;

static IRQ_HANDLER_TABLE: HandlerTable<MAX_IRQ_COUNT> = HandlerTable::new();

static TIMER_HANDLER: BootOnceCell<IrqHandler> = unsafe {
    BootOnceCell::new()
};

pub fn dispatch_irq(scause: usize) {
    match scause {
        S_TIMER => {
            log::trace!("IRQ: timer");
            TIMER_HANDLER.get();
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
