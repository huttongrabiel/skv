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

Supported requests are GET, PUT, and DELETE.

GET Request
```
curl -X GET localhost:3400/<key>
```

PUT Request
```
curl -X PUT localhost:3400/<key> --data <value>
```

DELETE Request (careful with this one ;)...)
```
curl -X DELETE localhost:3400/<key>
```

# TODO
- [X] Switch to multi-threaded
- [ ] REPL

# Possible TODOS
- [ ] Possibly encrypt values before storing in hashmap?
- [ ] Possibly add frontend GUI for interacting with key-value store (Rust Yew?)
