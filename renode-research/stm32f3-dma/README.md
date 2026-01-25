# STM32F3 DMA Peripheral Test

This example demonstrates DMA peripheral testing on STM32F303 using Renode emulation.

## Overview

- Tests DMA1 Channel1 memory-to-memory transfer
- Verifies data integrity after transfer
- Checks transfer complete flag and NDTR register
- Reports all results via USART1

## Hardware Configuration

| Peripheral | Description |
|------------|-------------|
| DMA1 | 7-channel DMA controller |
| DMA1_CH1 | Channel 1 for M2M transfer |
| USART1 TX | PA9 - Debug output |
| USART1 RX | PA10 - Debug input |
| LED | PE9 - Status indicator |

## DMA Features Tested

1. **Memory-to-Memory Transfer** - 16-byte buffer copy
2. **Transfer Complete Flag** - TCIF1 status verification
3. **NDTR Register** - Data count decrement to zero
4. **Multiple Transfers** - Reconfigure and transfer again

## Building

```bash
cargo build --release
```

## Running in Renode

```bash
renode renode-config.resc
start
```

## Running Tests

```bash
renode-test tests/test-dma.robot
```

## UART Access

```bash
cat /tmp/uart
```

## Test Output Example

```
DMA Peripheral Test

Test 1: Memory-to-Memory Transfer
SRC: 0x20000000
DST: 0x20000010
DMA transfer started
Transfer complete flag: SET
Verifying data...
Data verified: PASS

Test 2: NDTR Register
NDTR after transfer: 0000
NDTR is zero: PASS

Test 3: Second Transfer
Second transfer: PASS

=== Test Summary ===
Passed: 03
Failed: 00
DMA TEST PASSED
```
