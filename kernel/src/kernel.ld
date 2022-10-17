/*
 * Use of this source code is governed by a MIT-style
 * license that can be found in the LICENSE file or
 * at https://opensource.org/licenses/MIT
 */

OUTPUT_ARCH(riscv)
ENTRY(_start)

INCLUDE config.generated.ld

SECTIONS
{
    . = KERNEL_BASE;
    __code_start = .;

    .head.text : {
        KEEP(*(.head.text))
    }

    .text : {
        *(.text .text.*)
    }

    PROVIDE(__code_end = .);

    /*
     * The kernel's actual segments are aligned to the -z max-page-size=...
     * value, which is 64k for ARM.  But the exported VMOs within segments
     * are only aligned to 4k, since that's still the user-visible page size.
     */
    . = ALIGN(CONSTANT(MAXPAGESIZE));
    PROVIDE(__rodata_start = .);

    .init_array : {
        PROVIDE(__init_array_start = .);
        KEEP(*(SORT_BY_INIT_PRIORITY(.init_array.*)))
        KEEP(*(.init_array .ctors))
        PROVIDE(__init_array_end = .);
        ASSERT(ALIGNOF(.init_array) == 8 || SIZEOF(.init_array) == 0,
               ".init_array overalignment -> maybe padding gaps");
    }

    .rodata ALIGN(4K) : {
        *(.rodata) *(.rodata.*)
    }
    .srodata : {
        *(.srodata*)
    }

    .data : ALIGN(CONSTANT(MAXPAGESIZE)) {
        PROVIDE(__rodata_end = .);
        PROVIDE(__data_start = .);

        *(.data .data.*)
    }
    .sdata : {
        __global_pointer$ = . + 0x800;  /* For the global pointer */
        *(.sdata*)
    }

    PROVIDE(__data_end = .);

    .bss ALIGN(4K) : {
        PROVIDE(__bss_start = .);

        bootstack = .;
        *(.bss.bootstack)
        bootstacktop = .;

        . = ALIGN(4K);
        sbss = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)
        *(COMMON)
        ebss = .;
    }

     . = ALIGN(CONSTANT(MAXPAGESIZE));
    PROVIDE(_end = .);
}
