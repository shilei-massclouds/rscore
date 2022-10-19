/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use super::defines::*;
use crate::vm::bootalloc::*;

/*
 * PTE format:
 * | XLEN-1  10 | 9             8 | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0
 *       PFN      reserved for SW   D   A   G   U   X   W   R   V
 */

const _PAGE_PRESENT : u64 = 1 << 0;     /* Valid */
const _PAGE_READ    : u64 = 1 << 1;     /* Readable */
const _PAGE_WRITE   : u64 = 1 << 2;     /* Writable */
const _PAGE_EXEC    : u64 = 1 << 3;     /* Executable */
const _PAGE_USER    : u64 = 1 << 4;     /* User */
const _PAGE_GLOBAL  : u64 = 1 << 5;     /* Global */
const _PAGE_ACCESSED: u64 = 1 << 6;     /* Accessed (set by hardware) */
const _PAGE_DIRTY   : u64 = 1 << 7;     /* Dirty (set by hardware)*/

const PAGE_TABLE: u64 = _PAGE_PRESENT;

const PAGE_KERNEL: u64 =
    _PAGE_PRESENT | _PAGE_READ | _PAGE_WRITE |
    _PAGE_GLOBAL | _PAGE_ACCESSED | _PAGE_DIRTY;

const PAGE_KERNEL_EXEC : u64 = PAGE_KERNEL | _PAGE_EXEC;

const PAGE_TABLE_ENTRIES: usize = 1 << (PAGE_SHIFT - 3);

#[repr(align(4096))]
pub struct PageTable([usize; PAGE_TABLE_ENTRIES]);

impl PageTable {
    const ZERO: Self = Self([0usize; PAGE_TABLE_ENTRIES]);
}

static TRAMPOLINE_PG_DIR: PageTable = PageTable::ZERO;
pub static SWAPPER_PG_DIR: PageTable = PageTable::ZERO;

macro_rules! MMU_LX_X {
    ($page_shift: expr, $level: expr) => {
        ((4 - ($level)) * (($page_shift) - 3) + 3)
    }
}

fn vaddr_to_index(addr: usize, level: usize) -> usize {
    (addr >> MMU_LX_X!(PAGE_SHIFT, level)) & (PAGE_TABLE_ENTRIES - 1)
}

fn _boot_map<F0, F1>(table0: &PageTable,
                     vaddr: usize, paddr: usize, len: usize,
                     exflag: usize,
                     alloc_func: F0, phys_to_virt: F1)
where F0: FnMut() -> usize,
      F1: Fn(usize) -> usize
{
    /* Loop through the virtual range and map each physical page,
     * using the largest page size supported.
     * Allocates necessar page tables along the way. */
    let mut off = 0;
    while (off < len) {
        /* make sure the level 1 pointer is valid */
        let index0 = vaddr_to_index(vaddr + off, 0);
        if table0.item_leaf(index0) {
            // not legal as a leaf at this level
            return ZX_ERR_BAD_STATE;
        }
        if !table0.item_valid(index0) {
            let pa: usize = alloc_func();
            table0.mk_item(pa, PAGE_KERNEL|exflag);
        }

        let table1 = table0.next_level(index0);
    }
}

pub fn riscv64_boot_map(bootalloc: &mut BootAlloc,
                        table: &PageTable,
                        vaddr: usize, paddr: usize, len: usize,
                        exflag: usize) {
    /* The following helper routines assume that code is running
     * in physical addressing mode (mmu off).
     * Any physical addresses calculated are assumed to be
     * the same as virtual */
    let alloc = || {
        /* allocate a page out of the boot allocator,
         * asking for a physical address */
        bootalloc.alloc_page_phys()
    };

    let phys_to_virt = |pa: usize| { pa };

    _boot_map(table, vaddr, paddr, len, exflag, alloc, phys_to_virt);
}
