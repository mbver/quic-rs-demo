# 🚀 Minimal QUIC Server & Client with Quinn

This guide walks through a bare-bones example of creating a QUIC server and client using the [Quinn](https://github.com/quinn-rs/quinn) library in Rust. It covers certificate generation, starting a server, and connecting with a client.

---

## 🔐 Step 1: Generate TLS Certificates

QUIC requires TLS. Generate a simple, self-signed certificate for `localhost`

```bash
cargo run --example zeroRTT-genkey
```


expected output:
```
✅ Finished generating key!
  📄 Cert: /tmp/quinn_certs/cert.der
  🔑 Key:  /tmp/quinn_certs/key.der
```


## 🖥️ Step 2: Start the Server
```bash
cargo run --example zeroRTT-server
```
expected output
```
🚀 QUIC server listening at: 127.0.0.1:4843
```

## 🧑‍💻 Step 3: Run the Client
```bash
cargo run --example zeroRTT-client
```
expected output on client
```
connected to server 127.0.0.1:4843
response received:
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
```