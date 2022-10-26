/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

extern crate alloc;

use core::cmp::min;
use core::alloc::*;
use super::defines::*;
use crate::errors::Error;

/*
 * PTE format:
 * | XLEN-1  10 | 9             8 | 7 | 6 | 5 | 4 | 3 | 2 | 1 | 0
 *       PFN      reserved for SW   D   A   G   U   X   W   R   V
 */

const _PAGE_PFN_SHIFT: usize = 10;

const _PAGE_PRESENT : usize = 1 << 0;     /* Valid */
const _PAGE_READ    : usize = 1 << 1;     /* Readable */
const _PAGE_WRITE   : usize = 1 << 2;     /* Writable */
const _PAGE_EXEC    : usize = 1 << 3;     /* Executable */
const _PAGE_USER    : usize = 1 << 4;     /* User */
const _PAGE_GLOBAL  : usize = 1 << 5;     /* Global */
const _PAGE_ACCESSED: usize = 1 << 6;     /* Accessed (set by hardware) */
const _PAGE_DIRTY   : usize = 1 << 7;     /* Dirty (set by hardware)*/

/*
 * when all of R/W/X are zero, the PTE is a pointer to the next level
 * of the page table; otherwise, it is a leaf PTE.
 */
const _PAGE_LEAF: usize = _PAGE_READ | _PAGE_WRITE | _PAGE_EXEC;

const PAGE_TABLE: usize = _PAGE_PRESENT;

pub const PAGE_KERNEL: usize =
    _PAGE_PRESENT | _PAGE_READ | _PAGE_WRITE |
    _PAGE_GLOBAL | _PAGE_ACCESSED | _PAGE_DIRTY;

pub const PAGE_KERNEL_EXEC : usize = PAGE_KERNEL | _PAGE_EXEC;

const PAGE_TABLE_ENTRIES: usize = 1 << (PAGE_SHIFT - 3);

#[repr(C, align(4096))]
pub struct PageTable([usize; PAGE_TABLE_ENTRIES]);

impl PageTable {
    const ZERO: Self = Self([0usize; PAGE_TABLE_ENTRIES]);

    fn clear(&mut self) {
        self.0 = [0; PAGE_TABLE_ENTRIES];
    }

    fn mk_item(&mut self, index: usize, pfn: usize, prot: usize) {
        self.0[index] = (pfn << _PAGE_PFN_SHIFT) | prot;
    }

    fn item_present(&self, index: usize) -> bool {
        (self.0[index] & _PAGE_PRESENT) == _PAGE_PRESENT
    }

    fn item_leaf(&self, index: usize) -> bool {
        self.item_present(index) && ((self.0[index] & _PAGE_LEAF) != 0)
    }

    fn item_descend(&self, index: usize) -> *mut PageTable {
        ((self.0[index] >> _PAGE_PFN_SHIFT) << PAGE_SHIFT) as *mut PageTable
    }
}

pub static mut SWAPPER_PG_DIR: PageTable = PageTable::ZERO;
pub static mut SWAPPER_SATP: usize = 0;

const MMU_LEVELS: usize =
    (KERNEL_ASPACE_BITS - PAGE_SHIFT) / (PAGE_SHIFT - 3);

macro_rules! LEVEL_SHIFT {
    ($level: expr) => {
        ((MMU_LEVELS - ($level)) * (PAGE_SHIFT - 3) + 3)
    }
}

macro_rules! LEVEL_SIZE {
    ($level: expr) => {
        1 << LEVEL_SHIFT!($level)
    }
}

macro_rules! LEVEL_MASK {
    ($level: expr) => {
        !(LEVEL_SIZE!($level) - 1)
    }
}

macro_rules! LEVEL_PA_TO_PFN {
    ($pa: expr, $level: expr) => {
        (($pa) >> LEVEL_SHIFT!($level))
    }
}

macro_rules! PA_TO_PFN {
    ($pa: expr) => {
        (($pa) >> PAGE_SHIFT)
    }
}

fn vaddr_to_index(addr: usize, level: usize) -> usize {
    (addr >> LEVEL_SHIFT!(level)) & (PAGE_TABLE_ENTRIES - 1)
}

fn aligned_in_level(addr: usize, level: usize) -> bool {
    (addr & !(LEVEL_MASK!(level))) == 0
}

fn _boot_map<F1>(table: &mut PageTable, level: usize,
                 vaddr: usize, paddr: usize, len: usize,
                 prot: usize,
                 phys_to_virt: &F1) -> Result<Error, Error>
where F1: Fn(usize) -> usize
{
    let mut off = 0;
    while off < len {
        let index = vaddr_to_index(vaddr + off, level);
        if level == (MMU_LEVELS-1) {
            /* generate a standard leaf mapping */
            table.mk_item(index, PA_TO_PFN!(paddr + off), prot);

            off += PAGE_SIZE;
            continue;
        }
        if !table.item_present(index) {
            if (level != 0) &&
                aligned_in_level(vaddr+off, level) &&
                aligned_in_level(paddr+off, level) &&
                ((len - off) >= LEVEL_SIZE!(level)) {
                /* set up a large leaf at this level */
                table.mk_item(index,
                              LEVEL_PA_TO_PFN!(paddr + off, level),
                              prot);

                off += LEVEL_SIZE!(level);
                continue;
            }

            let layout = Layout::from_size_align(4096, 4096).unwrap();
            unsafe {
                let pa: usize = alloc::alloc::alloc(layout) as usize;
                table.mk_item(index, PA_TO_PFN!(pa), PAGE_TABLE);
            }
        }
        if table.item_leaf(index) {
            /* not legal as a leaf at this level */
            return Err(Error::BadState);
        }

        let lower_table_ptr = table.item_descend(index);
        let lower_len = min(LEVEL_SIZE!(level), len-off);
        unsafe {
            (*lower_table_ptr).clear();
            _boot_map(&mut (*lower_table_ptr), level+1,
                      vaddr+off, paddr+off, lower_len,
                      prot, phys_to_virt)?;
        }

        off += LEVEL_SIZE!(level);
    }

    return Ok(Error::OK);
}

pub fn riscv64_boot_map(table: &mut PageTable,
                        vaddr: usize, paddr: usize, len: usize,
                        prot: usize) -> Result<Error, Error> {
    let phys_to_virt = |pa: usize| { pa };

    /* Loop through the virtual range and map each physical page,
     * using the largest page size supported.
     * Allocates necessar page tables along the way. */
    _boot_map(table, 0, vaddr, paddr, len, prot, &mut &phys_to_virt)
}

pub unsafe fn riscv64_setup_mmu_mode()
{
    let ptr = (&SWAPPER_PG_DIR) as *const PageTable;
    let pfn = (ptr as usize) >> PAGE_SHIFT;

    let mode = match MMU_LEVELS {
        4 => SATP_MODE_48,
        3 => SATP_MODE_39,
        _ => 0,
    };

    SWAPPER_SATP = mode | pfn;
}
