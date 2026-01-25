*** Settings ***
Suite Setup                   Setup
Suite Teardown                Teardown
Test Setup                    Reset Emulation
Resource                      ${RENODEKEYWORDS}

*** Variables ***
${PLATFORM}                   ${CURDIR}/../stm32f3_timer.repl
${ELF}                        ${CURDIR}/../target/thumbv7em-none-eabihf/release/stm32f3-timer

*** Test Cases ***
Should Initialize Timer And Report
    [Documentation]           Verify Timer test initializes and reports on UART
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     Timer Peripheral Test    timeout=5

Should Complete Timer2 Countdown
    [Documentation]           Verify Timer2 countdown completes
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     Test 1: Timer2 Countdown    timeout=5
    Wait For Line On Uart     Timer2 started              timeout=5
    Wait For Line On Uart     Timer2 expired: PASS        timeout=10

Should Complete Timer3 Periodic
    [Documentation]           Verify Timer3 periodic mode works
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     Test 2: Timer3 Periodic     timeout=5
    Wait For Line On Uart     Period 01 complete          timeout=10
    Wait For Line On Uart     Period 02 complete          timeout=10
    Wait For Line On Uart     Period 03 complete          timeout=10

Should Increment Timer4 Counter
    [Documentation]           Verify Timer4 counter increments
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     Test 3: Timer4 Counter            timeout=15
    Wait For Line On Uart     Counter incrementing: PASS        timeout=5

Should Report Test Summary
    [Documentation]           Verify test summary shows results
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     === Test Summary ===    timeout=20
    Wait For Line On Uart     TIMER TEST              timeout=5
