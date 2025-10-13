#!/usr/bin/env python3
"""
Simple Commitment Scheme Demo
==============================

This is a SIMPLIFIED educational implementation to understand commitments.
NOT cryptographically secure! Just for learning.

In real systems like Penumbra, this uses elliptic curves (much more secure).
"""

import random
import hashlib

# ============================================================================
# PART 1: SUPER SIMPLE VERSION (Easy to Understand)
# ============================================================================

print("=" * 70)
print("PART 1: SUPER SIMPLE COMMITMENT SCHEME")
print("=" * 70)
print()

# Public parameters (everyone knows these)
G = 5  # Generator 1
H = 7  # Generator 2

print(f"Public parameters (everyone knows):")
print(f"  G = {G}")
print(f"  H = {H}")
print()

def simple_commit(value, blinding):
    """
    Create a commitment: value * G + blinding * H

    Args:
        value: The secret value you want to commit to
        blinding: A random secret number to hide the value

    Returns:
        The commitment (a single number)
    """
    return value * G + blinding * H

def simple_verify(commitment, claimed_value, claimed_blinding):
    """
    Verify that a commitment matches the claimed values

    Returns:
        True if valid, False otherwise
    """
    recomputed = simple_commit(claimed_value, claimed_blinding)
    return recomputed == commitment

# ============================================================================
# Example 1: Creating and verifying a commitment
# ============================================================================

print("Example 1: Alice commits to 100 tokens")
print("-" * 70)

alice_value = 100
alice_blinding = 42  # Random secret

alice_commitment = simple_commit(alice_value, alice_blinding)

print(f"Alice's secret value: {alice_value} tokens")
print(f"Alice's secret blinding: {alice_blinding}")
print(f"Alice's commitment (public): {alice_commitment}")
print()

# Later, Alice reveals the value and blinding
print("Alice reveals her secrets...")
is_valid = simple_verify(alice_commitment, alice_value, alice_blinding)
print(f"✓ Commitment is valid: {is_valid}")
print()

# ============================================================================
# Example 2: Can Alice cheat and claim a different value?
# ============================================================================

print("Example 2: Can Alice cheat?")
print("-" * 70)

print(f"Alice's commitment was: {alice_commitment}")
print()

# Alice tries to lie and say she committed to 200 instead of 100
fake_value = 200
print(f"Alice tries to claim she committed to {fake_value} tokens...")
is_valid_fake = simple_verify(alice_commitment, fake_value, alice_blinding)
print(f"✗ Commitment is valid: {is_valid_fake}")
print("She can't cheat!")
print()

# ============================================================================
# Example 3: Can Alice swap value and blinding?
# ============================================================================

print("Example 3: Can Alice swap value and blinding?")
print("-" * 70)

print(f"Original: value={alice_value}, blinding={alice_blinding}")
print(f"Original commitment: {alice_commitment}")
print()

# Try swapping
swapped_value = alice_blinding  # Use blinding as value
swapped_blinding = alice_value  # Use value as blinding

swapped_commitment = simple_commit(swapped_value, swapped_blinding)
print(f"If swapped: value={swapped_value}, blinding={swapped_blinding}")
print(f"Swapped commitment: {swapped_commitment}")
print()

print(f"Original commitment: {alice_commitment}")
print(f"Swapped commitment:  {swapped_commitment}")
print(f"Are they equal? {alice_commitment == swapped_commitment}")
print()

if alice_commitment != swapped_commitment:
    print("✗ NOPE! You CANNOT swap value and blinding!")
    print(f"  Original: {alice_value}*{G} + {alice_blinding}*{H} = {alice_commitment}")
    print(f"  Swapped:  {swapped_value}*{G} + {swapped_blinding}*{H} = {swapped_commitment}")
    print(f"  These produce DIFFERENT commitments!")
print()

# ============================================================================
# Example 4: Homomorphic property (adding commitments)
# ============================================================================

print("Example 4: Adding commitments (Homomorphic property)")
print("-" * 70)

alice_value = 50
alice_blinding = 10
alice_commit = simple_commit(alice_value, alice_blinding)

bob_value = 30
bob_blinding = 20
bob_commit = simple_commit(bob_value, bob_blinding)

print(f"Alice commits to {alice_value} tokens: {alice_commit}")
print(f"Bob commits to {bob_value} tokens: {bob_commit}")
print()

# Add the commitments
total_commit = alice_commit + bob_commit
print(f"Sum of commitments: {total_commit}")
print()

# This should equal a commitment to the sum!
expected_value = alice_value + bob_value  # 50 + 30 = 80
expected_blinding = alice_blinding + bob_blinding  # 10 + 20 = 30
expected_commit = simple_commit(expected_value, expected_blinding)

print(f"Expected (commit to {expected_value}): {expected_commit}")
print(f"Actual sum: {total_commit}")
print(f"Match? {total_commit == expected_commit}")
print()

if total_commit == expected_commit:
    print("✓ YES! Adding commitments = commitment to sum!")
    print("  This is the MAGIC that makes private transactions work!")
print()

# ============================================================================
# Example 5: Transaction balancing
# ============================================================================

print("Example 5: Transaction (inputs = outputs)")
print("-" * 70)

# Alice has 100 tokens (input)
input_value = 100
input_blinding = random.randint(1000, 9999)
input_commit = simple_commit(input_value, input_blinding)

# Alice sends 60 to Bob (output 1)
output1_value = 60
output1_blinding = random.randint(1000, 9999)
output1_commit = simple_commit(output1_value, output1_blinding)

# Alice gets 40 change (output 2)
output2_value = 40
output2_blinding = random.randint(1000, 9999)
output2_commit = simple_commit(output2_value, output2_blinding)

print(f"Input:  {input_value} tokens → commitment: {input_commit}")
print(f"Output 1: {output1_value} tokens (to Bob) → commitment: {output1_commit}")
print(f"Output 2: {output2_value} tokens (change) → commitment: {output2_commit}")
print()

# Check if transaction balances
balance_check = input_commit - output1_commit - output2_commit

print(f"Balance check: {input_commit} - {output1_commit} - {output2_commit} = {balance_check}")
print()

# For it to balance perfectly, we need the values to sum to zero
# But the blindings won't sum to zero (they're random)
# In real systems, you'd prove in ZK that the values balance

# Let's check if the VALUES balance (ignoring blindings)
value_balance = input_value - output1_value - output2_value
print(f"Value balance: {input_value} - {output1_value} - {output2_value} = {value_balance}")

if value_balance == 0:
    print("✓ Transaction balances! (values sum to zero)")
    print("  In Penumbra, a ZK proof ensures this WITHOUT revealing amounts!")
print()

# ============================================================================
# PART 2: MORE REALISTIC VERSION (Using Hash Functions)
# ============================================================================

print("=" * 70)
print("PART 2: MORE REALISTIC COMMITMENT SCHEME (Hash-based)")
print("=" * 70)
print()

def hash_commit(value, blinding):
    """
    More realistic commitment using hash function
    Commitment = Hash(value || blinding)

    This is closer to how real systems work (though Penumbra uses elliptic curves)
    """
    # Combine value and blinding into a string
    data = f"{value}||{blinding}".encode()

    # Hash it
    digest = hashlib.sha256(data).hexdigest()

    return digest

def hash_verify(commitment, claimed_value, claimed_blinding):
    """Verify hash-based commitment"""
    recomputed = hash_commit(claimed_value, claimed_blinding)
    return recomputed == commitment

print("Example 6: Hash-based commitment")
print("-" * 70)

secret_value = 100
secret_blinding = random.randint(10**20, 10**30)

commitment = hash_commit(secret_value, secret_blinding)

print(f"Secret value: {secret_value}")
print(f"Secret blinding: {secret_blinding}")
print(f"Commitment: {commitment}")
print()

# Properties:
print("Properties:")
print(f"✓ HIDING: You can't tell the value from: {commitment}")
print(f"✓ BINDING: Can't change value after committing")
print()

# Verify
is_valid = hash_verify(commitment, secret_value, secret_blinding)
print(f"Verification: {is_valid}")
print()

# Try to cheat
fake_value = 200
is_fake_valid = hash_verify(commitment, fake_value, secret_blinding)
print(f"Can we claim it was {fake_value}? {is_fake_valid}")
print()

# ============================================================================
# INTERACTIVE PLAYGROUND
# ============================================================================

print("=" * 70)
print("INTERACTIVE PLAYGROUND")
print("=" * 70)
print()

def playground():
    """Interactive mode to play with commitments"""
    print("Try creating your own commitments!")
    print()

    while True:
        try:
            value = int(input("Enter a value to commit to (or 'q' to quit): "))
            blinding = random.randint(1000, 999999)

            commitment = simple_commit(value, blinding)

            print(f"\n✓ Created commitment:")
            print(f"  Value: {value} (secret)")
            print(f"  Blinding: {blinding} (secret)")
            print(f"  Commitment: {commitment} (public)")
            print()

            # Try to guess
            guess = input("Can you guess the value from the commitment? ")
            if guess.isdigit() and int(guess) == value:
                print("You got it! (But only because I told you the value)")
            else:
                print(f"Nope! The actual value was {value}")
                print("This shows commitments are HIDING - you can't guess the value!")

            print()

            # Ask to continue
            cont = input("Try another? (y/n): ")
            if cont.lower() != 'y':
                break

        except ValueError:
            break

    print("\nThanks for playing!")

# Uncomment to run interactive mode:
# playground()

print()
print("=" * 70)
print("KEY TAKEAWAYS")
print("=" * 70)
print("""
1. Commitments HIDE the value
   - Commitment looks random, reveals nothing about value

2. Commitments are BINDING
   - Can't change value after creating commitment
   - Can't swap value and blinding (produces different commitment!)

3. Commitments are HOMOMORPHIC
   - Can add commitments → commitment to sum
   - This makes private transactions possible!

4. In Penumbra:
   - Uses elliptic curves (much more secure than simple math)
   - Commitments are curve points, not simple numbers
   - But the IDEA is the same!

5. Zero-Knowledge Proofs prove properties about commitments
   - Like "these commitments sum to zero"
   - WITHOUT revealing the actual values!
""")

print()
print("Try modifying this script to experiment!")
print("  - Change the values and see what happens")
print("  - Try to break the commitment scheme")
print("  - Uncomment playground() to play interactively")
