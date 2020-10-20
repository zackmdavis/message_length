import itertools

for n in range(1, 4):
    lines = []
    for bits in itertools.product(['0', '1'], repeat=n):
        lines.append("{} -> {}".format(''.join(bits), ''.join(bits[1:] + ('0',))))
        lines.append("{} -> {}".format(''.join(bits), ''.join(bits[1:] + ('1',))))
    print('\n'.join(lines))
    print()
