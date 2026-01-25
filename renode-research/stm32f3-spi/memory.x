/* STM32F303xC Memory Layout */
MEMORY
{
  /* Flash memory begins at 0x08000000 and has a size of 256KB */
  FLASH : ORIGIN = 0x08000000, LENGTH = 256K
  /* CCM (Core Coupled Memory) at 0x10000000, 8KB */
  CCM : ORIGIN = 0x10000000, LENGTH = 8K
  /* Main RAM begins at 0x20000000 and has a size of 40KB */
  RAM : ORIGIN = 0x20000000, LENGTH = 40K
}

/* The entry point is the reset handler */
ENTRY(Reset);

/* Stack size */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);
