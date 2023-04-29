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


x_100000, y_100000 = get_snr_and_p_error(100000)
print("100000 done")

plt.plot(x_100000, y_100000)
plt.xlabel("SNR") # Energy_signal / variance
plt.ylabel("P_ERROR")
plt.set_title("100 bits")

plt.show()
