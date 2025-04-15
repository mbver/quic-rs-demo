# 🚀  QUIC Datagram Extension Demo with Quinn

A minimal example showcasing QUIC datagram support: how to send and receive datagrams using Quinn.
---

## 🔐 Step 1: Generate TLS Certificates

```bash
cargo run --example datagram-genkey
```

## 🖥️ Step 2: Start the Server
```bash
cargo run --example datagram-server
```

## 🧑‍💻 Step 3: Run the Client
```bash
cargo run --example datagram-client
```
expected output on client
```
connected to server 127.0.0.1:4843
sending hello datagram...
received response: 
Hello from server
terminating connection...
Done!   
```

expected output on server
```
accepting connection from 127.0.0.1:4385
established connection from 127.0.0.1:4385
receive msg: 
Hello from client
responding to client 127.0.0.1:4385...
client 127.0.0.1:4385 terminated
Done handle conn from 127.0.0.1:4385
```