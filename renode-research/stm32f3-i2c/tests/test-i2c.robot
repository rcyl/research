*** Settings ***
Suite Setup                   Setup
Suite Teardown                Teardown
Test Setup                    Reset Emulation
Resource                      ${RENODEKEYWORDS}

*** Variables ***
${PLATFORM}                   ${CURDIR}/../stm32f3_i2c.repl
${ELF}                        ${CURDIR}/../../target/thumbv7em-none-eabihf/release/stm32f3-i2c

*** Test Cases ***
Should Initialize I2C And Report
    [Documentation]           Verify I2C1 initializes and reports on UART
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     I2C1 Sensor Test    timeout=5
    Wait For Line On Uart     I2C1 initialized    timeout=5

Should Read BME280 Chip ID
    [Documentation]           Verify I2C can read the BME280 chip ID
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     Test 1: Read Chip ID    timeout=5
    Wait For Line On Uart     Chip ID: 0x60           timeout=5

Should Write And Read I2C Registers
    [Documentation]           Verify I2C write/read operations work
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     Test 2: Write/Read CTRL_HUM    timeout=5
    Wait For Line On Uart     Write CTRL_HUM: 0x01 OK        timeout=5

Should Report Test Summary
    [Documentation]           Verify test summary shows results
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     === Test Summary ===    timeout=10
    Wait For Line On Uart     I2C TEST                timeout=5
