# rsync rust project

File synchronization tool that compares the diff between two different device files and attempts to synchronize them. essentially an rsync clone.

- client
- server
    - remote system requires openssh-server

```bash
rsync ... SRC ... [USER@]HOST:DEST # synchronize a remote file with local
rsync ... [USER@]HOST:SRC ... DEST # synchronize a local file with remote
```

## how rsync works

Running `rsync local-file user@remote-host:remote-file` will lead rsync to attempt to synchronize the file on the local machine and the host machine (remote file to match local). Rsync uses ssh to connect as user to the remote host, and the will invoke the remote host's rsync. At this point both programs, will determine the difference between files and determine what parts of the local file need to be transfered to the remote file.

### the rsync algorithm

Given two computers _a_ and _b_ with access to files _A_ and _B_, where the goal is to synchronize _B_ from _A_:
  1. split file _B_ into series of non-overlapping fixed-sized blocks of size _S_ bytes (last block can be shorter than _S_ bytes)
  2. For each block, _b_ calculates two checksums:
      - a weak "rolling 32-but checksum"
      - a strong 128-bit MD4 checksum
  3. _b_ sends _checksums_ to _a_
  4. _a_ searches through _A_ to find blocks on length S bytes that have the same weak and strong checksum as any of the blocks of _B_ (using rolling checksum)
  5. _a_ sends instructions to _b_ for copying _A_, where each instruction is either a reference to a block of B, or literal data (data where A did not match blocks of B)

#### rolling checksum

#### links
- [the rsync algorithm](https://www.andrew.cmu.edu/course/15-749/READINGS/required/cas/tridgell96.pdf)
- [rsync thesis](https://www.samba.org/~tridge/phd_thesis.pdf)

### testing

#### locally

testing doesn't neccesarily require a local machine, can be done by simulating a loopback ssh connection.

1. test ssh connection to localhost (where -i is your identity file, username is your username)
```
ssh -i ~/.ssh/id_rsa username@localhost
```
2. test running commands
```
ssh -i ~/.ssh/id_rsa username@localhost "ls -l"
```
3. test file transfer
```
scp /path/to/localfile username@localhost:/path/on/local/machine
```
4. test port forwarding
```
ssh -L local_port::localhost::remote_port username@localhost
```
5. test program using username@localhost

#### on a remote machine

## crates
- [ssh](https://docs.rs/ssh/latest/ssh/)
- [tokio](https://docs.rs/tokio/latest/tokio/)


## notes
Authentication: The public key is used for authentication purposes. When you want to log in to a remote server using SSH key authentication, you provide your public key to the server. The server checks whether your public key is listed in an authorized keys file (usually ~/.ssh/authorized_keys on the server). If your public key is found and matches the private key you possess, you are granted access without needing to enter a password.

This authentication method is more secure than password-based authentication because it relies on something you have (the private key) rather than something you know (a password).

Encryption: The public and private keys are used for encryption and decryption during the SSH session. After the initial authentication, a symmetric encryption key is generated for the duration of the session. This symmetric key is used to encrypt the data transferred between your local machine and the remote server. However, the initial key exchange and session negotiation are encrypted using your public and private keys.

This encryption ensures that even if someone intercepts the data being transmitted, they cannot decipher it without access to your private key.

