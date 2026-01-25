/* STM32F303xC Memory Layout */
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 256K
  CCM   : ORIGIN = 0x10000000, LENGTH = 8K
  RAM   : ORIGIN = 0x20000000, LENGTH = 40K
}

/* Entry point */
ENTRY(Reset);

/* Stack pointer initial value */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);
