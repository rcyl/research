# STM32F3 RTC (Real-Time Clock) - Development Notes

## Project Status
- COMPLETE - All tests passing

## Implementation Details

### RTC Registers
- TR (0x00): Time register (BCD format)
- DR (0x04): Date register (BCD format)
- CR (0x08): Control register
- ISR (0x0C): Initialization and status register
- WPR (0x24): Write protection register

### Initialization Sequence
1. Enable PWR clock via RCC_APB1ENR
2. Enable backup domain access via PWR_CR.DBP
3. Enable LSI and select as RTC clock source
4. Disable write protection (write 0xCA then 0x53 to WPR)
5. Enter init mode (set INIT bit in ISR)
6. Wait for INITF flag
7. Configure time/date
8. Exit init mode
9. Re-enable write protection

### Renode Model
Using `Timers.STM32F4_RTC` at address 0x40002800.

## Build Log
```
cargo build --release
Finished `release` profile [optimized + debuginfo] target(s)
```

## Test Results
All 3 tests PASSED:
- Should Initialize RTC And Report: OK
- Should Set And Read Time: OK
- Should Report Test Summary: OK

## Known Issues / Limitations
- Time values are output in hex format (0C:1E:00 = 12:30:00)
- RTC initialization uses direct register access
- Renode's STM32F4_RTC model works correctly for basic time operations
- Alarm functionality not tested
