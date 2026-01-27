//! STM32F3 I2C Sensor Test in Rust
//!
//! This tests I2C1 functionality on the STM32F303:
//! - I2C1 configured to communicate with a BME280 sensor
//! - Reads chip ID register to verify communication
//! - Reports results via USART1

#![no_std]
#![no_main]

use panic_halt as _;

use cortex_m_rt::entry;
use stm32f3_common::{uart_write_hex, uart_write_str};
use stm32f3xx_hal::{
    i2c::I2c,
    pac,
    prelude::*,
    serial::{config::Config as UartConfig, Serial},
};

// BME280 I2C address (0x76 with SDO to GND, 0x77 with SDO to VDD)
const BME280_ADDR: u8 = 0x76;

// BME280 Register addresses
const BME280_REG_ID: u8 = 0xD0;
const BME280_REG_CTRL_HUM: u8 = 0xF2;
const BME280_REG_CTRL_MEAS: u8 = 0xF4;
const BME280_REG_TEMP_MSB: u8 = 0xFA;

// Expected chip ID for BME280
const BME280_CHIP_ID: u8 = 0x60;

#[entry]
fn main() -> ! {
    // Take ownership of the device peripherals
    let dp = pac::Peripherals::take().unwrap();

    // Set up the system clocks using HSI (8 MHz internal oscillator)
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    // GPIO ports
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);
    let mut gpioe = dp.GPIOE.split(&mut rcc.ahb);

    // Configure LED on PE9 as output (for status indication)
    let mut led = gpioe
        .pe9
        .into_push_pull_output(&mut gpioe.moder, &mut gpioe.otyper);

    // Configure USART1 pins for debug output
    // PA9 = TX, PA10 = RX (Alternate Function 7)
    let tx_pin =
        gpioa
            .pa9
            .into_af_push_pull::<7>(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrh);
    let rx_pin =
        gpioa
            .pa10
            .into_af_push_pull::<7>(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrh);

    // Set up USART1 at 115200 baud
    let mut serial = Serial::new(
        dp.USART1,
        (tx_pin, rx_pin),
        UartConfig::default().baudrate(115200.Bd()),
        clocks,
        &mut rcc.apb2,
    );

    uart_write_str(&mut serial, "I2C1 Sensor Test\n");

    // Configure I2C1 pins (Alternate Function 4)
    // PB6 = SCL, PB7 = SDA
    let scl =
        gpiob
            .pb6
            .into_af_open_drain::<4>(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
    let sda =
        gpiob
            .pb7
            .into_af_open_drain::<4>(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);

    // Configure I2C1 at 100kHz
    let mut i2c = I2c::new(
        dp.I2C1,
        (scl, sda),
        100_000.Hz(),
        clocks,
        &mut rcc.apb1,
    );

    uart_write_str(&mut serial, "I2C1 initialized\n");

    let mut pass_count = 0u8;
    let mut fail_count = 0u8;

    uart_write_str(&mut serial, "Starting I2C test...\n");

    // Test 1: Read chip ID register
    uart_write_str(&mut serial, "\nTest 1: Read Chip ID\n");
    let mut id_buf = [0u8; 1];
    match i2c.write_read(BME280_ADDR, &[BME280_REG_ID], &mut id_buf) {
        Ok(_) => {
            uart_write_str(&mut serial, "Chip ID: 0x");
            uart_write_hex(&mut serial, id_buf[0]);
            uart_write_str(&mut serial, " Expected: 0x");
            uart_write_hex(&mut serial, BME280_CHIP_ID);
            if id_buf[0] == BME280_CHIP_ID {
                uart_write_str(&mut serial, " PASS\n");
                pass_count += 1;
                led.set_high().ok();
            } else {
                uart_write_str(&mut serial, " FAIL\n");
                fail_count += 1;
            }
        }
        Err(_) => {
            uart_write_str(&mut serial, "I2C read error FAIL\n");
            fail_count += 1;
        }
    }

    // Test 2: Write and read back humidity control register
    uart_write_str(&mut serial, "\nTest 2: Write/Read CTRL_HUM\n");
    let ctrl_hum_val: u8 = 0x01; // oversampling x1
    match i2c.write(BME280_ADDR, &[BME280_REG_CTRL_HUM, ctrl_hum_val]) {
        Ok(_) => {
            uart_write_str(&mut serial, "Write CTRL_HUM: 0x");
            uart_write_hex(&mut serial, ctrl_hum_val);
            uart_write_str(&mut serial, " OK\n");

            // Read back
            let mut read_buf = [0u8; 1];
            match i2c.write_read(BME280_ADDR, &[BME280_REG_CTRL_HUM], &mut read_buf) {
                Ok(_) => {
                    uart_write_str(&mut serial, "Read CTRL_HUM: 0x");
                    uart_write_hex(&mut serial, read_buf[0]);
                    if read_buf[0] == ctrl_hum_val {
                        uart_write_str(&mut serial, " PASS\n");
                        pass_count += 1;
                    } else {
                        uart_write_str(&mut serial, " FAIL\n");
                        fail_count += 1;
                    }
                }
                Err(_) => {
                    uart_write_str(&mut serial, "I2C read error FAIL\n");
                    fail_count += 1;
                }
            }
        }
        Err(_) => {
            uart_write_str(&mut serial, "I2C write error FAIL\n");
            fail_count += 1;
        }
    }

    // Test 3: Configure and trigger measurement
    uart_write_str(&mut serial, "\nTest 3: Trigger Measurement\n");
    // Set temp oversampling x1, pressure oversampling x1, forced mode
    let ctrl_meas_val: u8 = 0x25; // osrs_t=001, osrs_p=001, mode=01
    match i2c.write(BME280_ADDR, &[BME280_REG_CTRL_MEAS, ctrl_meas_val]) {
        Ok(_) => {
            uart_write_str(&mut serial, "Write CTRL_MEAS: 0x");
            uart_write_hex(&mut serial, ctrl_meas_val);
            uart_write_str(&mut serial, " OK\n");

            // Small delay for measurement (in real hardware)
            for _ in 0..10000 {
                cortex_m::asm::nop();
            }

            // Read temperature registers (3 bytes: MSB, LSB, XLSB)
            let mut temp_buf = [0u8; 3];
            match i2c.write_read(BME280_ADDR, &[BME280_REG_TEMP_MSB], &mut temp_buf) {
                Ok(_) => {
                    uart_write_str(&mut serial, "Temp raw: 0x");
                    uart_write_hex(&mut serial, temp_buf[0]);
                    uart_write_hex(&mut serial, temp_buf[1]);
                    uart_write_hex(&mut serial, temp_buf[2]);
                    uart_write_str(&mut serial, " PASS\n");
                    pass_count += 1;
                }
                Err(_) => {
                    uart_write_str(&mut serial, "I2C read error FAIL\n");
                    fail_count += 1;
                }
            }
        }
        Err(_) => {
            uart_write_str(&mut serial, "I2C write error FAIL\n");
            fail_count += 1;
        }
    }

    // Summary
    uart_write_str(&mut serial, "\n=== Test Summary ===\n");
    uart_write_str(&mut serial, "Passed: ");
    uart_write_hex(&mut serial, pass_count);
    uart_write_str(&mut serial, "\nFailed: ");
    uart_write_hex(&mut serial, fail_count);
    uart_write_str(&mut serial, "\n");

    if fail_count == 0 {
        uart_write_str(&mut serial, "I2C TEST PASSED\n");
        led.set_high().ok();
    } else {
        uart_write_str(&mut serial, "I2C TEST FAILED\n");
        led.set_low().ok();
    }

    // Halt
    loop {
        cortex_m::asm::wfi();
    }
}
