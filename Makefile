release.bin:
	cargo build --target=thumbv7m-none-eabi --release
	arm-none-eabi-objcopy -O binary target/thumbv7m-none-eabi/release/bluepill_spidemo $@

debug.bin:
	cargo build --target=thumbv7m-none-eabi
	arm-none-eabi-objcopy -O binary target/thumbv7m-none-eabi/debug/bluepill_spidemo $@

flash.%: %.bin
	stm32flash /dev/ttyUSB0 -b115200 -w $<

.PHONY: release.bin debug.bin

tty:
	python /usr/lib/python3.9/site-packages/serial/tools/miniterm.py /dev/ttyUSB0 115200
