# Penumbra Cryptography Learning Guide

Welcome! This directory contains educational documentation about Penumbra's cryptographic architecture, written for developers new to zero-knowledge proofs and privacy-preserving blockchain technology.

## 📚 Reading Order

If you're new to this material, read the documents in this order:

### Fundamentals (Start Here!)
1. **[01-what-is-zero-knowledge.md](./01-what-is-zero-knowledge.md)**
   - Core concept: Prove something without revealing it
   - Real-world analogies
   - Example: Buying coffee with ZK proofs
   - ~5 min read

2. **[09-commitments-explained.md](./09-commitments-explained.md)** ⭐ NEW!
   - What commitments are (the locked box analogy)
   - Pedersen commitments with concrete math examples
   - Why you can't reverse or fake them
   - How transactions use commitments to hide amounts
   - ~12 min read

3. **[02-what-is-a-snark.md](./02-what-is-a-snark.md)**
   - What makes proofs "succinct" and "non-interactive"
   - How SNARKs compress complex proofs into tiny sizes
   - Penumbra's 7 proof types and their performance
   - ~7 min read

4. **[03-what-is-groth16.md](./03-what-is-groth16.md)**
   - Specific SNARK system used by Penumbra
   - Trusted setup ceremonies
   - Why BLS12-377 curve was chosen
   - Trade-offs vs other proof systems
   - ~8 min read

5. **[04-what-is-a-circuit.md](./04-what-is-a-circuit.md)**
   - How to write zero-knowledge programs
   - Mock examples you can understand
   - Real Penumbra circuits explained
   - How R1CS constraints work
   - ~10 min read

### Penumbra-Specific
6. **[05-viewing-keys-explained.md](./05-viewing-keys-explained.md)**
   - Penumbra's key hierarchy
   - Spend Key → Full Viewing Key → IVK/OVK
   - How each key type works in the codebase
   - Selective disclosure capabilities
   - ~12 min read

7. **[08-how-penumbra-notes-work.md](./08-how-penumbra-notes-work.md)**
   - Notes as the fundamental unit of value
   - Complete lifecycle: creation → encryption → scanning → spending
   - How Diffie-Hellman key agreement enables decryption
   - Note commitments and the Merkle tree
   - ~15 min read

8. **[06-penumbra-crypto-architecture.md](./06-penumbra-crypto-architecture.md)**
   - Complete system overview
   - How all the pieces fit together
   - The 7-layer crypto stack
   - Example transaction traced through the system
   - ~20 min read

### Advanced Topics
9. **[07-roadmap-for-extending-viewing-keys.md](./07-roadmap-for-extending-viewing-keys.md)**
   - Ideas for extending viewing key functionality
   - Implementation roadmap and priorities
   - Technical prerequisites
   - First project suggestions
   - ~15 min read

## 🎯 Quick Reference by Goal

**Want to understand the basics?**
→ Read 01, 09, 02, then 03

**Want to understand how privacy works?**
→ Read 01, 05, then 08

**Want to understand the proofs?**
→ Read 02, 03, then 04

**Want to modify/extend viewing keys?**
→ Read all of them, focusing on 05 and 07

**Want the complete picture?**
→ Read them all in order!

## 🔑 Key Concepts Covered

### Cryptographic Primitives
- Zero-knowledge proofs (ZKP)
- SNARKs (Succinct Non-interactive Arguments of Knowledge)
- Groth16 proof system
- Elliptic curves (BLS12-377, Decaf377)
- Hash functions (Poseidon377)
- Commitment schemes (Pedersen)
- Key agreement (Diffie-Hellman)

### Penumbra-Specific
- Viewing key hierarchy
- Note structure and lifecycle
- Tiered Commitment Tree (TCT)
- Fuzzy Message Detection (FMD)
- Nullifiers (prevent double-spending)
- Circuits (Spend, Output, Swap, etc.)

### Privacy Techniques
- Encrypted notes
- Hidden balances (commitments)
- Unlinkable addresses
- Selective disclosure
- Private transactions

## 📂 Codebase Reference

Key files referenced in these documents:

### Keys & Addresses
- `crates/core/keys/src/keys/spend.rs` - Spend key
- `crates/core/keys/src/keys/fvk.rs` - Full viewing key
- `crates/core/keys/src/keys/ivk.rs` - Incoming viewing key
- `crates/core/keys/src/keys/ovk.rs` - Outgoing viewing key
- `crates/core/keys/src/address.rs` - Address structure

### Notes & Encryption
- `crates/core/component/shielded-pool/src/note.rs` - Note implementation
- `crates/core/keys/src/symmetric.rs` - Encryption keys

### Proofs & Circuits
- `crates/core/component/shielded-pool/src/spend/proof.rs` - Spend circuit
- `crates/core/component/shielded-pool/src/output/proof.rs` - Output circuit
- `tools/parameter-setup/src/main.rs` - Proof parameter generation

### Cryptographic Primitives
- `crates/crypto/decaf377-*` - Elliptic curve operations
- `crates/crypto/proof-params/` - Groth16 parameters
- `crates/crypto/tct/` - Tiered Commitment Tree

### View Service
- `crates/view/src/service.rs` - Transaction scanning and decryption

## 🛠️ Prerequisites

To understand this material, you should have:

**Required:**
- ✅ Basic Rust knowledge
- ✅ Understanding of public/private key cryptography
- ✅ Willingness to learn!

**Helpful but not required:**
- ⭐ Basic understanding of elliptic curves
- ⭐ Familiarity with hash functions
- ⭐ Knowledge of symmetric encryption

**Not required:**
- ❌ Advanced math (we explain as we go!)
- ❌ Prior ZK experience
- ❌ Deep cryptography background

## 🎓 Learning Tips

1. **Don't rush**: These concepts build on each other. Take your time!

2. **Try the examples**: The documents include mock code examples. Try implementing them!

3. **Read the actual code**: After reading each document, look at the referenced files in the codebase.

4. **Ask questions**: If something is unclear, that's normal! These are complex topics.

5. **Draw diagrams**: Visualizing the flows (keys, notes, proofs) really helps.

## 🌟 What Makes This Different?

These documents were created by:
- ✅ Reading the **actual Penumbra codebase**
- ✅ Analyzing **real implementations**, not just theory
- ✅ Tracing **concrete code paths** through the system
- ✅ Including **real file locations** and **line numbers**
- ✅ Using **ELI5 (Explain Like I'm 5)** style

**Everything here is grounded in the actual code you'll work with!**

## 📖 Additional Resources

### Official Documentation
- [Penumbra Guide](https://guide.penumbra.zone/)
- [Penumbra Protocol Spec](https://protocol.penumbra.zone/)
- [Penumbra GitHub](https://github.com/penumbra-zone/penumbra)

### Background on ZK
- [Zero-Knowledge Proofs: An illustrated primer](https://blog.cryptographyengineering.com/2014/11/27/zero-knowledge-proofs-illustrated-primer/)
- [Zcash Protocol Spec](https://zips.z.cash/protocol/protocol.pdf) - Penumbra builds on similar ideas

### Groth16
- [Groth16 Paper](https://eprint.iacr.org/2016/260.pdf)
- [Why and How zk-SNARK Works](https://arxiv.org/abs/1906.07221)

## 🤝 Contributing

Found an error? Have a suggestion? Want to add more examples?

These documents were created to help **you** learn. If something is unclear or could be better explained, please:
1. Open an issue describing what's confusing
2. Suggest improvements
3. Add your own examples or analogies

## ⚠️ Important Notes

1. **These are learning documents**, not official specifications
2. **Always verify against the actual code** - code is the source of truth
3. **Cryptography is hard** - these simplifications help learning but aren't cryptographically rigorous
4. **Security-critical code** should be reviewed by cryptography experts

## 🚀 Next Steps

After reading these documents, you should:

1. **Clone the Penumbra repository**
   ```bash
   git clone https://github.com/penumbra-zone/penumbra
   cd penumbra
   ```

2. **Build and test**
   ```bash
   cargo build
   cargo test -p penumbra-keys
   cargo test -p penumbra-view
   ```

3. **Start with a small project**
   - See document 07 for suggestions
   - Try implementing a filtered viewing key
   - Contribute to the codebase!

4. **Join the community**
   - Penumbra Discord
   - GitHub discussions
   - Ask questions!

## 📝 Document Status

**Created**: 2025-10-13
**Based on**: Penumbra codebase at commit `9bc30fcd7` (release/2.0.x branch)
**Last Updated**: 2025-10-13

As the codebase evolves, some details may change. Always cross-reference with the current code!

---

Happy learning! 🎉

Remember: **The journey of understanding privacy-preserving cryptography starts with a single zero-knowledge proof!** 🔒✨
