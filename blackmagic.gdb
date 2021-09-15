target extended-remote /dev/serial/by-id/usb-Black_Sphere_Technologies_Black_Magic_Probe_v1.7.1-121-gb5e4653_79A75FA8-if00

monitor jtag_scan
attach 1

set mem inaccessible-by-default off

# setup a command to build and load the executable
#define build
#shell cargo build --release
#load target/thumbv6m-none-eabi/release/threshpan
#run
#end
#
#define buildd
#shell cargo build
#load target/thumbv6m-none-eabi/debug/threshpan
#run
#end

# print demangled symbols
set print asm-demangle on

# detect unhandled exceptions, hard faults and panics
#break DefaultHandler
#break HardFault
#break rust_begin_unwind

# *try* to stop at the user entry point (it might be gone due to inlining)
#break main

#monitor arm semihosting enable

# # send captured ITM to the file itm.fifo
# # (the microcontroller SWO pin must be connected to the programmer SWO pin)
# # 8000000 must match the core clock frequency
# monitor tpiu config internal itm.txt uart off 8000000

# # OR: make the microcontroller SWO pin output compatible with UART (8N1)
# # 8000000 must match the core clock frequency
# # 2000000 is the frequency of the SWO pin
# monitor tpiu config external uart off 8000000 2000000

# # enable ITM port 0
# monitor itm port 0 on

load

# start the process but immediately halt the processor
stepi
