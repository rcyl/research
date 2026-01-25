*** Settings ***
Suite Setup                   Setup
Suite Teardown                Teardown
Test Setup                    Reset Emulation
Resource                      ${RENODEKEYWORDS}

*** Variables ***
${PLATFORM}                   /src/renode-research/stm32f3-spi/stm32f3_spi.repl
${ELF}                        /src/renode-research/stm32f3-spi/target/thumbv7em-none-eabihf/release/stm32f3-spi

*** Test Cases ***
Should Initialize SPI And Report
    [Documentation]           Verify SPI1 initializes and reports on UART
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     SPI1 Loopback Test    timeout=5
    Wait For Line On Uart     SPI1 initialized      timeout=5

Should Pass SPI Loopback Test
    [Documentation]           Verify SPI loopback test passes with all bytes matching
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     Starting loopback test    timeout=5

    # Verify each test byte passes (TX should equal RX in loopback)
    Wait For Line On Uart     TX: 0xAA RX: 0xAA PASS    timeout=5
    Wait For Line On Uart     TX: 0x55 RX: 0x55 PASS    timeout=5
    Wait For Line On Uart     TX: 0x12 RX: 0x12 PASS    timeout=5
    Wait For Line On Uart     TX: 0x34 RX: 0x34 PASS    timeout=5
    Wait For Line On Uart     TX: 0xFF RX: 0xFF PASS    timeout=5

Should Report Test Summary
    [Documentation]           Verify test summary shows all passed
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     === Test Summary ===      timeout=10
    Wait For Line On Uart     Passed: 05                timeout=5
    Wait For Line On Uart     Failed: 00                timeout=5
    Wait For Line On Uart     SPI TEST PASSED           timeout=5
