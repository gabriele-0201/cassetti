import numpy as np
import matplotlib.pyplot as plt
import qam as mqam

# CONSTS
signal_step_emulation = 0.001 # 1 milliseconds

LOWER_BOUND_VARIANCE = 150
UPPER_BOUND_VARIANCE = 200
STEP_VARIANCE = 0.01
SIGNAL_PERIOD = 1
FREQ = 1000 # Hz

M = 16

def get_snr_and_p_error(blob_of_bits_dim):

    base_signal = np.zeros(int(1 / signal_step_emulation)) + 1

    qam = mqam.MQAM(SIGNAL_PERIOD, M, base_signal)
    source_bits = mqam.blob_of_bits(blob_of_bits_dim)
    source_integers = mqam.bits_to_integers(source_bits, qam.bits_per_signal)
    dim_integers = len(source_integers)
    x,y_qam = qam.module(source_bits)
    
    x_variance = []
    y_perror = []
    
    for variance in np.arange(LOWER_BOUND_VARIANCE, UPPER_BOUND_VARIANCE, STEP_VARIANCE):
        y_qam_with_noise = mqam.add_noise(y_qam, variance)
        x_demod, y_demod, received_integers = qam.demod(y_qam_with_noise)
    
        # P(Error) = errors / BPSK_BLOB_BITS_DIMENSIONA
        errors = 0
        for i in range(dim_integers):
            if source_integers[i] != received_integers[i]:
                errors = errors + 1
    
        #print("variance: " + str(variance) + " errors: " + str(errors))
        
        # Energe of the qam = 1
        x_variance.append(1 / variance)
        y_perror.append(errors / dim_integers)

    return x_variance, y_perror

print("want to display")


n_bit_M = int(np.log2(M))

#x_100, y_100 = get_snr_and_p_error(n_bit_M * 100)
#print("100 done")
x_1000, y_1000 = get_snr_and_p_error(n_bit_M * 1000)
print("1000 done")
#x_10000, y_10000 = get_snr_and_p_error(n_bit_M * 10000)
#print("10000 done")
#x_100000, y_100000 = get_snr_and_p_error(100000)
#print("100000 done")

figure, axis = plt.subplots(2, 2)

#axis[0, 0].plot(x_100, y_100)
#axis[0, 0].xlabel("SNR") # Energy_signal / variance
#axis[0, 0].ylabel("P_ERROR")
#axis[0, 0].set_title("100 bits")

axis[0, 1].plot(x_1000, y_1000)
##axis[0, 1].xlabel("SNR") # Energy_signal / variance
##axis[0, 1].ylabel("P_ERROR")
axis[0, 1].set_title("1000 bits")

#axis[1, 0].plot(x_10000, y_10000)
##axis[1, 0].xlabel("SNR") # Energy_signal / variance
##axis[1, 0].ylabel("P_ERROR")
#axis[1, 0].set_title("10000 bits")

#axis[1, 1].plot(x_100000, y_100000)
#axis[1, 1].xlabel("SNR") # Energy_signal / variance
#axis[1, 1].ylabel("P_ERROR")
#axis[1, 1].set_title("100000 bits")

plt.show()
