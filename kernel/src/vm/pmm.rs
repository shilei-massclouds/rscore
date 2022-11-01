/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

use crate::{
    dprint, INFO, PmmNode, ErrNO,
};

/* all of the configured memory arenas */
pub const MAX_ARENAS: usize = 16;

pub struct ArenaInfo<'a> {
    pub name: &'a str,
    pub flags: u32,
    pub base: usize,
    pub size: usize,
}

impl<'a> ArenaInfo<'a> {
    pub fn new(name: &str, flags: u32, base: usize, size: usize)
        -> ArenaInfo {

        ArenaInfo {
            name, flags, base, size
        }
    }
}

pub fn pmm_add_arena(info: &ArenaInfo, pmm_node: &mut PmmNode)
    -> Result<(), ErrNO> {

    dprint!(INFO, "Arena.{}: flags[{:x}] {:x} {:x}\n",
            info.name, info.flags, info.base, info.size);
    pmm_node.add_arena(info)
}
