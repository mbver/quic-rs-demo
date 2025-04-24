# 🚀 Zero-RTT in Quinn

This guide walks through the complete setup of 0-RTT in Quinn, with the server responsible for validating incoming 0-RTT requests before processing them. It also includes a replay attack simulation to highlight the security concerns associated with 0-RTT.

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
posting something in full handshake...
successfully post

resuming connection...
0-RTT connected server 127.0.0.1:4843
{
  "message": "Welcome to Awesome Quinn!",
  "note": "Not to be confused with Queen 👑",
  "disclaimer": "QUIC is quick 🏎️💨",
  "version": "0.1.0",
  "listening_on": "127.0.0.1:4843"
}
resending request...
{
  "message": "Welcome to Awesome Quinn!",
  "note": "Not to be confused with Queen 👑",
  "disclaimer": "QUIC is quick 🏎️💨",
  "version": "0.1.0",
  "listening_on": "127.0.0.1:4843"
}
posting something after 0-rtt...
successfully post

resuming connection again...
0-RTT connected server 127.0.0.1:4843
posting something in 0-rtt...
failed to handle request

resuming connection for replay attack...
0-RTT connected server 127.0.0.1:4843
replay requests in 0-rtt...
{
  "message": "Welcome to Awesome Quinn!",
  "note": "Not to be confused with Queen 👑",
  "disclaimer": "QUIC is quick 🏎️💨",
  "version": "0.1.0",
  "listening_on": "127.0.0.1:4843"
}
{
  "message": "Welcome to Awesome Quinn!",
  "note": "Not to be confused with Queen 👑",
  "disclaimer": "QUIC is quick 🏎️💨",
  "version": "0.1.0",
  "listening_on": "127.0.0.1:4843"
}
{
  "message": "Welcome to Awesome Quinn!",
  "note": "Not to be confused with Queen 👑",
  "disclaimer": "QUIC is quick 🏎️💨",
  "version": "0.1.0",
  "listening_on": "127.0.0.1:4843"
}
{
  "message": "Welcome to Awesome Quinn!",
  "note": "Not to be confused with Queen 👑",
  "disclaimer": "QUIC is quick 🏎️💨",
  "version": "0.1.0",
  "listening_on": "127.0.0.1:4843"
}
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
accepting incomming connection from 127.0.0.1:4385
established connection from 127.0.0.1:4385
req GET sample.json\r\n
req is_0rtt false
complete stream handling!
req POST /something some important thing\r\n
req is_0rtt false
client post:  some important thing
complete stream handling!
connection closed

accepting incomming connection from 127.0.0.1:4385
established connection from 127.0.0.1:4385
req GET sample.json\r\n
req is_0rtt true
complete stream handling!
req GET sample.json\r\n
req is_0rtt false
complete stream handling!
req POST /something some important thing\r\n
req is_0rtt false
client post:  some important thing
complete stream handling!
connection closed

accepting incomming connection from 127.0.0.1:4385
established connection from 127.0.0.1:4385
req POST /something some important thing\r\n
req is_0rtt true
handle request failed: 0-RTT is not applied to POST
complete stream handling!
connection closed

accepting incomming connection from 127.0.0.1:4385
established connection from 127.0.0.1:4385
req GET sample.json\r\n
req is_0rtt true
req GET sample.json\r\n
req is_0rtt true
req GET sample.json\r\n
req is_0rtt true
complete stream handling!
complete stream handling!
req GET sample.json\r\n
req is_0rtt true
req GET sample.json\r\n
req is_0rtt true
complete stream handling!
complete stream handling!
complete stream handling!
```