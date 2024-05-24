/* This is a bare-bones linker script for the interpreter that has 1GB starting from 0x80000000. */

MEMORY
{
  RAM : ORIGIN = 0x80000000, LENGTH = 1024M
}

SECTIONS
{
  . = 0x80000000;
  .text : { *(.text) } > RAM
  .data : { *(.data) } > RAM
  .bss : { *(.bss) } > RAM
}
