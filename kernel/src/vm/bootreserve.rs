/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use alloc::vec::Vec;
use crate::{
    dprint, INFO, paddr_t,
};
use crate::errors::ErrNO;

pub const MAX_RESERVES: usize = 64;

#[derive(Default)]
pub struct BootReserveRange {
    pub pa: usize,
    pub len: usize,
}

pub fn boot_reserve_init(pa: paddr_t, len: usize,
                         ranges: &mut Vec<BootReserveRange>)
    -> Result<(), ErrNO> {
    /* add the kernel to the boot reserve list */
    boot_reserve_add_range(pa, len, ranges)
}

/* given two offset/length pairs, determine if they overlap at all */
#[inline]
fn intersects(offset1: usize, len1: usize,
              offset2: usize, len2: usize) -> bool {
    /* Can't overlap a zero-length region. */
    if len1 == 0 || len2 == 0 {
        return false;
    }

    if offset1 <= offset2 {
        /* doesn't intersect, 1 is completely below 2 */
        if offset1 + len1 <= offset2 {
            return false;
        }
    } else if offset1 >= offset2 + len2 {
        /* 1 is completely above 2 */
        return false;
    }

    true
}

fn boot_reserve_add_range(pa: usize, len: usize,
                          ranges: &mut Vec<BootReserveRange>)
    -> Result<(), ErrNO> {

    dprint!(INFO, "PMM: boot reserve add [0x{:x}, 0x{:x}]\n",
            pa, pa + len - 1);

    let end = pa + len - 1;

    /* insert into the list, sorted */
    let mut i = 0;
    while i < ranges.len() {
        if intersects(ranges[i].pa, ranges[i].len, pa, len) {
            return Err(ErrNO::BadRange);
        }

        if ranges[i].pa > end {
            break;
        }

        i += 1;
    }

    let range = BootReserveRange{pa: pa, len: len};
    ranges.insert(i, range);

    dprint!(INFO, "Boot reserve #range {}\n", ranges.len());
    Ok(())
}

fn upper_align(range_pa: paddr_t, range_len: usize,
               alloc_len: usize) -> paddr_t {
    range_pa + range_len - alloc_len
}

pub fn boot_reserve_range_search(range_pa: paddr_t,
                                 range_len: usize,
                                 alloc_len: usize,
                                 ranges: &Vec<BootReserveRange>,
                                 alloc_range: &mut BootReserveRange)
    -> Result<(), ErrNO> {

    dprint!(INFO, "range pa {:x} len {:x} alloc_len {:x}\n",
            range_pa, range_len, alloc_len);

    let mut alloc_pa = upper_align(range_pa, range_len, alloc_len);

    /* see if it intersects any reserved range */
    dprint!(INFO, "trying alloc range {:x} len {:x}\n",
            alloc_pa, alloc_len);

    'retry: loop {
        for r in ranges {
            if intersects(r.pa, r.len, alloc_pa, alloc_len) {
                alloc_pa = r.pa - alloc_len;
                /* make sure this still works with input constraints */
                if alloc_pa < range_pa {
                    return Err(ErrNO::NoMem);
                }

                continue 'retry;
            }
        }

        break;
    }

    alloc_range.pa = alloc_pa;
    alloc_range.len = alloc_len;
    Ok(())
}
