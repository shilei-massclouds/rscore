/*
 * Use of this source code is governed by a MIT-style license
 * that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

pub const MAX_ZBI_MEM_RANGES: usize = 32;

pub enum ZBIMemRangeType {
    RAM,
    PERIPHERAL,
    _RESERVED,
}

pub struct ZBIMemRange {
    pub mtype:      ZBIMemRangeType,
    pub paddr:      usize,
    pub length:     usize,
    pub reserved:   u32,
}

impl ZBIMemRange {
    pub fn new(mtype: ZBIMemRangeType, paddr: usize, length: usize)
        -> ZBIMemRange {

        ZBIMemRange {
            mtype,
            paddr,
            length,
            reserved: 0,
        }
    }
}
