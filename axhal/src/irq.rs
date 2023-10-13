use lazy_init::LazyInit;
use handler_table::HandlerTable;

/// The maximum number of IRQs.
pub const MAX_IRQ_COUNT: usize = 1024;

/// The type if an IRQ handler.
pub type IrqHandler = handler_table::Handler;

static IRQ_HANDLER_TABLE: HandlerTable<MAX_IRQ_COUNT> = HandlerTable::new();

static TIMER_HANDLER: LazyInit<IrqHandler> = LazyInit::new();

macro_rules! with_cause {
    ($cause: expr, @TIMER => $timer_op: expr, @EXT => $ext_op: expr $(,)?) => {
        match $cause {
            S_TIMER => $timer_op,
            S_EXT => $ext_op,
            _ => panic!("invalid trap cause: {:#x}", $cause),
        }
    };
}

/// Dispatches the IRQ.
///
/// This function is called by the common interrupt handler. It looks
/// up in the IRQ handler table and calls the corresponding handler. If
/// necessary, it also acknowledges the interrupt controller after handling.
pub fn dispatch_irq(scause: usize) {
    with_cause!(
        scause,
        @TIMER => {
            trace!("IRQ: timer");
            TIMER_HANDLER();
        },
        @EXT => crate::irq::dispatch_irq_common(0), // TODO: get IRQ number from PLIC
    );
}

/// Platform-independent IRQ dispatching.
#[allow(dead_code)]
pub(crate) fn dispatch_irq_common(irq_num: usize) {
    trace!("IRQ {}", irq_num);
    if !IRQ_HANDLER_TABLE.handle(irq_num) {
        warn!("Unhandled IRQ {}", irq_num);
    }
}
