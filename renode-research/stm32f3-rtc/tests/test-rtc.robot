*** Settings ***
Suite Setup                   Setup
Suite Teardown                Teardown
Test Setup                    Reset Emulation
Resource                      ${RENODEKEYWORDS}

*** Variables ***
${PLATFORM}                   ${CURDIR}/../stm32f3_rtc.repl
${ELF}                        ${CURDIR}/../target/thumbv7em-none-eabihf/release/stm32f3-rtc

*** Test Cases ***
Should Initialize RTC And Report
    [Documentation]           Verify RTC initializes and reports on UART
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     RTC Peripheral Test     timeout=5
    Wait For Line On Uart     RTC initialized         timeout=5

Should Set And Read Time
    [Documentation]           Verify RTC can set and read time
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     Time set: 0C:1E:00      timeout=5
    Wait For Line On Uart     Time read:              timeout=5

Should Report Test Summary
    [Documentation]           Verify RTC test completes successfully
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     RTC TEST PASSED         timeout=10
