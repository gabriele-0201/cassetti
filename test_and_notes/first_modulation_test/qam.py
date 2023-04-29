import numpy as np
import random as rd
from scipy import signal
import matplotlib.pyplot as plt
from scipy.fftpack import fft, fftshift

# IDK where put this
signal_step_emulation = 0.001 # 1 milliseconds

def blob_of_bits(length):
    return [rd.randint(0, 1) for _ in range(length)]

def bits_to_integers(bits, n_bits):
    if len(bits) % n_bits != 0:
        raise Exception("Impossibel convert this bits to integers")

    conversion = []
    for b in range(0, len(bits), n_bits):
        # Assume little endian, the firs arrived bit is the leas significant
        index = 0
        for i in range(n_bits):
            index += bits[b + i] << i
        conversion.append(index)
        #print("associaed symbol " + str(index))

    return conversion

class MQAM:

    def __init__(self, freq, number_signal, base_signal):

        #if log - int(log) != 0:
        #   raise Exception("Number of signal must be a positive power of two")

        L = np.sqrt(number_signal)
        if L - int(L) != 0:
            raise Exception("Not accepted number of signals")
        self.L = int(L)

        # this should be always an even number
        self.bits_per_signal = int(np.log2(number_signal))

        self.number_signal = number_signal  
        self.samples = len(base_signal)
        self.signal_period = self.samples * signal_step_emulation
        self.costellation_index = []
        self.costellation_to_symbol = [[0 for _ in range(self.L)] for _ in range(self.L)]

        # In QAM modulation the points inside the costellation_index is np.sqrt(base_enargy/2)
        # distance each one
        self.base_signal_energy = np.sum(np.power(base_signal, 2))

        # coefficients will also be used in the demodulation for the minimum distance
        self.coefficients = np.array([float(2*l - self.L + 1) for l in range(self.L)])

        # For the demodulation will be used an array containing all the points
        # that construct the costellation_index, those are the same for x and y axis
        for i in range(self.L):
            self.costellation_index.append(self.coefficients[i] * np.sqrt(self.base_signal_energy/2))

        #print("Base energy signal: " + str(self.base_signal_energy))
        #print("Costellation" + str(self.costellation_index))

        # Costellation with the "mappatura di gray"
        # here will be associated each point in the costellation to a symbol (integer)

        # 0, 1
        # 00, 01, 11, 10
        # 000, 001, 011, 010, 100, 101, 111, 110
        values = ["0", "1"] # starting point for a 4qam

        for i in range(1, int(self.bits_per_signal/2)):
            new_values_0 = [ "0" + v for v in values]
            new_values_1 = [ "1" + v for v in values]
            values = new_values_0 + new_values_1[::-1]

        # from string rapresent bytes to int

        for i in range(self.L):
            for j in range(self.L):
                self.costellation_to_symbol[i][j] = values[i] + values[j]
        
        # This double map have the same form of the costellation
        # but the value inside is the integer symbol associated to a square inside 
        # the QAM grid
        self.costellation_to_symbol = np.array([np.array([int(b, 2) for b in self.costellation_to_symbol[i]]) for i in range(self.L)])
        #print("Costellation_to_symbol: \n" + str(self.costellation_to_symbol))

        time = np.linspace(0, self.signal_period, self.samples)

        cos = np.cos(2.0 * np.pi * freq * time)
        sin = np.sin(2.0 * np.pi * freq * time)

        self.base_x = np.sqrt(2/self.base_signal_energy) * cos * base_signal
        self.base_y = np.sqrt(2/self.base_signal_energy) * sin * base_signal

        #print("base signal: " + str(base_signal))
        #print("cos: " + str(cos))
        #print("sin: " + str(sin))
        #print("self.coefficients: " + str(self.coefficients))

        # coefficients goes from lower to upper
        
        # [-3, -1, 1, 3]

        # [[ 0  1  3  2]
        #  [ 4  5  7  6]
        #  [12 13 15 14]
        #  [ 8  9 11 10]]

        self.symbol_to_fun = {}
        for i in range(self.L):
            for j in range(self.L):
                signal = self.coefficients[i] * cos * base_signal - self.coefficients[j] * sin * base_signal

                # theorically the index in the costellation and in the self.coefficients is the same
                # this value will be used as key int the transation from bits to the associed symbol
                associed_int = self.costellation_to_symbol[i][j]
                #print("coeff i: " + str(i) + " coeff j: " + str(j) + " -> associed int" + str(associed_int))

                self.symbol_to_fun[associed_int] =  signal

        
        # print all the QAM signals
        '''
        figure, axis = plt.subplots(self.L, self.L)
        for i in range(self.L):
            for j in range(self.L):
                axis[i, j].plot(self.symbol_to_fun[(i*self.L) + j])
        plt.show()
        '''

    # array of bits as input
    # function as output
    def module(self, bits):

        if len(bits) % self.bits_per_signal != 0:
            raise Exception("Bits must be a multiple of " + str(self.bits_per_signal))

        number_simbols = int(len(bits)/self.bits_per_signal)

        time = np.linspace(0, number_simbols * self.signal_period, self.samples * number_simbols)
        val = np.zeros(0)

        symbol_to_send = bits_to_integers(bits, self.bits_per_signal)

        for s in symbol_to_send:
            val = np.append(val, self.symbol_to_fun.get(s))

        return time, val
    
    def costellation(self):
        cost_x = []
        cost_y = []

        for i in range(self.L):
            for j in range(self.L):
                cost_x.append(self.costellation_index[i]) 
                cost_y.append(self.costellation_index[j]) 

        #print("x: "+str(cost_x))
        #print("y: "+str(cost_y))
    
        return cost_x, cost_y

    # Accept the data to demodule
    # for now return a simple vec with all the interanl product
    def demod(self, data):

        demod = lambda base : np.array([
            np.sum( base * data[t_start: t_start + self.samples])
            for t_start in np.arange(0, len(data), self.samples)
        ])

        demod_x = demod(self.base_x)
        demod_y = demod(self.base_y)

        # third param will be the array of demodulated integers
        demodulated = []
        for i in range(len(demod_x)): # should be equal using demod_y
            # the result of the internal product will be used in the minimum distance
            # the minimum distance will be done with all the values in the costellation

            # for the x axis the index of the nearest value in the costellation_index 
            #is equal to the associated symbol in the costellation_to_symbol map

            # for the y axis is a little bit different because the costellation_index is reverted
            # the first value in the array is the opposite value in the costellation, 
            # for this I have to revert the array

            index_row = min_distance(self.costellation_index, demod_x[i])
            index_column = min_distance(self.costellation_index[::-1], demod_y[i])
            #print("singal " + str(i) + " -> " + "[" + str(index_row) +"]" + "[" + str(index_column) +"]")
            demodulated.append(self.costellation_to_symbol[index_row][index_column])

        return  demod_x, demod_y, demodulated

# Will iterate over the array data and return the index of the nearest element
# in the array to value
def min_distance(data, value):
    distances = []
    for v in data:
        distances.append(np.abs(v - value))

    #print("distances: " + str(distances))

    min = distances[0]
    min_index = 0
    for i in range(1, len(distances)):
        if min > distances[i]:
            min = distances[i]
            min_index = i

    return min_index 

        

def add_noise(data, variance):

    noise = np.random.normal(0,np.sqrt(variance),len(data))

    return data + noise

# MODULATION
freq = 1000 #Hz
base_signal = np.zeros(int(1 / signal_step_emulation)) + 1

number_qam = 64
symbol_to_send = 10
variance = 5

# Based on input
print("M_QAM, M =  ")
if number_qam == 0:
    number_qam = int(input())
print("Number of symbol to send: ")
if symbol_to_send == 0:
    symbol_to_send  = int(input())
print("Noise variance: ")
if variance == 0:
    variance = float(input())


# Create the modulation class
qam = MQAM(freq, number_qam, base_signal)
bits_to_send = blob_of_bits(qam.bits_per_signal * symbol_to_send)
int_to_send = bits_to_integers(bits_to_send, qam.bits_per_signal)

# module the bits
x,y_qam = qam.module(bits_to_send)

# Add noise
y_qam_with_noise = add_noise(y_qam, variance)

# Get the costellation
x_cost, y_cost = qam.costellation()
x_demod, y_demod, demod_array = qam.demod(y_qam_with_noise)

# Applay minimum distance to demodule

# Plot everithing
figure, axis = plt.subplots(2, 2)

# Modulation
axis[0, 0].plot(x, y_qam)
if len(int_to_send) > 10:
    axis[0, 0].set_title(str(number_qam) + "_QAM, input " + str(len(int_to_send)) + " symbols")
else:
    axis[0, 0].set_title(str(number_qam) + "_QAM, input: " + str(int_to_send))

# Modulation with noise
axis[0, 1].plot(x, y_qam_with_noise)
axis[0, 1].set_title(str(number_qam) + "_QAM with noise, variance: " + str(variance))

# MOVE X and Y AXIS to CENTER 
axis[1, 0].spines['left'].set_position('center')
axis[1, 0].spines['bottom'].set_position('center')
# Eliminate others
axis[1, 0].spines['right'].set_color('none')
axis[1, 0].spines['top'].set_color('none')

# Show ticks in the left and lower axes only
axis[1, 0].xaxis.set_ticks_position('bottom')
axis[1, 0].yaxis.set_ticks_position('left')

axis[1, 0].plot(x_demod, y_demod, color='r', marker = ".", linestyle='None', markersize = 10.0)
axis[1, 0].plot(x_cost, y_cost, color='b', marker = ".", linestyle='None', markersize = 10.0)
if len(demod_array) > 10:
    axis[1, 0].set_title(str(number_qam) + "_QAM demodulation" + str(len(demod_array)) + " symbols")
else:
    axis[1, 0].set_title(str(number_qam) + "_QAM demodulation: " + str(demod_array))

plt.show()
