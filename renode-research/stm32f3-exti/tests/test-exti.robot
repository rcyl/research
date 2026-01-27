*** Settings ***
Suite Setup                   Setup
Suite Teardown                Teardown
Test Setup                    Reset Emulation
Resource                      ${RENODEKEYWORDS}

*** Variables ***
${PLATFORM}                   ${CURDIR}/../stm32f3_exti.repl
${ELF}                        ${CURDIR}/../../target/thumbv7em-none-eabihf/release/stm32f3-exti

*** Test Cases ***
Should Initialize EXTI And Report
    [Documentation]           Verify EXTI initializes and reports on UART
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     EXTI Peripheral Test     timeout=5
    Wait For Line On Uart     EXTI0 configured         timeout=5

Should Detect Rising Edge Interrupt
    [Documentation]           Verify EXTI detects rising edge interrupt
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     Waiting for button press    timeout=5

    # Press button to generate rising edge
    Execute Command           gpioPortA.UserButton Press

    Wait For Line On Uart     Rising edge interrupt detected: PASS    timeout=5

Should Detect Falling Edge Interrupt
    [Documentation]           Verify EXTI detects falling edge interrupt
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     Waiting for button press    timeout=5

    # Press and release button
    Execute Command           gpioPortA.UserButton Press

    Wait For Line On Uart     Rising edge interrupt detected: PASS    timeout=5
    Wait For Line On Uart     Waiting for button release    timeout=5

    Execute Command           gpioPortA.UserButton Release

    Wait For Line On Uart     Falling edge interrupt detected: PASS    timeout=5

Should Count Multiple Interrupts
    [Documentation]           Verify EXTI counts multiple interrupts
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     Waiting for button press    timeout=5

    # First press/release
    Execute Command           gpioPortA.UserButton Press
    Wait For Line On Uart     Rising edge interrupt detected: PASS    timeout=5
    Execute Command           gpioPortA.UserButton Release
    Wait For Line On Uart     Falling edge interrupt detected: PASS    timeout=5

    # Second press/release
    Wait For Line On Uart     Press button 2 more times    timeout=5
    Execute Command           gpioPortA.UserButton Press
    Sleep                     0.1
    Execute Command           gpioPortA.UserButton Release
    Sleep                     0.1

    # Third press/release
    Execute Command           gpioPortA.UserButton Press
    Sleep                     0.1
    Execute Command           gpioPortA.UserButton Release

    Wait For Line On Uart     Multiple interrupt count: PASS    timeout=5

Should Report Test Summary
    [Documentation]           Verify EXTI test completes successfully
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     Waiting for button press    timeout=5

    # Generate all required interrupts
    # First press/release (for tests 1 and 2)
    Execute Command           gpioPortA.UserButton Press
    Sleep                     0.1
    Execute Command           gpioPortA.UserButton Release
    Sleep                     0.1

    # Two more press/release cycles (for test 3)
    Execute Command           gpioPortA.UserButton Press
    Sleep                     0.1
    Execute Command           gpioPortA.UserButton Release
    Sleep                     0.1
    Execute Command           gpioPortA.UserButton Press
    Sleep                     0.1
    Execute Command           gpioPortA.UserButton Release

    Wait For Line On Uart     EXTI TEST PASSED    timeout=10
