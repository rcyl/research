// Minimal STM32F3 UART test for Renode
// Uses raw register access to test platform compatibility

#include <stdint.h>

// Register base addresses for STM32F303
#define RCC_BASE        0x40021000
#define GPIOA_BASE      0x48000000
#define GPIOE_BASE      0x48001000
#define USART1_BASE     0x40013800

// RCC registers
#define RCC_AHBENR      (*(volatile uint32_t *)(RCC_BASE + 0x14))
#define RCC_APB2ENR     (*(volatile uint32_t *)(RCC_BASE + 0x18))

// RCC enable bits
#define RCC_AHBENR_GPIOAEN  (1 << 17)
#define RCC_AHBENR_GPIOEEN  (1 << 21)
#define RCC_APB2ENR_USART1EN (1 << 14)

// GPIO registers (STM32F3 GPIO at AHB2)
#define GPIOA_MODER     (*(volatile uint32_t *)(GPIOA_BASE + 0x00))
#define GPIOA_AFRL      (*(volatile uint32_t *)(GPIOA_BASE + 0x20))
#define GPIOA_IDR       (*(volatile uint32_t *)(GPIOA_BASE + 0x10))
#define GPIOE_MODER     (*(volatile uint32_t *)(GPIOE_BASE + 0x00))
#define GPIOE_ODR       (*(volatile uint32_t *)(GPIOE_BASE + 0x14))

// USART1 registers (STM32F3 uses newer USART with different register layout)
#define USART1_CR1      (*(volatile uint32_t *)(USART1_BASE + 0x00))
#define USART1_CR2      (*(volatile uint32_t *)(USART1_BASE + 0x04))
#define USART1_CR3      (*(volatile uint32_t *)(USART1_BASE + 0x08))
#define USART1_BRR      (*(volatile uint32_t *)(USART1_BASE + 0x0C))
#define USART1_ISR      (*(volatile uint32_t *)(USART1_BASE + 0x1C))
#define USART1_TDR      (*(volatile uint32_t *)(USART1_BASE + 0x28))

// USART bits
#define USART_CR1_UE    (1 << 0)   // USART enable
#define USART_CR1_TE    (1 << 3)   // Transmitter enable
#define USART_ISR_TXE   (1 << 7)   // Transmit data register empty

void uart_putc(char c) {
    while (!(USART1_ISR & USART_ISR_TXE));
    USART1_TDR = c;
}

void uart_puts(const char *s) {
    while (*s) {
        if (*s == '\n') {
            uart_putc('\r');
        }
        uart_putc(*s++);
    }
}

int main(void) {
    // Enable clocks for GPIOA, GPIOE, and USART1
    RCC_AHBENR |= RCC_AHBENR_GPIOAEN | RCC_AHBENR_GPIOEEN;
    RCC_APB2ENR |= RCC_APB2ENR_USART1EN;

    // Configure PA9 (USART1_TX) as alternate function (AF7)
    // MODER: 10 = Alternate function mode
    GPIOA_MODER &= ~(3 << (9 * 2));
    GPIOA_MODER |= (2 << (9 * 2));
    // AFR[1] (AFRH) for PA9: AF7 = 0x7
    // PA9 is in AFRH (pins 8-15), bit position = (9-8)*4 = 4
    volatile uint32_t *GPIOA_AFRH = (volatile uint32_t *)(GPIOA_BASE + 0x24);
    *GPIOA_AFRH &= ~(0xF << 4);
    *GPIOA_AFRH |= (7 << 4);

    // Configure PE9 (LED) as output
    GPIOE_MODER &= ~(3 << (9 * 2));
    GPIOE_MODER |= (1 << (9 * 2));

    // Configure USART1: 115200 baud @ 8MHz HSI
    // BRR = 8000000 / 115200 = 69 (0x45)
    USART1_BRR = 69;
    USART1_CR1 = USART_CR1_TE | USART_CR1_UE;

    // Print hello world
    uart_puts("hello world!\n");

    // Toggle LED
    GPIOE_ODR |= (1 << 9);

    // Button handling
    int button_is_pressed = 0;

    while (1) {
        int button_state = (GPIOA_IDR & 1);  // PA0

        if (!button_is_pressed && button_state) {
            button_is_pressed = 1;
        } else if (button_is_pressed && !button_state) {
            uart_puts("button pressed\n");
            button_is_pressed = 0;
            // Toggle LED
            GPIOE_ODR ^= (1 << 9);
        }
    }

    return 0;
}

// Startup code
void Reset_Handler(void);
void Default_Handler(void);

// Stack pointer (defined by linker)
extern uint32_t _estack;

// Vector table
__attribute__((section(".isr_vector")))
const void *vector_table[] = {
    &_estack,           // Initial stack pointer
    Reset_Handler,      // Reset handler
    Default_Handler,    // NMI
    Default_Handler,    // HardFault
    Default_Handler,    // MemManage
    Default_Handler,    // BusFault
    Default_Handler,    // UsageFault
    0, 0, 0, 0,         // Reserved
    Default_Handler,    // SVCall
    Default_Handler,    // Debug Monitor
    0,                  // Reserved
    Default_Handler,    // PendSV
    Default_Handler,    // SysTick
};

void Reset_Handler(void) {
    main();
    while (1);
}

void Default_Handler(void) {
    while (1);
}
