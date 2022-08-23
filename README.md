# SKV

A (VERY) simple networked key-value store.

**This is not by any means a production tool, and I wouldn't use it as such.**

*Also note that this key-value store is NOT persistent, if the server goes down,
so does the data*

---

# Installation

Clone the repository, cd into the directory, and build the project.

```
git clone https://github.com/huttongrabiel/skv.git

cd path/to/skvrepo

cargo build --release
```

# Usage

*For user friendly usage see [skv_talk](https://github.com/huttongrabiel/skv_talk).*

```bash

# Start the server. It will run on localhost (127.0.0.1) on port 3400.
./target/release/skv -p <port> # [default 3400] -p is optional

# SAVE THE ENCRYPTION KEY GENERATED AT SERVER START. Message looks as such:
"Save this key and keep it secret! It cannot and will not be regenerated.
9be2b1462a8364d51b1dfb66c7a729101bf3e7ac196c68f22c8af83918f605ab"

# GET Request
curl -X GET -H "key: <encryption_key>" localhost:3400/<key>

# PUT Request. No encryption key required.
curl -X PUT localhost:3400/<key> --data <value>

# DELETE Request (careful with this one ;)...)
curl -X DELETE -H "key: <encryption_key>" localhost:3400/<key>

# To list all keys in the key-value store use the 'ls' key.
curl -X GET -H "key: <encryption_key>" localhost:3400/ls

# Users can also store file contents as values
curl -X PUT localhost:3400/<key> --data /path/to/file

```

# TODO
- [X] Data encryption/decryption
    - [X] Basic encryption
    - [X] Write error messages to stream
    - [X] Handle incorrect keys with helpful error messages
- [ ] Encrypt data that is not at rest
    - [ ] TLS
