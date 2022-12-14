/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use core::slice;
use crate::{
    BootContext, dprint, CRITICAL, INFO, WARN,
    PAGE_SIZE, ROUNDUP_PAGE_SIZE,
    kernel_base_phys, kernel_size,
};
use crate::errors::ErrNO;
use crate::vm::bootreserve::boot_reserve_init;
use crate::vm::physmap::paddr_to_physmap;
use crate::vm::pmm::{
    MAX_ARENAS, ArenaInfo, pmm_add_arena, pmm_alloc_range,
};
use crate::vm::page::vm_page_t;
use alloc::vec::Vec;
use device_tree::DeviceTree;
use crate::boot::image::*;
use crate::arch::periphmap::add_periph_range;
use crate::lib::list::List;

type ZBIMemRangeVec = Vec<ZBIMemRange>;

const OF_ROOT_NODE_SIZE_CELLS_DEFAULT: u32 = 1;
const OF_ROOT_NODE_ADDR_CELLS_DEFAULT: u32 = 1;

fn process_phys_handoff(ctx: &mut BootContext)
    -> Result<Vec<ArenaInfo>, ErrNO> {

    /* discover memory ranges */
    let mut mem_config = parse_dtb(ctx)?;

    init_mem_config_arch(&mut mem_config);

    process_mem_ranges(ctx, mem_config)
}

fn boot_reserve_wire(ctx: &mut BootContext) -> Result<(), ErrNO> {
    for r in &(ctx.reserve_ranges) {
        dprint!(INFO, "PMM: boot reserve marking WIRED [{:x}, {:x}]\n",
                r.pa, r.pa + r.len - 1);

        let pages = ROUNDUP_PAGE_SIZE!(r.len) / PAGE_SIZE;
        let mut alloc_page_list = List::<vm_page_t>::new();
        pmm_alloc_range(r.pa, pages,
                        &mut ctx.pmm_node,
                        &mut alloc_page_list)?;
    }

    Ok(())

    /*
    if (list_is_empty(&reserved_page_list)) {
      list_move(&alloc_page_list, &reserved_page_list);
    } else {
      list_splice_after(&alloc_page_list, list_peek_tail(&reserved_page_list));
    }
  */

  // mark all of the pages we allocated as WIRED
    /*
  vm_page_t* p;
  list_for_every_entry (&reserved_page_list, p, vm_page_t, queue_node) {
    p->set_state(vm_page_state::WIRED);
  }
  */
}

fn process_mem_ranges(ctx: &mut BootContext,
                      mem_config: Vec<ZBIMemRange>)
    -> Result<Vec<ArenaInfo>, ErrNO> {

    let mut mem_arenas = Vec::<ArenaInfo>::with_capacity(MAX_ARENAS);

    for range in mem_config {
        match &(range.mtype) {
            ZBIMemRangeType::RAM => {
                dprint!(INFO, "ZBI: mem arena {:x} - {:x}\n",
                        range.paddr, range.length);

                if mem_arenas.len() >= MAX_ARENAS {
                    dprint!(CRITICAL,
                            "ZBI: too many memory arenas,
                             dropping additional\n");
                    break;
                }
                mem_arenas.push(
                    ArenaInfo::new("ram", 0, range.paddr, range.length)
                );
            },
            ZBIMemRangeType::PERIPHERAL => {
                dprint!(INFO, "ZBI: peripheral range {:x} - {:x}\n",
                        range.paddr, range.length);
                add_periph_range(ctx, range.paddr, range.length)?;
            },
            ZBIMemRangeType::_RESERVED => {
                dprint!(WARN, "FIND RESERVED Memory Range {:x} {:x}!\n",
                        range.paddr, range.length);
            }
        }
    }

    Ok(mem_arenas)
}

fn init_mem_config_arch(config: &mut Vec<ZBIMemRange>) {
    config.push(
        ZBIMemRange::new(ZBIMemRangeType::PERIPHERAL, 0, 0x40000000)
    );
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

fn early_init_dt_add_memory_arch(config: &mut Vec<ZBIMemRange>,
                                 base: usize, size: usize) {
    config.push(ZBIMemRange::new(ZBIMemRangeType::RAM, base, size));
}

/*
 * early_init_dt_scan_memory - Look for and parse memory nodes
 */
fn early_init_dt_scan_memory(dt: &DeviceTree,
                             addr_cells: u32, size_cells: u32)
    -> Result<ZBIMemRangeVec, ErrNO> {

    let root = dt.find("/").ok_or_else(|| ErrNO::BadDTB)?;

    let mut mem_config =
        Vec::<ZBIMemRange>::with_capacity(MAX_ZBI_MEM_RANGES);

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

            early_init_dt_add_memory_arch(&mut mem_config, base, size);
        }
    }

    Ok(mem_config)
}

fn early_init_dt_scan(dt: &DeviceTree)
    -> Result<ZBIMemRangeVec, ErrNO> {

    /* Initialize {size,address}-cells info */
    let (addr_cells, size_cells) = early_init_dt_scan_root(dt);

    /* Retrieve various information from the /chosen node */
    let cmdline = early_init_dt_scan_chosen(dt);
    dprint!(INFO, "command line = {}\n", cmdline);

    /* Setup memory, calling early_init_dt_add_memory_arch */
    early_init_dt_scan_memory(dt, addr_cells, size_cells)
}

pub fn parse_dtb(ctx: &mut BootContext)
    -> Result<ZBIMemRangeVec, ErrNO> {

    /* Early scan of device tree from init memory */
    let dtb_va = paddr_to_physmap(ctx.dtb_pa);
    dprint!(CRITICAL, "HartID {:x} DTB 0x{:x} -> 0x{:x}\n",
            ctx.hartid, ctx.dtb_pa, dtb_va);

    let dt = early_init_dt_load(dtb_va)?;

    early_init_dt_scan(&dt)
}

pub fn platform_early_init(ctx: &mut BootContext)
    -> Result<(), ErrNO> {
    /* initialize the boot memory reservation system */
    boot_reserve_init(kernel_base_phys(), kernel_size(),
                      &mut ctx.reserve_ranges)?;

    let mut mem_arenas = process_phys_handoff(ctx)?;

    /* find memory ranges to use if one is found. */
    loop {
        if let Some(a) = mem_arenas.pop() {
            pmm_add_arena(a, &mut (ctx.pmm_node),
                          &(ctx.reserve_ranges))?;
        } else {
            break;
        }
    }

    for range in &(ctx.periph_ranges) {
        dprint!(INFO, "PERIPH: {:x} -> {:x}, {:x}\n",
                range.base_phys, range.base_virt, range.length);
    }

    dprint!(INFO, "platform early init ok!\n");

    /* tell the boot allocator to mark ranges we've reserved. */
    boot_reserve_wire(ctx)
}
