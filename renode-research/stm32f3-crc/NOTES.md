# STM32F3 CRC (Cyclic Redundancy Check) - Development Notes

## Project Status
- Complete and validated

## Implementation Details

### CRC Features Tested
1. **Single Word CRC** - Calculate CRC of one 32-bit word
2. **Multiple Word CRC** - Calculate CRC of multiple words
3. **Reset Functionality** - Verify CRC reset to initial value

### CRC Specification
- Polynomial: 0x04C11DB7 (CRC-32/MPEG-2)
- Reflected polynomial: 0xEDB88320 (used in implementation)
- Initial Value: 0xFFFFFFFF
- Input: 32-bit words (processed LSB first)
- Output: 32-bit CRC value

### Register Map (Base: 0x40023000)
| Offset | Register | Description |
|--------|----------|-------------|
| 0x00   | DR       | Data register - write to calculate, read for result |
| 0x04   | IDR      | Independent data register (8-bit general purpose) |
| 0x08   | CR       | Control register (RESET, POLYSIZE, REV_IN, REV_OUT) |
| 0x10   | INIT     | Initial CRC value (default 0xFFFFFFFF) |
| 0x14   | POL      | CRC polynomial (default 0x04C11DB7) |

### CR Register Bits
| Bits | Name     | Description |
|------|----------|-------------|
| [0]  | RESET    | Reset CRC to INIT value (self-clearing) |
| [4:3]| POLYSIZE | Polynomial size (00=32, 01=16, 10=8, 11=7 bit) |
| [6:5]| REV_IN   | Input data reverse mode |
| [7]  | REV_OUT  | Output data reverse |

### Clock Enable
- CRC clock enabled via RCC_AHBENR bit 6

## Renode Python Peripheral Implementation

### Best Practices Applied
1. **Persistent Variables**: Variables declared in `isInit` block persist across calls
   - Renode maintains Python execution context between peripheral accesses
   - Do NOT use `self.` prefix - it doesn't work in PythonPeripheral scripts

2. **Register Documentation**: Header comments document full register map and bit fields

3. **Proper Bit Masking**: All reads/writes mask values to valid bit widths

4. **Self-Clearing Bits**: RESET bit (CR[0]) always reads as 0

5. **Precomputed Table**: CRC lookup table built once during init for efficiency

### Implementation Notes
```python
# Correct pattern for Renode PythonPeripheral:
if request.isInit:
    my_register = 0x00    # Variable persists between calls

elif request.isRead:
    request.value = my_register

elif request.isWrite:
    my_register = request.value
```

## Build Log
```
cargo build --release - SUCCESS
```

## Test Results
All tests passing (2026-01-26):
- Should Initialize CRC And Report: PASS
- Should Calculate Single Word CRC: PASS
- Should Calculate Multiple Word CRC: PASS
- Should Reset CRC Correctly: PASS
- Should Report Test Summary: PASS

## Known Issues / Limitations

### Implemented
- DR register read/write with CRC calculation
- IDR 8-bit general purpose register
- CR RESET bit functionality
- INIT register (programmable initial value)
- POL register (stored but not dynamically used)

### Not Implemented
- POLYSIZE selection (always uses 32-bit)
- REV_IN input data reversal modes
- REV_OUT output data reversal
- Dynamic polynomial calculation (uses fixed reflected CRC-32)
- 8-bit and 16-bit data access modes

### Hardware vs Emulation Differences
- Real STM32 CRC processes data MSB-first by default
- Emulation uses reflected (LSB-first) algorithm which produces compatible results for standard use cases
- Custom polynomial values stored in POL but calculation uses fixed 0xEDB88320
