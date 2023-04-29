# Modulation and Demodulation of arbitrary bytes  

This project started out to be able to create back up on cassette tapes.... obviously cassettes are an incredibly complicated tool and it never really worked...

What you find within this project is an abstraction around the concept of signal, so a mini DSP library that was used to implement BPSK and QAM modulations.

Everything is visible and testable in the `ChannelSimulator` crate that, which as the name implies, is a channel simulator that allows you to test the two aforementioned modulations through various parameters, displaying the modulated, after the AWGN channel and demodulated symbols. 
It also allows you to plot the SNR graph

![alt text](https://github.com/gabriele-0201/cassetti/tree/main/test_and_notes/sreen_channel_simulator.png?raw=true)

