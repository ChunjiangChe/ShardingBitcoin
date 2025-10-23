# Input hex string (with leading zeros preserved)
prop_size = 5
avai_size = prop_size
shard_num = 4
tx_hex = "00000c7fffffffffffffffffffffffffffffffffffffffffffffffffffffffff"

# Convert to integer
tx_num = int(tx_hex, 16)

prop_num = tx_num // prop_size
in_avai_num = (prop_num // shard_num) // 5
avai_num = in_avai_num * 4
# quotient = num // 5
manifoldchain_in_num = in_avai_num * avai_size
manifoldchain_ex_num = avai_num * avai_size

# Format back to hex, padded to the same length
prop_hex = f"{prop_num:0{len(tx_hex)}x}"
avai_hex = f"{avai_num:0{len(tx_hex)}x}"
in_avai_hex = f"{in_avai_num:0{len(tx_hex)}x}"
manifoldchain_in_hex = f"{manifoldchain_in_num:0{len(tx_hex)}x}"
manifoldchain_ex_hex = f"{manifoldchain_ex_num:0{len(tx_hex)}x}"

print("Tx diff :", tx_hex)
print("Proposer diff :", prop_hex)
print("Availability diff :", avai_hex)
print("Inclusive Availability diff :", in_avai_hex)
print("Manifoldchain Inclusive diff :", manifoldchain_in_hex)
print("Manifoldchain Exclusive diff :", manifoldchain_ex_hex)
