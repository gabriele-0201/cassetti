import matplotlib.pyplot as plt
import bpsk_modulation as bpskM

signal_period = 1
freq = 1000 # Hz

bpsk = bpskM.BPSK(signal_period, freq)
bits = bpskM.blob_of_bits(10)
x,y_bpsk = bpsk.module(bits)
variance = 0.3
y_bpsk_with_noise = bpskM.add_noise(y_bpsk, variance)

x_costellation = bpsk.costellation(y_bpsk_with_noise)
y_costellation = [0 for _ in range(len(x_costellation))] 
x_dem = bpsk.demodule(x_costellation)
y_dem = y_costellation

figure, axis = plt.subplots(2, 2)

# For Sine Function
axis[0, 0].plot(x, y_bpsk)
#axis[0].ylabel("amplitude")
#axis[0].xlabel("time")
axis[0, 0].set_title("BPSK, input: " + str(bits))

# For Cosine Function
axis[0, 1].plot(x, y_bpsk_with_noise)
axis[0, 1].set_title("BPSK with noise, variance: " + str(variance))

# MOVE X AXIS to CENTER and REMOVE Y to NONE
# Move left y-axis and bottim x-axis to centre, passing through (0,0)
# axis[1, 0].spines['left'].set_position('center')
axis[1, 0].spines['bottom'].set_position('center')

# Eliminate upper and right axes
axis[1, 0].spines['left'].set_color('none')
axis[1, 0].spines['right'].set_color('none')
axis[1, 0].spines['top'].set_color('none')

# Show ticks in the left and lower axes only
axis[1, 0].xaxis.set_ticks_position('bottom')
axis[1, 0].yaxis.set_ticks_position('left')

axis[1, 0].plot(x_costellation, y_costellation, color='r', marker = ".", linestyle='None', markersize = 10.0)
axis[1, 0].set_title("BPSK costellation")

axis[1, 1].plot(x_dem, y_dem, color='r', marker = ".", linestyle='None', markersize = 10.0)
axis[1, 1].set_title("BPSK demodutation: " + str(x_dem))

plt.show()
