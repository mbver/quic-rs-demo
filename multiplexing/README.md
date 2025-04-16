# 🚀 QUIC multiplexing demo in Quinn

This guide showcases how QUIC, through Quinn, supports multiplexing — allowing a single connection to handle multiple bidirectional and unidirectional streams, along with datagrams, all at the same time.

---

## 🔐 Step 1: Generate TLS Certificates

```bash
cargo run --example basic-genkey
```

## 🖥️ Step 2: Start the Server
```bash
cargo run --example basic-server
```

## 🧑‍💻 Step 3: Run the Client
```bash
cargo run --example basic-client
```
expected output on client
```
connected to server 127.0.0.1:4843

open bidirectional stream number 0
stream number 0 is sending request ...
stream number 0 is receiving response ...

open bidirectional stream number 1
stream number 1 is sending request ...
stream number 1 is receiving response ...

open unidirectional stream
uni_stream uploading data...
Done uploading data with uni_stream!

Start sending/receiving datagram...
recevied datagram response: 
Hello from server
Done sending/receiving datagram!

response received on stream number 0:
{
  "message": "Welcome to Awesome Quinn!",
  "note": "Not to be confused with Queen 👑",
  "disclaimer": "QUIC is quick 🏎️💨",
  "version": "0.1.0",
  "listening_on": "127.0.0.1:4843"
}

response received on stream number 1:
{
  "message": "Welcome to Awesome Quinn!",
  "note": "Not to be confused with Queen 👑",
  "disclaimer": "QUIC is quick 🏎️💨",
  "version": "0.1.0",
  "listening_on": "127.0.0.1:4843"
}

closing connection...
done
```

expected output on server
```
accepting connection from 127.0.0.1:4385
established connection from 127.0.0.1:4385
accepting bidirectional stream...
accepting unidirectional stream...
accepting datagram from client...
received datagram: 
Hello from client
sending datagram to client 127.0.0.1:4385...
Done respond to datagram!
req GET sample.json\r\n
accepting bidirectional stream...
req GET sample.json\r\n
complete bidirectional stream handling!
complete bidirectional stream handling!

data received from client:
{
  "message": "Welcome to Awesome Quinn!",
  "note": "Not to be confused with Queen 👑",
  "disclaimer": "QUIC is quick 🏎️💨",
  "version": "0.1.0",
  "listening_on": "127.0.0.1:4843"
}
Done handle uni_stream!
connection closed
```