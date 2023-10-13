#![no_std]

#[macro_use]
extern crate axlog;

extern crate alloc;

mod lang_items;
mod trap;

use core::str;
use alloc::string::String;
use alloc::vec::Vec;
use axdtb::util::SliceRead;
use axhal::paging::PageTable;
use axsync::BootOnceCell;
use axhal::mem::phys_to_virt;

extern "C" {
    fn _skernel();
    fn main();
}

struct LogIfImpl;

#[crate_interface::impl_interface]
impl axlog::LogIf for LogIfImpl {
    fn console_write_str(s: &str) {
        axhal::console::write_bytes(s.as_bytes());
    }

    fn current_time() -> core::time::Duration {
        axhal::time::current_time()
    }
}

#[no_mangle]
pub extern "C" fn rust_main(hartid: usize, dtb: usize) -> ! {
    let log_level = option_env!("AX_LOG").unwrap_or("");
    ax_println!("\nArceOS is starting... [{}]\n", log_level);

    axlog::init();
    axlog::set_max_level(log_level);
    info!("Logging is enabled.");
    info!("Primary CPU {} started, dtb = {:#x}.", hartid, dtb);

    #[cfg(feature = "alloc")]
    {
        // For Riscv64 memory layout, we reserve 2M memory space for SBI.
        // Now SBI just occupies about 194K, so reserve 1M for it and
        // requisition another 1M for early heap.
        axalloc::early_init(_skernel as usize - 0x100000, 0x100000);
    }

    // Parse fdt for early memory info
    let dtb_info = match parse_dtb(dtb.into()) {
        Ok(info) => info,
        Err(err) => panic!("Bad dtb {:?}", err),
    };

    info!("Memory: {:#x}, size: {:#x}", dtb_info.memory_addr, dtb_info.memory_size);
    info!("Virtio_mmio[{}]:", dtb_info.mmio_regions.len());
    for r in &dtb_info.mmio_regions {
        info!("\t{:#x}, size: {:#x}", r.0, r.1);
    }

    #[cfg(feature = "paging")]
    {
        info!("Initialize kernel page table...");
        remap_kernel_memory(&dtb_info).expect("remap kernel memoy failed");
    }

    info!("Heap total: {}K, avail: {}K, used: {}K ({} pages)",
          axalloc::total_bytes()/1024,
          axalloc::available_bytes()/1024,
          axalloc::used_bytes()/1024,
          axalloc::used_pages());

    #[cfg(feature = "alloc")]
    {
        allocator_final_init(dtb_info.memory_addr + dtb_info.memory_size);
    }

    info!("Initialize platform devices...");
    axhal::platform_init();

    axtask::init_scheduler();

    info!("Initialize interrupt handlers...");
    init_interrupt();

    unsafe {
        main();
    }

    debug!("main task exited: exit_code={}", 0);
    axhal::misc::terminate();
}

fn init_interrupt() {
    use axhal::irq::TIMER_IRQ_NUM;

    // Setup timer interrupt handler
    const PERIODIC_INTERVAL_NANOS: u64 =
        axhal::time::NANOS_PER_SEC / axconfig::TICKS_PER_SEC as u64;

    static mut NEXT_DEADLINE: u64 = 0;

    fn update_timer() {
        let now_ns = axhal::time::current_time_nanos();
        // Safety: we have disabled preemption in IRQ handler.
        let mut deadline = unsafe { NEXT_DEADLINE };
        if now_ns >= deadline {
            deadline = now_ns + PERIODIC_INTERVAL_NANOS;
        }
        unsafe { NEXT_DEADLINE = deadline + PERIODIC_INTERVAL_NANOS };
        trace!("now {} deadline {}", now_ns, deadline);
        axhal::time::set_oneshot_timer(deadline);
    }

    axhal::irq::register_handler(TIMER_IRQ_NUM, || {
        update_timer();
        debug!("On timer tick!");
        //#[cfg(feature = "multitask")]
        //axtask::on_timer_tick();
    });

    // Enable IRQs before starting app
    axhal::irq::enable_irqs();
}

#[cfg(feature = "alloc")]
fn allocator_final_init(memory_size: usize) {
    use axhal::mem::{free_regions, MemRegionFlags};

    info!("Finalize global memory allocator...");

    let mut max_region_size = 0;
    let mut max_region_paddr = 0.into();
    for r in free_regions(memory_size) {
        if r.flags.contains(MemRegionFlags::FREE) && r.size > max_region_size {
            max_region_size = r.size;
            max_region_paddr = r.paddr;
        }
    }
    for r in free_regions(memory_size) {
        if r.flags.contains(MemRegionFlags::FREE) && r.paddr == max_region_paddr {
            axalloc::final_init(phys_to_virt(r.paddr).as_usize(), r.size);
            break;
        }
    }
    for r in free_regions(memory_size) {
        if r.flags.contains(MemRegionFlags::FREE) && r.paddr != max_region_paddr {
            axalloc::final_add_memory(phys_to_virt(r.paddr).as_usize(), r.size)
                .expect("add heap memory region failed");
        }
    }
}

#[cfg(feature = "paging")]
fn remap_kernel_memory(dtb: &DtbInfo) -> Result<(), axhal::paging::PagingError> {
    use axhal::mem::{kernel_image_regions, free_regions};
    use axhal::mem::{MemRegion, MemRegionFlags};

    let mmio_regions = dtb.mmio_regions.iter().map(|reg| MemRegion {
        paddr: reg.0.into(),
        size: reg.1,
        flags: MemRegionFlags::RESERVED
            | MemRegionFlags::DEVICE
            | MemRegionFlags::READ
            | MemRegionFlags::WRITE,
        name: "mmio",
    });

    let regions = kernel_image_regions()
        .chain(free_regions(dtb.memory_addr + dtb.memory_size))
        .chain(mmio_regions);

    let mut kernel_page_table = PageTable::try_new()?;
    for r in regions {
        kernel_page_table.map_region(
            phys_to_virt(r.paddr),
            r.paddr,
            r.size,
            r.flags.into(),
            true,
        )?;
    }

    static KERNEL_PAGE_TABLE: BootOnceCell<PageTable> =
        unsafe { BootOnceCell::new() };

    KERNEL_PAGE_TABLE.init(kernel_page_table);

    unsafe {
        axhal::paging::write_page_table_root(KERNEL_PAGE_TABLE.get().root_paddr())
    };

    Ok(())
}

struct DtbInfo {
    memory_addr: usize,
    memory_size: usize,
    mmio_regions: Vec<(usize, usize)>,
}

fn parse_dtb(dtb_pa: axhal::mem::PhysAddr) -> axdtb::DeviceTreeResult<DtbInfo> {
    let dtb_va = phys_to_virt(dtb_pa);
    debug!("dtb: {:#x} => {:#x}", dtb_pa, dtb_va);

    let mut memory_addr = 0;
    let mut memory_size = 0;
    let mut mmio_regions = Vec::new();

    let mut cb = |name: String, addr_cells: usize, size_cells: usize, props: Vec<(String, Vec<u8>)>| {
        debug!("{}: cells {}, {}", name, addr_cells, size_cells);
        let mut is_memory = false;
        let mut is_mmio = false;
        let mut reg = None;
        for prop in props {
            match prop.0.as_str() {
                "device_type" => {
                    is_memory = str::from_utf8(&(prop.1))
                        .map_or_else(|_| false, |v| v == "memory\0");
                },
                "compatible" => {
                    is_mmio = str::from_utf8(&(prop.1))
                        .map_or_else(|_| false, |v| v == "virtio,mmio\0");
                },
                "reg" => {
                    reg = Some(prop.1);
                },
                _ => (),
            }
        }
        if is_memory {
            assert!(addr_cells == 2);
            assert!(size_cells == 2);
            if let Some(ref reg) = reg {
                memory_addr = reg.as_slice().read_be_u64(0).unwrap() as usize;
                memory_size = reg.as_slice().read_be_u64(8).unwrap() as usize;
            }
        }
        if is_mmio {
            assert!(addr_cells == 2);
            assert!(size_cells == 2);
            if let Some(ref reg) = reg {
                let addr = reg.as_slice().read_be_u64(0).unwrap() as usize;
                let size = reg.as_slice().read_be_u64(8).unwrap() as usize;
                mmio_regions.push((addr, size));
            }
        }
    };

    let dt = axdtb::DeviceTree::init(dtb_va.into())?;
    dt.parse(dt.off_struct, 0, 0, &mut cb)?;

    Ok(DtbInfo {
        memory_addr,
        memory_size,
        mmio_regions,
    })
}
