/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use crate::config_generated::*;

pub const NR_CPUS: usize = _CONFIG_NR_CPUS;

pub const PAGE_SHIFT: usize = _CONFIG_PAGE_SHIFT;
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
//pub const PAGE_MASK: usize = PAGE_SIZE - 1;

/* Virtual address where the kernel address space begins.
 * Below this is the user address space. */
pub const KERNEL_ASPACE_BASE: usize = 0xffff_0000_0000_0000;
pub const KERNEL_ASPACE_SIZE: usize = 0x0001_0000_0000_0000;
pub const KERNEL_ASPACE_MASK: usize = KERNEL_ASPACE_SIZE - 1;

/* map 512GB at the base of the kernel.
 * this is the max that can be mapped
 * with a single level 1 page table using 1GB pages.
 */
pub const ARCH_PHYSMAP_SIZE: usize = 1 << 39;

pub const BOOT_HEAP_SIZE: usize = _CONFIG_BOOT_HEAP_SIZE;

pub const KERNEL_BASE: usize = _CONFIG_KERNEL_BASE;

pub const SATP_MODE_39: usize = 0x8000000000000000;
pub const SATP_MODE_48: usize = 0x9000000000000000;

/* clang-format off */
macro_rules! IFTE {
    ($c: expr, $t: expr, $e: expr) => {
        if $c != 0usize { $t } else { $e }
    }
}

macro_rules! NBITS01 {
    ($n: expr) => {
        IFTE!($n, 1, 0)
    }
}
macro_rules! NBITS02 {
    ($n: expr) => {
        IFTE!(($n) >>  1,  1 + NBITS01!(($n) >>  1), NBITS01!($n))
    }
}
macro_rules! NBITS04 {
    ($n: expr) => {
        IFTE!(($n) >>  2,  2 + NBITS02!(($n) >>  2), NBITS02!($n))
    }
}
macro_rules! NBITS08 {
    ($n: expr) => {
        IFTE!(($n) >>  4,  4 + NBITS04!(($n) >>  4), NBITS04!($n))
    }
}
macro_rules! NBITS16 {
    ($n: expr) => {
        IFTE!(($n) >>  8,  8 + NBITS08!(($n) >>  8), NBITS08!($n))
    }
}
macro_rules! NBITS32 {
    ($n: expr) => {
        IFTE!(($n) >> 16, 16 + NBITS16!(($n) >> 16), NBITS16!($n))
    }
}
macro_rules! NBITS {
    ($n: expr) => {
        IFTE!(($n) >> 32, 32 + NBITS32!(($n) >> 32), NBITS32!($n))
    }
}

pub const KERNEL_ASPACE_BITS: usize = NBITS!(KERNEL_ASPACE_MASK);

/* These symbols come from kernel.ld */
extern "C" {
    pub fn __code_start();
    pub fn __bss_start();
    pub fn _end();
}
