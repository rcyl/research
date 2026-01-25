CC = arm-none-eabi-gcc
CFLAGS = -mcpu=cortex-m4 -mthumb -mfloat-abi=hard -mfpu=fpv4-sp-d16 \
         -Os -g -Wall -ffunction-sections -fdata-sections
LDFLAGS = -T link.ld -nostartfiles -Wl,--gc-sections

all: test-f3.elf

test-f3.elf: main.c link.ld
	$(CC) $(CFLAGS) $(LDFLAGS) -o $@ main.c

clean:
	rm -f test-f3.elf

.PHONY: all clean
