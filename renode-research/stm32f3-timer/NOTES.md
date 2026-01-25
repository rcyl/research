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
- Update events trigger correctly
- PWM output capture may have limitations
