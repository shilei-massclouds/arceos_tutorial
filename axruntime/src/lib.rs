#![no_std]

pub use axhal::ax_println as println;

#[macro_use]
extern crate axlog;
extern crate alloc;

use core::str;
use alloc::string::String;
use alloc::vec::Vec;
use axdtb::SliceRead;
use axconfig::phys_to_virt;

#[no_mangle]
pub extern "C" fn rust_main(hartid: usize, dtb: usize) -> ! {
    extern "C" {
        fn _skernel();
        fn main();
    }

    println!("\nArceOS is starting ...");

    // We reserve 2M memory range [0x80000000, 0x80200000) for SBI,
    // but it only occupies ~194K. Split this range in half,
    // requisition the higher part(1M) for early heap.
    axalloc::early_init(_skernel as usize - 0x100000, 0x100000);

    axlog::init();
    axlog::set_max_level(option_env!("LOG").unwrap_or("")); // no effect if set `log-level-*` features
    info!("Logging is enabled.");
    info!("Primary CPU {} started, dtb = {:#x}.", hartid, dtb);

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

    unsafe { main(); }
    axhal::terminate();
}

struct DtbInfo {
    memory_addr: usize,
    memory_size: usize,
    mmio_regions: Vec<(usize, usize)>,
}

fn parse_dtb(dtb_pa: usize) -> axdtb::DeviceTreeResult<DtbInfo> {
    let dtb_va = phys_to_virt(dtb_pa);

    let mut memory_addr = 0;
    let mut memory_size = 0;
    let mut mmio_regions = Vec::new();

    let mut cb = |_name: String, addr_cells: usize, size_cells: usize, props: Vec<(String, Vec<u8>)>| {
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
