# Renode Research Notes

## Project Goal
Follow the Interrupt Memfault tutorial on Renode to completion:
https://interrupt.memfault.com/blog/intro-to-renode

## Progress Log

### Session 1 - Setup and Testing

**What is Renode?**
- Open-source emulator for embedded systems from Antmicro
- Supports Cortex-M and other ARM architectures
- Better embedded focus than QEMU (more Cortex-M targets)
- Includes GDB server integration
- Robot Framework for automated testing

**Environment Setup:**
- Platform: Debian Bookworm on ARM64 (aarch64)
- Renode Version: 1.16.0 (ARM64 .NET portable version)
- ARM Toolchain: gcc-arm-none-eabi
- GDB: gdb-multiarch

**Steps Completed:**

1. **Installed Renode 1.16.0** (ARM64 portable dotnet version)
   - Downloaded from https://github.com/renode/renode/releases
   - Extracted to /opt/renode_1.16.0-dotnet_portable

2. **Cloned example repository**
   - Repository: https://github.com/memfault/interrupt
   - Example location: interrupt/example/renode/

3. **Built STM32F4 Discovery firmware**
   - Built libopencm3 library (commit 89074d6a)
   - Compiled renode-example.c with arm-none-eabi-gcc
   - Generated renode-example.elf (243KB)

4. **Ran emulation successfully**
   - UART2 output: "hello world!"
   - GDB server started on port 3333

5. **Tested GDB debugging**
   - Connected with gdb-multiarch
   - Retrieved registers, backtrace
   - Breakpoints work correctly

6. **Tested button press functionality**
   - Used sysbus.gpioPortA.UserButton Press/Release
   - UART2 output: "button pressed"

**Key Configuration Files:**

- `renode-config.resc`: Main Renode script
  - Loads STM32F4 Discovery platform
  - Adds CCM memory region
  - Starts GDB server on port 3333

- `add-ccm.repl`: CCM memory fix
  - Maps 64KB at 0x10000000 (required for STM32F4)

**Issues Encountered:**

1. ARM64 platform required specific Renode version (1.16.0+)
2. Python symlink needed (python -> python3)
3. Robot Framework tests require Mono version of Renode (not .NET portable)

**Commands Reference:**

```bash
# Run Renode with script
/opt/renode_1.16.0-dotnet_portable/renode --console --disable-xwt script.resc

# Start GDB session
gdb-multiarch firmware.elf -ex "target remote :3333"

# Enable function tracing
sysbus.cpu LogFunctionNames True
logFile @/tmp/function-trace.log

# Simulate button press/release
sysbus.gpioPortA.UserButton Press
sysbus.gpioPortA.UserButton Release
```

---

### Session 2 - Rust Port

**Ported the C UART example to Rust**

**Rust Environment:**
- Rust: 1.93.0
- Target: thumbv7em-none-eabihf (Cortex-M4F with hardware FPU)
- HAL: stm32f4xx-hal v0.21

**Dependencies:**
```toml
cortex-m = { version = "0.7", features = ["critical-section-single-core"] }
cortex-m-rt = "0.7"
panic-halt = "0.2"
stm32f4xx-hal = { version = "0.21", features = ["stm32f407"] }
```

**Binary Size Comparison:**

| Version | Text | Data | BSS | Total |
|---------|------|------|-----|-------|
| C (libopencm3) | 4564 | 108 | 16 | 4688 bytes |
| Rust (stm32f4xx-hal) | 1252 | 0 | 4 | 1256 bytes |

Rust version is **73% smaller** due to:
- Zero-cost abstractions
- Type-level state machines for GPIO (unused code eliminated at compile time)
- No libc overhead
- LTO (Link-Time Optimization) enabled

**Key Differences from C:**

1. **No manual clock enable** - HAL handles RCC automatically when peripherals are split
2. **Type-safe GPIO** - Can't accidentally use wrong pin mode
3. **Ownership model** - Peripherals can only be used once
4. **No printf** - Used custom `uart_write_str` function

**Build Commands:**
```bash
# Build Rust firmware
cd rust-uart
cargo build --release

# Run in Renode
/opt/renode_1.16.0-dotnet_portable/renode --console --disable-xwt test-button.resc
```

**Test Results:**
- UART output: "hello world!" on startup
- UART output: "button pressed" after button release
- Identical behavior to C version

---

### Session 3 - STM32F3 Platform Support

**Created custom Renode platform for STM32F303 (not natively supported)**

**Challenge:** STM32F3 series not included in Renode's built-in platforms

**Solution:** Created custom platform definition files:
- `stm32f3.repl` - CPU/peripheral definitions
- `stm32f3_discovery.repl` - Board with button/LED

**Key STM32F3 vs STM32F4 Differences:**

| Feature | STM32F4 | STM32F3 |
|---------|---------|---------|
| GPIO Base | 0x40020000 (AHB1) | 0x48000000 (AHB2) |
| USART Type | STM32_UART | STM32F7_USART |
| CCM Size | 64KB | 8KB |
| RCC Address | 0x40023800 | 0x40021000 |

**Platform Components Defined:**
- Cortex-M4F CPU
- 256KB Flash, 40KB SRAM, 8KB CCM
- GPIO ports A-F at AHB2 addresses
- USART1/2/3 using STM32F7_USART model
- NVIC with correct interrupt numbers
- Simplified RCC via Python peripheral

**Testing Approach:**
1. First created minimal C test (320 bytes) using raw registers
2. Verified platform works in Renode
3. Then ported to Rust with stm32f3xx-hal

**Binary Sizes (STM32F3):**

| Version | Text | Data | BSS | Total |
|---------|------|------|-----|-------|
| C (bare metal) | 256 | 64 | 0 | 320 bytes |
| Rust (stm32f3xx-hal) | 1296 | 0 | 4 | 1300 bytes |

**Test Results:**
- Both C and Rust versions work correctly
- USART1 output: "hello world!"
- Button press detection works
- Platform successfully added to Renode

**Files Created:**
```
stm32f3-uart/
├── stm32f3.repl           # Platform definition
├── stm32f3_discovery.repl # Board definition
├── src/main.rs            # Rust firmware
├── test-c/main.c          # C test firmware
└── README.md              # Documentation
```
