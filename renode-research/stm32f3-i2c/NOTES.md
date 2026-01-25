# STM32F3 I2C Development Notes

## Renode I2C Support

Renode provides I2C peripheral emulation via `I2C.STM32F7_I2C` which is compatible with STM32F3.

### Available I2C Devices in Renode

- `Sensors.BME280` - Temperature/humidity/pressure sensor
- `I2C.EEPROM24c` - 24C series EEPROM
- Various other I2C sensors and peripherals

## Platform Configuration

The I2C sensor is attached in the .repl file:
```
i2cSensor: Sensors.BME280 @ i2c1 0x76
```

This creates a BME280 sensor at I2C address 0x76 on the I2C1 bus.

## STM32F3 I2C Pin Mapping

I2C1:
- PB6 = SCL (AF4)
- PB7 = SDA (AF4)

I2C2:
- PB10 = SCL (AF4)
- PB11 = SDA (AF4)

## BME280 Register Map

| Register | Address | Description |
|----------|---------|-------------|
| ID | 0xD0 | Chip ID (returns 0x60) |
| CTRL_HUM | 0xF2 | Humidity control |
| CTRL_MEAS | 0xF4 | Measurement control |
| TEMP_MSB | 0xFA | Temperature MSB |
| TEMP_LSB | 0xFB | Temperature LSB |
| TEMP_XLSB | 0xFC | Temperature XLSB |

## Issues Encountered

- I2C requires open-drain configuration for SCL/SDA pins
- AF4 is the alternate function for I2C1 on STM32F3
- BME280 sensor model in Renode returns chip ID 0x60

## UART PTY Terminal

Added PTY terminal for programmatic UART access:
```
emulation CreateUartPtyTerminal "term" "/tmp/uart" true
connector Connect sysbus.usart1 term
```
