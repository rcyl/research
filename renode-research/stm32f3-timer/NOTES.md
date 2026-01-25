# STM32F3 Timer Development Notes

## Renode Timer Support

Renode provides timer emulation via `Timers.STM32_Timer` peripheral.

## STM32F3 Timer Configuration

### Available Timers
- TIM1: Advanced control timer (16-bit)
- TIM2: General purpose (32-bit)
- TIM3, TIM4: General purpose (16-bit)
- TIM6, TIM7: Basic timers
- TIM8: Advanced control timer
- TIM15, TIM16, TIM17: General purpose

### Timer Clock
- APB1 timers: TIM2, TIM3, TIM4, TIM6, TIM7
- APB2 timers: TIM1, TIM8, TIM15, TIM16, TIM17
- Default clock: 72MHz (HSI x PLL)

## HAL Timer API

```rust
use stm32f3xx_hal::timer::Timer;

// Create timer
let mut timer = Timer::new(dp.TIM2, clocks, &mut rcc.apb1);

// Start countdown
timer.start(100.millis());

// Wait for expiry (blocking)
nb::block!(timer.wait()).ok();

// Or poll
while timer.wait().is_err() {
    // Still counting
}
```

## Direct Register Access

```rust
let tim4 = unsafe { &*pac::TIM4::ptr() };
tim4.psc.write(|w| unsafe { w.psc().bits(7999) }); // Prescaler
tim4.arr.write(|w| unsafe { w.bits(0xFFFF) });     // Auto-reload
tim4.cr1.write(|w| w.cen().enabled());             // Enable
let count = tim4.cnt.read().bits();                // Read counter
```

## Key Registers

| Register | Description |
|----------|-------------|
| CR1 | Control register (CEN = counter enable) |
| PSC | Prescaler (divides input clock) |
| ARR | Auto-reload register (period) |
| CNT | Counter value |
| SR | Status register (UIF = update interrupt flag) |

## Renode Timer Behavior

- Timers count based on virtual time
- Prescaler and ARR work as expected
- Counter increments and wraps correctly
- PWM output capture may have limitations

### Known Issue: UIF Flag Not Set

The HAL's `timer.wait()` method polls the Update Interrupt Flag (UIF) in the status register to detect timer expiration. Renode's `Timers.STM32_Timer` emulation does not properly set this flag when the counter overflows, causing `wait()` to never return `Ok`.

**Symptom:** Code using `nb::block!(timer.wait())` or polling `timer.wait().is_err()` will hang indefinitely in Renode.

### Workaround: Counter Wrap-Around Detection

Instead of polling the UIF flag, poll the counter register directly and detect wrap-around:

```rust
let tim2 = unsafe { &*pac::TIM2::ptr() };

// Configure timer
tim2.psc.write(|w| w.psc().bits(7199));  // 72MHz / 7200 = 10kHz
tim2.arr.write(|w| w.bits(999));          // 1000 ticks = 100ms
tim2.cnt.write(|w| w.bits(0));

// Generate update event to load prescaler
tim2.egr.write(|w| w.ug().update());
tim2.sr.write(|w| w.uif().clear_bit());

// Enable counter
tim2.cr1.write(|w| w.cen().enabled());

// Detect wrap-around (counter resets to 0 after reaching ARR)
let arr_val = tim2.arr.read().bits();
let mut last_cnt: u32 = 0;

loop {
    let cnt = tim2.cnt.read().bits();

    // Counter was high, now low = wrapped
    if cnt < last_cnt && last_cnt > (arr_val / 2) {
        break; // Timer expired
    }

    last_cnt = cnt;
}
```

This approach works in both Renode emulation and on real hardware.
