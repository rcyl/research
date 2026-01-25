# STM32F3 Timer Peripheral Test

This example demonstrates Timer peripheral testing on STM32F303 using Renode emulation.

## Overview

- Tests Timer2, Timer3, and Timer4 functionality
- Timer2: Countdown/delay mode
- Timer3: Periodic mode with multiple periods
- Timer4: Direct counter register access
- Reports all results via USART1

## Hardware Configuration

| Peripheral | Description |
|------------|-------------|
| TIM2 | 32-bit general purpose timer (APB1) |
| TIM3 | 16-bit general purpose timer (APB1) |
| TIM4 | 16-bit general purpose timer (APB1) |
| USART1 TX | PA9 - Debug output |
| USART1 RX | PA10 - Debug input |
| LED | PE9 - Status indicator |

## Timer Features Tested

1. **Countdown Timer** - Single-shot delay
2. **Periodic Timer** - Auto-reload mode
3. **Counter Register** - Direct CNT register read

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
renode-test tests/test-timer.robot
```

## UART Access

```bash
cat /tmp/uart
```

## Test Output Example

```
Timer Peripheral Test

Test 1: Timer2 Countdown
Timer2 started (100ms)
Timer2 expired: PASS

Test 2: Timer3 Periodic
Timer3 started (50ms periodic)
Period 01 complete
Period 02 complete
Period 03 complete
Timer3 periodic: PASS

Test 3: Timer4 Counter
CNT1: 0x00000000
CNT2: 0x00000042
Counter incrementing: PASS

=== Test Summary ===
Passed: 03
Failed: 00
TIMER TEST PASSED
```
