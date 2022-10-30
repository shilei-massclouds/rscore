/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use crate::{
    BootContext, dprint, CRITICAL, INFO, WARN,
    ZBIMemRange, ZBI_MEM_RANGE_PERIPHERAL,
};
use crate::errors::ErrNO;
use crate::vm::bootreserve::boot_reserve_init;
use core::slice;
use device_tree::DeviceTree;

const OF_ROOT_NODE_SIZE_CELLS_DEFAULT: u32 = 1;
const OF_ROOT_NODE_ADDR_CELLS_DEFAULT: u32 = 1;

/* all of the configured memory arenas */
pub const NUM_ARENAS: usize = 16;

#[derive(Copy, Clone)]
pub struct ArenaInfo {
    name: [char; 16],

    flags: usize,

    base: usize,
    size: usize,
}

pub fn platform_early_init(ctx: &mut BootContext) -> Result<(), ErrNO> {
    /* initialize the boot memory reservation system */
    boot_reserve_init(ctx)?;

    /* discover memory ranges */
    parse_dtb(ctx)?;

    init_mem_config_arch(ctx)?;

    /* find memory ranges to use if one is found. */
    /*
    for (size_t i = 0; i < arena_count; i++) {
        if (!have_limit || status != ZX_OK) {
            pmm_add_arena(&mem_arena[i]);
        }
    }
    */

    dprint!(INFO, "platform early init ok!\n");
    Ok(())
}

fn init_mem_config_arch(ctx: &mut BootContext) -> Result<(), ErrNO> {
    ctx.mem_config.push(ZBIMemRange {
        mtype: ZBI_MEM_RANGE_PERIPHERAL,
        paddr: 0,
        length: 0x40000000,
        reserved: 0,
    });

    Ok(())
}

fn process_mem_ranges(ctx: &BootContext) -> Result<(), ErrNO> {
    Ok(())
}

fn fdt_get_u32(dtb_va: usize, offset: usize) -> u32 {
    let ptr = (dtb_va + offset) as *const u32;
    unsafe {
        u32::from_be(*ptr)
    }
}

const FDT_MAGIC: u32 = 0xd00dfeed;

const FDT_MAGIC_OFFSET: usize = 0;
const FDT_TOTALSIZE_OFFSET: usize = 4;

fn early_init_dt_verify(dtb_va: usize) -> Result<(), ErrNO> {
    if dtb_va == 0 {
        dprint!(CRITICAL, "No DTB passed to the kernel\n");
        return Err(ErrNO::NoDTB);
    }

    /* check device tree validity */
    if fdt_get_u32(dtb_va, FDT_MAGIC_OFFSET) != FDT_MAGIC {
        dprint!(CRITICAL, "Bad DTB passed to the kernel\n");
        return Err(ErrNO::BadDTB);
    }

    Ok(())
}

fn early_init_dt_load(dtb_va: usize) -> Result<DeviceTree, ErrNO> {

    early_init_dt_verify(dtb_va)?;

    let totalsize = fdt_get_u32(dtb_va, FDT_TOTALSIZE_OFFSET);
    unsafe {
        let buf = slice::from_raw_parts_mut(dtb_va as *mut u8,
                                            totalsize as usize);
        DeviceTree::load(buf).or_else(|e| {
            dprint!(CRITICAL, "Can't load dtb: {:?}\n", e);
            Err(ErrNO::BadDTB)
        })
    }
}

/*
 * early_init_dt_scan_root - fetch the top level address and size cells
 */
fn early_init_dt_scan_root(dt: &DeviceTree) -> (u32, u32) {
    let root = match dt.find("/") {
        Some(node) => { node },
        None => {
            dprint!(CRITICAL, "Can't find root of this dtb!\n");
            return (OF_ROOT_NODE_ADDR_CELLS_DEFAULT,
                    OF_ROOT_NODE_SIZE_CELLS_DEFAULT);
        }
    };

    let addr_cells = root.prop_u32("#address-cells")
        .unwrap_or_else(|_| OF_ROOT_NODE_ADDR_CELLS_DEFAULT);
    dprint!(INFO, "dt_root_addr_cells = 0x{:x}\n", addr_cells);

    let size_cells = root.prop_u32("#size-cells")
        .unwrap_or_else(|_| OF_ROOT_NODE_SIZE_CELLS_DEFAULT);
    dprint!(INFO, "dt_root_size_cells = 0x{:x}\n", size_cells);

    (addr_cells, size_cells)
}

fn early_init_dt_scan_chosen(dt: &DeviceTree) -> &str {
    let chosen = match dt.find("/chosen") {
        Some(node) => { node },
        None => {
            if let Some(node) = dt.find("/chosen@0") {
                node
            } else {
                dprint!(WARN, "No chosen node found!\n");
                return "";
            }
        }
    };

    /* Retrieve command line */
    if let Ok(s) = chosen.prop_str("bootargs") {
        return s;
    }

    ""
}

fn early_init_dt_add_memory_arch(base: usize, size: usize) {
    dprint!(INFO, " - 0x{:x}, 0x{:x}\n", base, size);
}

/*
 * early_init_dt_scan_memory - Look for and parse memory nodes
 */
fn early_init_dt_scan_memory(dt: &DeviceTree,
                             addr_cells: u32, size_cells: u32)
    -> Result<(), ErrNO> {

    let root = dt.find("/").ok_or_else(|| ErrNO::BadDTB)?;

    for child in &root.children {
        /* We are scanning "memory" nodes only */
        if let Ok(t) = child.prop_str("device_type") {
            if t != "memory" {
                continue;
            }
        } else {
            continue;
        }

        let mut pos = 0;
        let reg_len = child.prop_len("reg");
        while pos < reg_len {
            let base = if addr_cells == 2 {
                child.prop_u64_at("reg", pos).unwrap() as usize
            } else {
                child.prop_u32_at("reg", pos).unwrap() as usize
            };
            pos += (addr_cells << 2) as usize;

            let size = if size_cells == 2 {
                child.prop_u64_at("reg", pos).unwrap() as usize
            } else {
                child.prop_u32_at("reg", pos).unwrap() as usize
            };
            pos += (size_cells << 2) as usize;

            if size == 0 {
                continue;
            }
            dprint!(INFO, " - 0x{:x}, 0x{:x}\n", base, size);

            early_init_dt_add_memory_arch(base, size);
        }
    }

    Ok(())
}

fn early_init_dt_scan(dt: &DeviceTree) -> Result<(), ErrNO> {
    /* Initialize {size,address}-cells info */
    let (addr_cells, size_cells) = early_init_dt_scan_root(dt);

    /* Retrieve various information from the /chosen node */
    let cmdline = early_init_dt_scan_chosen(dt);
    dprint!(INFO, "command line = {}\n", cmdline);

    /* Setup memory, calling early_init_dt_add_memory_arch */
    early_init_dt_scan_memory(dt, addr_cells, size_cells)
}

pub fn parse_dtb(ctx: &mut BootContext) -> Result<(), ErrNO> {
    /* Early scan of device tree from init memory */
    let dtb_va = ctx.pa_to_va(ctx.dtb_pa);
    dprint!(CRITICAL, "HartID {:x} DTB 0x{:x} -> 0x{:x}\n",
            ctx.hartid, ctx.dtb_pa, dtb_va);

    let dt = early_init_dt_load(dtb_va)?;

    early_init_dt_scan(&dt)
}
