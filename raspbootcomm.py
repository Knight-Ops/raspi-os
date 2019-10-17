import serial
import serial.tools.miniterm
import sys
from struct import pack

PORT = 'COM3'
BAUD = 115200

def listenForStart(ser):
    print("Listening for RBIN64")
    start = b''
    while True:
            if ser.in_waiting > 0:
                start += ser.read(1)
            
            if b'RBIN64\r\n' in start:
                print("Found RBIN64!")
                return

def listenForTrips(ser):
    print("Listening for trips")
    start = b''
    while True:
            if ser.in_waiting > 0:
                start += ser.read(1)
            
            if b'\x03\x03\x03' in start:
                print("Found Trips!")
                return

def listenForOk(ser):
    print("Listening for OK")
    start = b''
    while True:
            if ser.in_waiting > 0:
                start += ser.read(1)
            
            if b'OK' in start:
                print("Found OK!")
                return

def sendKernelSize(ser, kernelSize):
    print("Sending Kernel Size")
    size = pack("<I", kernelSize)
    ser.write(size)
    ser.flush()

def sendKernel(ser, kernel, kernelSize):
    print("Sending Kernel")
    sent = 0
    ser.write(kernel)
    ser.flush()

    
    print("Sent {} bytes of kernel".format(kernelSize))


def main():
    with serial.Serial(PORT, BAUD, timeout=0, parity=serial.PARITY_NONE, rtscts=False) as ser:
        listenForStart(ser)
        listenForTrips(ser)
        with open(sys.argv[1], 'rb') as f:
            kernel = f.read()
            kernelSize = len(kernel)
        sendKernelSize(ser, kernelSize)
        listenForOk(ser)
        sendKernel(ser, kernel, kernelSize)

    print("Kernel Loaded. Switching to Miniterm")

    with serial.serial_for_url(PORT, BAUD, timeout=0, parity=serial.PARITY_NONE, rtscts=False, do_not_open=True) as ser:
        if not hasattr(ser, 'cancel_read'):
            ser.timeout = 1

        mt = serial.tools.miniterm.Miniterm(ser, echo=False, eol='crlf')
        mt.exit_character = chr(0x1d)
        mt.menu_character = chr(0x14)
        mt.raw = False
        mt.set_rx_encoding('UTF-8')
        mt.set_tx_encoding('UTF-8')
        mt.start()
        try:
            mt.join(True)
        except KeyboardInterrupt:
            pass
        mt.join()
        mt.close()
   

if __name__ == "__main__":
    main()