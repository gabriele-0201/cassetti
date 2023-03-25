import numpy as np
import matplotlib.pyplot as plt
import bpsk_modulation as bpskM

# CONSTS
LOWER_BOUND_VARIANCE = 0.05
UPPER_BOUND_VARIANCE = 1.5
STEP_VARIANCE = 0.05
SIGNAL_PERIOD = 1
FREQ = 1000 # Hz

def get_snr_and_p_error(blob_of_bits_dim):

    bpsk = bpskM.BPSK(SIGNAL_PERIOD, FREQ)
    source_bits = bpskM.blob_of_bits(blob_of_bits_dim)
    x,y_bpsk = bpsk.module(source_bits)
    
    x_variance = []
    y_perror = []
    
    for variance in np.arange(LOWER_BOUND_VARIANCE, UPPER_BOUND_VARIANCE, STEP_VARIANCE):
        y_bpsk_with_noise = bpskM.add_noise(y_bpsk, variance)
        costellation = bpsk.costellation(y_bpsk_with_noise)
        receiver_bits = bpsk.demodule(costellation)
    
        # P(Error) = errors / BPSK_BLOB_BITS_DIMENSIONA
        errors = 0
        for i in range(blob_of_bits_dim):
            if source_bits[i] != receiver_bits[i]:
                errors = errors + 1
    
        #print("variance: " + str(variance) + " errors: " + str(errors))
        
        # Energe of the bpsk = 1
        x_variance.append(1 / variance)
        y_perror.append(errors / blob_of_bits_dim)

    return x_variance, y_perror


x_100, y_100 = get_snr_and_p_error(100)
print("100 done")
x_1000, y_1000 = get_snr_and_p_error(1000)
print("1000 done")
x_10000, y_10000 = get_snr_and_p_error(10000)
print("10000 done")
#x_100000, y_100000 = get_snr_and_p_error(100000)
#print("100000 done")

figure, axis = plt.subplots(2, 2)

axis[0, 0].plot(x_100, y_100)
#axis[0, 0].xlabel("SNR") # Energy_signal / variance
#axis[0, 0].ylabel("P_ERROR")
axis[0, 0].set_title("100 bits")

axis[0, 1].plot(x_1000, y_1000)
#axis[0, 1].xlabel("SNR") # Energy_signal / variance
#axis[0, 1].ylabel("P_ERROR")
axis[0, 1].set_title("1000 bits")

axis[1, 0].plot(x_10000, y_10000)
#axis[1, 0].xlabel("SNR") # Energy_signal / variance
#axis[1, 0].ylabel("P_ERROR")
axis[1, 0].set_title("10000 bits")

#axis[1, 1].plot(x_100000, y_100000)
#axis[1, 1].xlabel("SNR") # Energy_signal / variance
#axis[1, 1].ylabel("P_ERROR")
#axis[1, 1].set_title("100000 bits")

plt.show()
