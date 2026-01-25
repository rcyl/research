/* STM32F407VGT6 Memory Layout */
MEMORY
{
  /* Flash memory begins at 0x08000000 and has a size of 1MB */
  FLASH : ORIGIN = 0x08000000, LENGTH = 1024K
  /* CCM (Core Coupled Memory) at 0x10000000, 64KB */
  CCM : ORIGIN = 0x10000000, LENGTH = 64K
  /* Main RAM begins at 0x20000000 and has a size of 128KB */
  RAM : ORIGIN = 0x20000000, LENGTH = 128K
}

/* The entry point is the reset handler */
ENTRY(Reset);

/* Stack size */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);
