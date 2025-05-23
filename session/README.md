# 🚀 Minimal QUIC Server and Client with Authentication and Session Management

This guide extends the basic example to build a minimal protocol that supports authentication and session management using the Quinn library in Rust.​

​Session tokens are valid only for the duration of the TLS session between the client and server. Once the connection is closed, these tokens become invalid, preventing replay attacks. This design ensures that intercepted tokens cannot be reused, maintaining the security of the communication.​

---

## 🔐 Step 1: Generate TLS Certificates

```bash
cargo run --example session-genkey
```

## 🖥️ Step 2: Start the Server
```bash
cargo run --example session-server
```

## 🧑‍💻 Step 3: Run the Client
```bash
cargo run --example session-client
```
expected output on client
```
connected to server 127.0.0.1:4843
✅ Login success. Session token received

sending request number 0...
response received:
{
  "message": "Welcome to Awesome Quinn!",
  "note": "Not to be confused with Queen 👑",
  "disclaimer": "QUIC is quick 🏎️💨",
  "version": "0.1.0",
  "listening_on": "127.0.0.1:4843"
}

sending request number 1...
response received:
{
  "message": "Welcome to Awesome Quinn!",
  "note": "Not to be confused with Queen 👑",
  "disclaimer": "QUIC is quick 🏎️💨",
  "version": "0.1.0",
  "listening_on": "127.0.0.1:4843"
}

sending request number 2...
response received:
{
  "message": "Welcome to Awesome Quinn!",
  "note": "Not to be confused with Queen 👑",
  "disclaimer": "QUIC is quick 🏎️💨",
  "version": "0.1.0",
  "listening_on": "127.0.0.1:4843"
}

🔄 starting new connection to reuse session token ...
connected to server 127.0.0.1:4843
response received:
🔒 AUTH ERROR: authentication failed
```

expected output on server
```
accepting connection from 127.0.0.1:4385
established connection from 127.0.0.1:4385
✅ AUTH SUCCESS
🪪 Session established and token sent to client.

✅ session verified
req GET sample.json
done respond to request GET sample.json

✅ session verified
req GET sample.json
done respond to request GET sample.json

✅ session verified
req GET sample.json
done respond to request GET sample.json

connection closed
🛑 client closed, exiting...
complete stream handling!

accepting connection from 127.0.0.1:4385
established connection from 127.0.0.1:4385
🚫 authentication failed: missing POST 
connection closed
```