# Message Length

I'm reading Peter D. Grünwald's book on _The Minimum Description Length Principle_, and I'm thinking I should code up an honest-to-God implementation of some of this stuff! That's how you _know_!

§5.2 "Applying Two-Part Code MDL to Markov Chain Hypothesis Selection". I think I'm going to need—

 * implement (some representation) of _k_-th order Markov chain distribution
 * Shannon–Fano coding with respect to (my representation of) a probability distribution
 * the standard code for the integers

Then MDL inference of some bitstream with respect to the _k_-th order Markov chain hypothesis class means minimizing the sum of "bitstream encoded with respect to distribution" + "integer code for Markov-depth parameter" + "integer code for parameter precision" + "the actual parameter value".

... okay, now I'm reading Rissanen 1983, and I don't think I'm reading to start coding (no pun intended) yet?
