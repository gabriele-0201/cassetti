import numpy as np
import random as rd
from scipy import signal
import matplotlib.pyplot as plt
from scipy.fftpack import fft, fftshift

# IDK where put this
signal_step_emulation = 0.001 # 1 milliseconds

def blob_of_bits(length):
    return [rd.randint(0, 1) for _ in range(length)]

class BPSK:

    def __init__(self, signal_period, freq):
        self.signal_period = signal_period
        self.samples = int(signal_period / signal_step_emulation)
        
        time = np.linspace(0, signal_period, self.samples)
        cos = np.cos(2.0 * np.pi * freq * time)

        self.symbol_to_fun = { 
            0: cos,
            1: (cos * -1)
        }


    # array of bits as input
    # function as output
    def module(self, bits):
        time = np.linspace(0, len(bits) * self.signal_period, self.samples * len(bits))
        val = np.zeros(0)

        for b in bits:
            #print(str(b) + "-> " + str(self.symbol_to_fun.get(b)))
            val = np.append(val, self.symbol_to_fun.get(b))

        #print("time : " + str(time))
        #print("val : " + str(val))
        return time, val
    
    # Accept the data to demodule
    # for now return a simple vec with all the interanl product
    def costellation(self, data):
        # maybe there is a better way to manage time
        # time = np.linspace(0, len(data), self.samples)

        #for t_end in np.arange(o, len(data), self.signal_period):

        base = self.symbol_to_fun.get(0)

        # this is completely broken
        return np.array([
                np.sum( base * data[t_start: t_start + self.signal_period])  # + 1 because the upper range is exlusive
                for t_start in np.arange(0, len(data), self.signal_period * self.samples)
            ])
        
    def demodule(self, costllation):
        return [ 0 if d > 0 else 1 for d in costllation]

def add_noise(data, variance):

    noise = np.random.normal(0,np.sqrt(variance),len(data))

    return data + noise
