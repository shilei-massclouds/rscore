/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use crate::{
    dprint, INFO, PmmNode, ErrNO, BootReserveRange, paddr_t,
};
use alloc::vec::Vec;
use alloc::string::String;
use crate::lib::list::List;
use crate::vm::page::vm_page_t;

/* all of the configured memory arenas */
pub const MAX_ARENAS: usize = 16;

pub struct ArenaInfo {
    pub name: String,
    pub flags: u32,
    pub base: usize,
    pub size: usize,
}

impl ArenaInfo {
    pub fn new(name: &str, flags: u32, base: usize, size: usize)
        -> ArenaInfo {

        ArenaInfo {
            name: String::from(name),
            flags, base, size
        }
    }
}

pub fn pmm_add_arena(info: ArenaInfo, pmm_node: &mut PmmNode,
                     reserve_ranges: &Vec<BootReserveRange>)
    -> Result<(), ErrNO> {

    dprint!(INFO, "Arena.{}: flags[{:x}] {:x} {:x}\n",
            info.name, info.flags, info.base, info.size);
    pmm_node.add_arena(info, reserve_ranges)
}

pub fn pmm_alloc_range(paddr: paddr_t, count: usize,
                       pmm_node: &mut PmmNode,
                       list: &mut List<vm_page_t>)
    -> Result<(), ErrNO> {

    pmm_node.alloc_range(paddr, count, list)
}
