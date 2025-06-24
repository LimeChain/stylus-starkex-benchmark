import re

filename = ''

with open(filename, 'r') as f:
    data = f.read()

# Find all polynomial blocks, which start with 'result := add(' and end at the next closing parenthesis at the right nesting level
# We'll use a regex to match: result := add( ... )
blocks = re.findall(r'result := add\(([\s\S]*?)\n\s*\)', data)
all_coeffs = []
for block in blocks:
    # Extract all 0x... hex numbers from the block
    coeffs = re.findall(r'0x[0-9a-fA-F]+', block)
    # Reverse to Horner's method order
    coeffs = list(reversed(coeffs))
    all_coeffs.extend(coeffs)

print(f"const COEFFS: [U256; {len(all_coeffs)}] = [")
for c in all_coeffs:
    print(f"    uint!({c}_U256),")
print("];")
print(f"// COEFFS LEN: {len(all_coeffs)};")
