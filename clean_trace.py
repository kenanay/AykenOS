import sys, re

text = open(sys.argv[1]).read()

# First remove the perf phase lines completely as they might be prefixed by our char
text = re.sub(r'\[\[AYKEN_PERF_PHASE\]\][^\n]*\n', '', text)
text = re.sub(r'\[\[AYKEN_LOW_HALF_KHEAP_RUNTIME\]\][^\n]*\n', '', text)
text = re.sub(r'\[\[AYKEN_[^\]]+\]\]\n?', '', text)
text = re.sub(r'P10_[^\n]*\n', '', text)
text = re.sub(r'RE\[K\][^\n]*\n', '', text)
text = re.sub(r'\[K\]\[[^\n]*\n', '', text)
text = re.sub(r'\[K\][^\n]*\n', '', text)
text = re.sub(r'\[B\]\[[^\n]*\n', '', text)
text = re.sub(r'\[B\][^\n]*\n', '', text)
text = re.sub(r'\[MARKER\]\s*', '', text)
text = re.sub(r'\[TMR\][^\n]*\n', '', text)
text = re.sub(r'LTR=[0-9A-F]+\n', '', text)
text = re.sub(r'ApqiggggIBK', '', text) # some boot sequence 
text = re.sub(r'000000000[0-9A-F]+\n', '', text)
text = re.sub(r'MaAbBH', '', text)
text = re.sub(r'ELF\n', '', text)
text = re.sub(r'ELFE\n', '', text)

print(text)
