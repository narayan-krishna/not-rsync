# Rsync-rs

Rsync-rs is a WIP file synchronization tool based on Rsync and the Rsync algorithm, written in Rust. 

While functional, Rsync-rs needs work on the following:

- [ ] more robust integration and unit testing
- [ ] performance measurement (criterion)
- [ ] support for (recursive) directory synchronization
- [ ] batch processing for large groups of files
- [ ] a more strict send/receive protocol using serde serialize/deserialize
- [ ] better comments

and more.

Rsync-rs works locally and remotly, and operates differently depending on the nature of the connection. To synchronize files on the same machine, the client spawns a child thread server and communicates over shared-memory. To sync with a remote machine, the client creates an SSH tunnel and two respective channels: a session channel for launching the server, and a direct-tcpip channel for bidirectional communication between the client and the newly-launched server.

rsync-rs aims to use a similar CLI to the original rsync, currently with more limited options

```bash
rsync ... SRC ... [USER@]HOST:DEST # synchronize a remote file with local
rsync ... [USER@]HOST:SRC ... DEST # synchronize a local file with remote
```

### Rsync algorithm

Given two computers _a_ and _b_ with access to files _A_ and _B_, where the goal is to synchronize _B_ from _A_:
  1. split file _B_ into series of non-overlapping fixed-sized blocks of size _S_ bytes (last block can be shorter than _S_ bytes)
  2. For each block, _b_ calculates two checksums:
      - a weak "rolling 32-but checksum"
      - a strong 128-bit MD4 checksum
  3. _b_ sends _checksums_ to _a_
  4. _a_ searches through _A_ to find blocks on length S bytes that have the same weak and strong checksum as any of the blocks of _B_ (using rolling checksum)
  5. _a_ sends instructions to _b_ for copying _A_, where each instruction is either a reference to a block of B, or literal data (data where A did not match blocks of B)

Rsync-rs uses Dropbox's `fast_rsync` algorithm implementation, which attempts to speed up delta generation by leveraging SIMD instructions where possible. Rsync-rs implements its own wire protocol to allow client and server to remotely communicate at each step of the algorithm.

#### Links
- [the rsync algorithm](https://www.andrew.cmu.edu/course/15-749/READINGS/required/cas/tridgell96.pdf)
- [rsync thesis](https://www.samba.org/~tridge/phd_thesis.pdf)

