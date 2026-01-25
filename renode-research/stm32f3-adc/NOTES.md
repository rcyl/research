# STM32F3 ADC (Analog-to-Digital Converter) - Development Notes

## Project Status
- Complete and validated

## Implementation Details

### ADC Registers
- ISR (0x00): Interrupt and status register
- CR (0x08): Control register
- CFGR (0x0C): Configuration register
- SQR1 (0x30): Regular sequence register 1
- DR (0x40): Regular data register

### ADC Common Registers (0x50000300)
- CCR (0x08): Common control register (clock mode)

### Initialization Sequence
1. Enable ADC clock via RCC_AHBENR (bit 28)
2. Configure clock mode in ADC_CCR
3. Ensure ADC is disabled
4. Configure CFGR (resolution, alignment, mode)
5. Set sequence in SQR1
6. Enable ADC (ADEN)
7. Wait for ADRDY flag

### Conversion Sequence
1. Start conversion (ADSTART)
2. Wait for EOC flag
3. Read DR

### Renode Model
Using `Analog.STM32F0_ADC` at address 0x50000000.

## Build Log
```
cargo build --release - SUCCESS
```

## Test Results
All tests passing (2026-01-25):
- Should Initialize ADC And Report: PASS (0.71s)
- Should Perform ADC Conversion: PASS (0.19s)
- Should Report Test Summary: PASS (0.19s)

## Known Issues / Limitations
- STM32F0_ADC model requires `referenceVoltage` and `externalEventFrequency` parameters
- Fixed platform definition to include: `referenceVoltage: 3.3` and `externalEventFrequency: 1000`
- ADC returns simulated conversion values (not tied to actual analog input pins in Renode)
