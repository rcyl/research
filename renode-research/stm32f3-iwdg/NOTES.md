# STM32F3 IWDG (Independent Watchdog) - Development Notes

## Project Status
- COMPLETE - All tests passing

## Implementation Details

### IWDG Registers
- KR (0x00): Key register - controls access and operation
- PR (0x04): Prescaler register - divides LSI clock
- RLR (0x08): Reload register - counter reload value
- SR (0x0C): Status register - update flags

### Key Values
- 0xAAAA: Reload counter (feed the dog)
- 0xCCCC: Start watchdog
- 0x5555: Enable write access to PR/RLR

### Renode Model
Using `Timers.STM32_IndependentWatchdog` at address 0x40003000.
LSI frequency configured as 40kHz.

## Build Log
```
cargo build --release
Finished `release` profile [optimized + debuginfo] target(s)
```

## Test Results
All 3 tests PASSED:
- Should Initialize IWDG And Report: OK
- Should Feed Watchdog Successfully: OK
- Should Report Test Summary: OK

## Known Issues / Limitations
- Renode's IWDG model works correctly for basic operations
- Watchdog feeding via KEY_RELOAD (0xAAAA) works as expected
- No reset testing performed (would need separate test scenario)
