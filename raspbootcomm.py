import serial
import sys
from struct import pack

def listenForStart(ser):
    print("Listening for RBIN64")
    start = b''
    while True:
            if ser.in_waiting > 0:
                start += ser.read(1)
            
            if start == b'RBIN64\r\n':
                print("Found RBIN64!")
                return

def listenForTrips(ser):
    print("Listening for trips")
    start = b''
    while True:
            if ser.in_waiting > 0:
                start += ser.read(1)
            
            if start == b'\x03\x03\x03':
                print("Found Trips!")
                return

def listenForOk(ser):
    print("Listening for OK")
    start = b''
    while True:
            if ser.in_waiting > 0:
                start += ser.read(1)
            
            if start == b'OK':
                print("Found OK!")
                return

def sendKernelSize(ser, kernelSize):
    print("Sending Kernel Size")
    size = pack("<I", kernelSize)
    ser.write(size)
    ser.flush()
    # for ea in size:
    #     ser.write(ea)

def sendKernel(ser, kernel, kernelSize):
    print("Sending Kernel")
    sent = 0
    ser.write(kernel)
    ser.flush()
    # for ea in kernel:
    #     ser.write(ea)
    #     if sent % 100 == 0 :
    #         print("Sent : {}/{}\nIn waiting: {}".format(sent, kernelSize, ser.read(ser.in_waiting)), end= '\r')
    #     sent += 1
    
    print("Sent {} bytes of kernel".format(kernelSize))


def main():
    # with open(sys.argv[1], 'rb') as f:
    #     kernel = f.read()
    #     kernelSize = len(kernel) 
    
    # print("Kernel size is : {:X}".format(kernelSize))
    with serial.Serial('COM3', 115200, timeout=0, parity=serial.PARITY_NONE, rtscts=False) as ser:
        listenForStart(ser)
        listenForTrips(ser)
        with open(sys.argv[1], 'rb') as f:
            kernel = f.read()
            kernelSize = len(kernel)
        print("Kernel size is : {:X}".format(kernelSize))
        sendKernelSize(ser, kernelSize)
        listenForOk(ser)
        sendKernel(ser, kernel, kernelSize)

        # data = b''
        # while True:
        #     if ser.in_waiting > 0:
        #         data += ser.read(ser.in_waiting)
        #         print(data, end='\r')

    print("Kernel Loaded.")
        

if __name__ == "__main__":
    main()