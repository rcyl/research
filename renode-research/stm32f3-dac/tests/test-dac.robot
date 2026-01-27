*** Settings ***
Suite Setup                   Setup
Suite Teardown                Teardown
Test Setup                    Reset Emulation
Resource                      ${RENODEKEYWORDS}

*** Variables ***
${PLATFORM}                   ${CURDIR}/../stm32f3_dac.repl
${ELF}                        ${CURDIR}/../../target/thumbv7em-none-eabihf/release/stm32f3-dac

*** Test Cases ***
Should Initialize DAC And Report
    [Documentation]           Verify DAC initializes and reports on UART
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     DAC Peripheral Test     timeout=5
    Wait For Line On Uart     DAC clock enabled       timeout=5
    Wait For Line On Uart     DAC channels enabled    timeout=5

Should Write To DAC Channel 1
    [Documentation]           Verify DAC Channel 1 output works
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     DAC Channel 1: PASS    timeout=5

Should Write To DAC Channel 2
    [Documentation]           Verify DAC Channel 2 output works
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     DAC Channel 2: PASS    timeout=5

Should Handle DAC Value Range
    [Documentation]           Verify DAC handles full 12-bit range
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     DAC Value Range: PASS    timeout=5

Should Report Test Summary
    [Documentation]           Verify DAC test completes successfully
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     DAC TEST PASSED    timeout=10
