# 🚀 Minimal Zero-RTT setup in Quinn

This guide walks through the basic setup to work with 0-RTT in Quinn
---

## 🔐 Step 1: Generate TLS Certificates

```bash
cargo run --example zeroRTT-genkey
```


## 🖥️ Step 2: Start the Server
```bash
cargo run --example zeroRTT-server
```

## 🧑‍💻 Step 3: Run the Client
```bash
cargo run --example zeroRTT-client
```
expected output on client
```
initial connection...
connected to server 127.0.0.1:4843
{
  "message": "Welcome to Awesome Quinn!",
  "note": "Not to be confused with Queen 👑",
  "disclaimer": "QUIC is quick 🏎️💨",
  "version": "0.1.0",
  "listening_on": "127.0.0.1:4843"
}
resuming connection...
0-RTT connected server 127.0.0.1:4843
{
  "message": "Welcome to Awesome Quinn!",
  "note": "Not to be confused with Queen 👑",
  "disclaimer": "QUIC is quick 🏎️💨",
  "version": "0.1.0",
  "listening_on": "127.0.0.1:4843"
}  
```

expected output on server
```
accepting connection from 127.0.0.1:4385
established connection from 127.0.0.1:4385
req GET sample.json\r\n
complete stream handling!
connection closed
accepting connection from 127.0.0.1:4385
established connection from 127.0.0.1:4385
req GET sample.json\r\n
complete stream handling!
```