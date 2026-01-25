# STM32F3 DMA Development Notes

## Renode DMA Support

Renode provides DMA emulation via `DMA.STM32DMA` peripheral.

## STM32F3 DMA Configuration

### DMA Controllers
- DMA1: 7 channels (general purpose)
- DMA2: 5 channels (used with ADC, timers)

### DMA1 Channel Mapping
| Channel | Peripheral Options |
|---------|-------------------|
| CH1 | ADC1, TIM2_CH3, TIM4_CH1 |
| CH2 | SPI1_RX, USART3_TX, TIM2_UP |
| CH3 | SPI1_TX, USART3_RX, TIM4_CH2 |
| CH4 | SPI2_RX, USART1_TX, I2C2_TX |
| CH5 | SPI2_TX, USART1_RX, I2C2_RX |
| CH6 | USART2_RX, I2C1_TX, TIM3_CH1 |
| CH7 | USART2_TX, I2C1_RX, TIM4_CH3 |

## DMA Register Configuration

### Channel Registers
| Register | Description |
|----------|-------------|
| CCR | Channel configuration |
| CNDTR | Number of data to transfer |
| CPAR | Peripheral address |
| CMAR | Memory address |

### CCR Bits
- MEM2MEM: Memory-to-memory mode
- PL[1:0]: Priority level
- MSIZE[1:0]: Memory data size
- PSIZE[1:0]: Peripheral data size
- MINC: Memory increment mode
- PINC: Peripheral increment mode
- CIRC: Circular mode
- DIR: Data transfer direction
- EN: Channel enable

### Status/Clear Registers
| Register | Description |
|----------|-------------|
| ISR | Interrupt status (GIF, TCIF, HTIF, TEIF per channel) |
| IFCR | Interrupt flag clear |

## Memory-to-Memory Mode

```rust
dma1.ch1.cr.write(|w| {
    w.mem2mem().enabled()   // Enable M2M
     .pl().high()           // High priority
     .msize().bits8()       // 8-bit memory
     .psize().bits8()       // 8-bit peripheral
     .minc().enabled()      // Memory increment
     .pinc().enabled()      // Peripheral increment
     .dir().from_peripheral()
     .en().enabled()
});
```

## Important Notes

- CCM memory (0x10000000) is NOT accessible by DMA
- Must use main SRAM (0x20000000) for DMA buffers
- Clear interrupt flags before starting new transfer
- Disable channel before reconfiguring
