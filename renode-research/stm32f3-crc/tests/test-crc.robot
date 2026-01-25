*** Settings ***
Suite Setup                   Setup
Suite Teardown                Teardown
Test Setup                    Reset Emulation
Resource                      ${RENODEKEYWORDS}

*** Variables ***
${PLATFORM}                   ${CURDIR}/../stm32f3_crc.repl
${ELF}                        ${CURDIR}/../target/thumbv7em-none-eabihf/release/stm32f3-crc

*** Test Cases ***
Should Initialize CRC And Report
    [Documentation]           Verify CRC initializes and reports on UART
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     CRC Peripheral Test     timeout=5
    Wait For Line On Uart     CRC clock enabled       timeout=5

Should Calculate Single Word CRC
    [Documentation]           Verify CRC calculates single word correctly
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     Single word CRC: PASS    timeout=5

Should Calculate Multiple Word CRC
    [Documentation]           Verify CRC calculates multiple words correctly
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     Multiple word CRC: PASS    timeout=5

Should Reset CRC Correctly
    [Documentation]           Verify CRC reset functionality
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     CRC reset: PASS    timeout=5

Should Report Test Summary
    [Documentation]           Verify CRC test completes successfully
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     CRC TEST PASSED    timeout=10
