# 🚀 mTLS setup for QUIC with Quinn

This guide extends the basic example to illustrate how mTLS is setup in Quinn
---

## 🔐 Step 1: Generate TLS Certificates

```bash
cargo run --example mtls-genkey
```
expected output:
```
✅ Finished generating key!
  📄 Server Cert: /tmp/quinn_certs/server_cert.der
  🔑 Server Key:  /tmp/quinn_certs/server_key.der
  📄 Client Cert: /tmp/quinn_certs/client_cert.der
  🔑 Client Key:  /tmp/quinn_certs/client_key.der
```


## 🖥️ Step 2: Start the Server
```bash
cargo run --example mtls-server
```

## 🧑‍💻 Step 3: Run the Client
```bash
cargo run --example mtls-client
```
expected output on client.

Notice how bad client is optimistic until the actual read occurs.
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

anonymous client connecting...
connected to server 127.0.0.1:4843
anonymous client opening stream...
done opening stream!
anonymous client sending request...
done sending request!
anonymous client reading response...
✅ Expected Error: Read(ConnectionLost(ConnectionClosed(ConnectionClose { error_code: Code::crypto(74), frame_type: None, reason: b"peer sent no certificates" })))
```

expected output on server.

Notice how the server rejects bad client early.
```
accepting connection from 127.0.0.1:4385...
req GET sample.json\r\n
complete stream handling!
connection closed
accepting connection from 127.0.0.1:4386...
Error handle incomming connection failed to accept incoming connection

Caused by:
    the cryptographic handshake failed: error 116: peer sent no certificates
```