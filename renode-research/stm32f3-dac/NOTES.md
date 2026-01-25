# STM32F3 DAC (Digital-to-Analog Converter) - Development Notes

## Project Status
- Complete and validated

## Implementation Details

### DAC Features Tested
1. **Channel 1 Output** - Write and verify 12-bit value on DAC1
2. **Channel 2 Output** - Write and verify 12-bit value on DAC2
3. **Value Range** - Test full 12-bit range (0, 2047, 4095)

### DAC Specification
- 2 channels (DAC1, DAC2)
- 12-bit resolution (0-4095)
- Output pins: PA4 (DAC1), PA5 (DAC2)
- Voltage range: 0V to VREF+ (typically 3.3V)

### Register Map (Base: 0x40007400)
| Offset | Register  | Description |
|--------|-----------|-------------|
| 0x00   | CR        | Control register |
| 0x04   | SWTRIGR   | Software trigger register (write-only) |
| 0x08   | DHR12R1   | Channel 1 12-bit right-aligned data |
| 0x0C   | DHR12L1   | Channel 1 12-bit left-aligned data |
| 0x10   | DHR8R1    | Channel 1 8-bit right-aligned data |
| 0x14   | DHR12R2   | Channel 2 12-bit right-aligned data |
| 0x18   | DHR12L2   | Channel 2 12-bit left-aligned data |
| 0x1C   | DHR8R2    | Channel 2 8-bit right-aligned data |
| 0x20   | DHR12RD   | Dual channel 12-bit right-aligned |
| 0x24   | DHR12LD   | Dual channel 12-bit left-aligned |
| 0x28   | DHR8RD    | Dual channel 8-bit right-aligned |
| 0x2C   | DOR1      | Channel 1 data output register (read-only) |
| 0x30   | DOR2      | Channel 2 data output register (read-only) |
| 0x34   | SR        | Status register |

### CR Register Bits (per channel, CH2 offset by 16)
| Bits    | Name  | Description |
|---------|-------|-------------|
| [0]     | EN    | DAC channel enable |
| [1]     | BOFF  | Output buffer disable |
| [2]     | TEN   | Trigger enable |
| [5:3]   | TSEL  | Trigger selection (111 = software) |
| [11:6]  | WAVEx | Wave generation mode |
| [15:12] | MAMPx | Mask/amplitude selector |

### Data Transfer Behavior
- **TEN=0 (Trigger disabled)**: DHR → DOR transfer is immediate on write
- **TEN=1 (Trigger enabled)**: DHR → DOR transfer occurs on trigger event
  - Software trigger via SWTRIGR
  - Hardware triggers (timer, external) not emulated

### Clock Enable
- DAC clock enabled via RCC_APB1ENR bit 29

## Renode Python Peripheral Implementation

### Best Practices Applied
1. **Persistent Variables**: Variables in `isInit` block persist across calls
   - Do NOT use `self.` prefix in PythonPeripheral scripts
   - Renode maintains execution context automatically

2. **Register Documentation**: Full register map in header comments

3. **Proper Bit Masking**:
   - 12-bit values masked with 0xFFF
   - 8-bit values masked with 0xFF
   - Left-aligned values masked with 0xFFF0

4. **Write-Only Registers**: SWTRIGR reads as 0

5. **Read-Only Registers**: DOR1/DOR2 writes ignored

6. **Trigger Logic**: Proper TEN bit checking before DHR→DOR transfer

### Implementation Pattern
```python
# DHR to DOR transfer with trigger check:
if request.offset == 0x08:    # DHR12R1
    dac_dhr12r1 = request.value & 0xFFF
    ch1_en = dac_cr & 0x01
    ch1_ten = dac_cr & 0x04
    if ch1_en and not ch1_ten:  # Immediate transfer if TEN=0
        dac_dor1 = dac_dhr12r1
```

## Build Log
```
cargo build --release - SUCCESS
```

## Test Results
All tests passing (2026-01-26):
- Should Initialize DAC And Report: PASS
- Should Write To DAC Channel 1: PASS
- Should Write To DAC Channel 2: PASS
- Should Handle DAC Value Range: PASS
- Should Report Test Summary: PASS

## Known Issues / Limitations

### Implemented
- CR register with EN, TEN bits for both channels
- All DHR formats (12R, 12L, 8R) for single and dual channel
- DOR read-only output registers
- SWTRIGR software trigger
- Immediate transfer when TEN=0
- Triggered transfer when TEN=1 and SWTRIG written
- SR status register

### Not Implemented
- BOFF (output buffer disable) - stored but no effect
- TSEL hardware trigger selection (timer, external triggers)
- WAVEx wave generation modes (noise, triangle)
- MAMPx mask/amplitude for wave generation
- DMA underrun detection
- Actual analog voltage output (values in DOR only)

### Hardware vs Emulation Differences
- Real DAC outputs analog voltage on PA4/PA5 pins
- Emulation only stores digital values in DOR registers
- Hardware triggers (TIM6, TIM7, external) not connected
- Output buffer behavior not simulated
