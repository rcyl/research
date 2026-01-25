*** Settings ***
Suite Setup                   Setup
Suite Teardown                Teardown
Test Setup                    Reset Emulation
Resource                      ${RENODEKEYWORDS}

*** Variables ***
${PLATFORM}                   ${CURDIR}/../stm32f3_iwdg.repl
${ELF}                        ${CURDIR}/../target/thumbv7em-none-eabihf/release/stm32f3-iwdg

*** Test Cases ***
Should Initialize IWDG And Report
    [Documentation]           Verify IWDG initializes and reports on UART
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     IWDG Peripheral Test    timeout=5
    Wait For Line On Uart     IWDG initialized        timeout=5

Should Feed Watchdog Successfully
    [Documentation]           Verify watchdog can be fed multiple times without reset
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     Feeding watchdog        timeout=5
    Wait For Line On Uart     Feed 01: OK             timeout=5
    Wait For Line On Uart     Feed 02: OK             timeout=5
    Wait For Line On Uart     Feed 03: OK             timeout=5

Should Report Test Summary
    [Documentation]           Verify IWDG test completes successfully
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     IWDG TEST PASSED        timeout=10
