# STM32F3 I2C Sensor Test

This example demonstrates I2C peripheral testing on STM32F303 using Renode emulation.

## Overview

- Tests I2C1 communication with a simulated BME280 temperature/pressure/humidity sensor
- Verifies chip ID reading, register write/read operations, and measurement triggering
- Reports all results via USART1

## Hardware Configuration

| Peripheral | Pins | Description |
|------------|------|-------------|
| I2C1 SCL | PB6 | I2C clock (AF4) |
| I2C1 SDA | PB7 | I2C data (AF4) |
| USART1 TX | PA9 | Debug output (AF7) |
| USART1 RX | PA10 | Debug input (AF7) |
| LED | PE9 | Status indicator |
| Button | PA0 | User button |

## I2C Device

- **BME280** environmental sensor at address 0x76
- Chip ID register (0xD0) returns 0x60
- Supports temperature, pressure, and humidity measurements

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
renode-test tests/test-i2c.robot
```

## UART Access

UART output is available at `/tmp/uart` via PTY terminal:

```bash
cat /tmp/uart
# or
screen /tmp/uart
```

## Test Output Example

```
I2C1 Sensor Test
I2C1 initialized
Starting I2C test...

Test 1: Read Chip ID
Chip ID: 0x60 Expected: 0x60 PASS

Test 2: Write/Read CTRL_HUM
Write CTRL_HUM: 0x01 OK
Read CTRL_HUM: 0x01 PASS

Test 3: Trigger Measurement
Write CTRL_MEAS: 0x25 OK
Temp raw: 0x800000 PASS

=== Test Summary ===
Passed: 03
Failed: 00
I2C TEST PASSED
```
