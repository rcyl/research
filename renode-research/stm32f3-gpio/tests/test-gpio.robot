*** Settings ***
Suite Setup                   Setup
Suite Teardown                Teardown
Test Setup                    Reset Emulation
Resource                      ${RENODEKEYWORDS}

*** Variables ***
${PLATFORM}                   ${CURDIR}/../stm32f3_gpio.repl
${ELF}                        ${CURDIR}/../../target/thumbv7em-none-eabihf/release/stm32f3-gpio

*** Test Cases ***
Should Initialize GPIO And Report
    [Documentation]           Verify GPIO initializes and reports on UART
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     GPIO Peripheral Test     timeout=5

Should Complete Output Toggle Test
    [Documentation]           Verify GPIO output toggle works
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     Output toggle test: PASS    timeout=5

Should Read Button Input
    [Documentation]           Verify GPIO can read button input
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     Waiting for button press    timeout=5

    # Press the button
    Execute Command           gpioPortA.UserButton Press

    Wait For Line On Uart     Button press detected: PASS    timeout=5

    # Release the button
    Execute Command           gpioPortA.UserButton Release

    Wait For Line On Uart     Button release detected: PASS    timeout=5

Should Test Pull Configuration
    [Documentation]           Verify pull-up/pull-down register configuration
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    # Press button early to pass input test
    Execute Command           gpioPortA.UserButton Press
    Sleep                     0.1
    Execute Command           gpioPortA.UserButton Release

    # Note: Renode doesn't simulate internal pull resistors on floating pins
    Wait For Line On Uart     Pull register configuration: OK    timeout=10
    Wait For Line On Uart     Pull configuration test: PASS    timeout=5

Should Report Test Summary
    [Documentation]           Verify GPIO test completes successfully
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    # Press button to pass input test
    Execute Command           gpioPortA.UserButton Press
    Sleep                     0.1
    Execute Command           gpioPortA.UserButton Release

    Wait For Line On Uart     GPIO TEST PASSED    timeout=10
