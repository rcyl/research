*** Settings ***
Suite Setup                   Setup
Suite Teardown                Teardown
Test Setup                    Reset Emulation
Resource                      ${RENODEKEYWORDS}

*** Variables ***
${PLATFORM}                   ${CURDIR}/../stm32f3_dma.repl
${ELF}                        ${CURDIR}/../target/thumbv7em-none-eabihf/release/stm32f3-dma

*** Test Cases ***
Should Initialize DMA And Report
    [Documentation]           Verify DMA test initializes and reports on UART
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     DMA Peripheral Test    timeout=5

Should Complete Memory To Memory Transfer
    [Documentation]           Verify DMA M2M transfer completes
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     Test 1: Memory-to-Memory Transfer    timeout=5
    Wait For Line On Uart     DMA transfer started                  timeout=5
    Wait For Line On Uart     Transfer complete flag: SET           timeout=10
    Wait For Line On Uart     Data verified: PASS                   timeout=5

Should Decrement NDTR To Zero
    [Documentation]           Verify NDTR register decrements to zero
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     Test 2: NDTR Register        timeout=10
    Wait For Line On Uart     NDTR is zero: PASS           timeout=5

Should Complete Second Transfer
    [Documentation]           Verify second DMA transfer works
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     Test 3: Second Transfer      timeout=15
    Wait For Line On Uart     Second transfer: PASS        timeout=5

Should Report Test Summary
    [Documentation]           Verify test summary shows results
    Execute Command           mach create
    Execute Command           machine LoadPlatformDescription @${PLATFORM}
    Execute Command           sysbus LoadELF @${ELF}

    Create Terminal Tester    sysbus.usart1

    Start Emulation

    Wait For Line On Uart     === Test Summary ===    timeout=20
    Wait For Line On Uart     DMA TEST                timeout=5
