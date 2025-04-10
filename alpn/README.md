# 🚀 Testing TLS ALPN on QUIC Server & Client with Quinn

This guide builds on the basic example by experimenting with different ALPN configurations to explore how TLS handles various protocol negotiation scenarios.
---

## 🔐 Step 1: Generate TLS Certificates

```bash
cargo run --example alpn-genkey
```

## 🖥️ Step 2: Start the Server
```bash
cargo run --example alpn-server
```

## 🧑‍💻 Step 3: Run the Client
### ✅ Connections succeed with supported ALPNs

```bash
cargo run --example alpn-client-h1
```

```bash
cargo run --example alpn-client-h2
```

### 🚫 Connections are rejected when the ALPN is missing or not recognized by the server.
```bash
cargo run --example alpn-client-no-alpn
```

```bash
cargo run --example alpn-client-h3
```
expected output on client
```
Error: failed to connect to server

Caused by:
    aborted by peer: the cryptographic handshake failed: error 120: peer doesn't support any known protocol
```
