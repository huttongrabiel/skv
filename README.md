# SKV

A (VERY) simple networked key-value store.

**This is not by any means a production tool, and I wouldn't use it as such.**

---

# Usage

Clone the repository and cd into the directory.

```
git clone https://github.com/huttongrabiel/skv.git
```

Start the server. It will run on localhost (127.0.0.1) on port 3400.

```
cargo run
```

User can also specify port by doing:

```
cargo run -- -p <port>
```

Starting the server should generate an encryption key for the user which is used
to encrypt/decrypt at rest data. **THE KEY MUST BE SAVED BY THE USER**

---

Supported requests are GET, PUT, and DELETE.

GET Request
```
curl -X GET -H "key: <encryption_key>" localhost:3400/<key>
```

To list all keys in the key-value store use the 'ls' key.
```
curl -X GET -H "key: <encryption_key> localhost:3400/ls
```

---

PUT Request
```
curl -X PUT localhost:3400/<key> --data <value>
```

Users can also store file contents as values
```
curl -X PUT localhost:3400/<key> --data /path/to/file
```

*Put requests do **NOT** require an encryption key. The encryption is handled internally.*

---

DELETE Request (careful with this one ;)...)
```
curl -X DELETE -H "key: <encryption_key> localhost:3400/<key>
```

# TODO
- [X] Data encryption/decryption
    - [X] Basic encryption
    - [X] Write error messages to stream
    - [X] Handle incorrect keys with helpful error messages
- [ ] Encrypt data that is not at rest
    - [ ] TLS
- [ ] CLI
