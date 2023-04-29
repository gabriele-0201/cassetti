from scipy import signal
import matplotlib.pyplot as plt
import numpy as np
from scipy.fftpack import fft, fftshift

#window = signal.cosine(100000)
#plt.plot(window)
#plt.title("Cosine window")
#plt.ylabel("Amplitude")
#plt.xlabel("Sample")
#
#plt.figure()

data = np.linspace(0, 10)
cos = np.cos(data)
plt.plot(data, cos)
plt.title("Cosine window")
plt.ylabel("Amplitude")
plt.xlabel("Sample")

#plt.figure()
#A = fft(window, 2048) / (len(window)/2.0)
#freq = np.linspace(-0.5, 0.5, len(A))
#response = 20 * np.log10(np.abs(fftshift(A / abs(A).max())))
#plt.plot(freq, response)
#plt.axis([-0.5, 0.5, -120, 0])
#plt.title("Frequency response of the cosine window")
#plt.ylabel("Normalized magnitude [dB]")
#plt.xlabel("Normalized frequency [cycles per sample]")
plt.show()
