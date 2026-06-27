# Rust HTTP Server

A simple HTTP server built from scratch in Rust, created as part of the [CodeCrafters HTTP Server challenge](https://codecrafters.io/challenges/http-server).

## What it does

This server listens on `127.0.0.1:4221` and handles HTTP/1.1 requests. It supports multiple concurrent connections using threads, and responds to the following endpoints:

### Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/` | Returns `200 OK` |
| GET | `/echo/{str}` | Returns the `{str}` back in the response body |
| GET | `/user-agent` | Returns the client's `User-Agent` header value |
| GET | `/files/{filename}` | Returns the contents of the file from the configured directory |
| POST | `/files/{filename}` | Creates a new file in the configured directory with the request body |
| ANY | anything else | Returns `404 Not Found` |

---

## How to run

```bash
./your_program.sh --directory /tmp/
```

The `--directory` flag tells the server where to read and write files.

---

## How to test each endpoint

### 1. Root endpoint
```bash
curl -v http://localhost:4221/
```
Expected response:
```
HTTP/1.1 200 OK
```

---

### 2. Echo endpoint
```bash
curl -v http://localhost:4221/echo/hello
```
Expected response:
```
HTTP/1.1 200 OK
Content-Type: text/plain
Content-Length: 5

hello
```

---

### 3. User-Agent endpoint
```bash
curl -v http://localhost:4221/user-agent
```
Expected response:
```
HTTP/1.1 200 OK
Content-Type: text/plain
Content-Length: 11

curl/8.5.0
```

---

### 4. GET a file
First create a file in your directory:
```bash
echo "hello world" > /tmp/test.txt
```

Then request it:
```bash
curl -v http://localhost:4221/files/test.txt
```
Expected response:
```
HTTP/1.1 200 OK
Content-Type: application/octet-stream
Content-Length: 12

hello world
```

If the file does not exist:
```
HTTP/1.1 404 Not Found
```

---

### 5. POST a file
```bash
curl -v --data "12345" -H "Content-Type: application/octet-stream" http://localhost:4221/files/file_123
```
Expected response:
```
HTTP/1.1 201 Created
```

This creates a file at `/tmp/file_123` containing `12345`. Verify it:
```bash
cat /tmp/file_123
# 12345
```

---

### 6. Concurrent connections
```bash
(sleep 1 && printf "GET / HTTP/1.1\r\n\r\n") | nc localhost 4221 &
(sleep 1 && printf "GET / HTTP/1.1\r\n\r\n") | nc localhost 4221 &
(sleep 1 && printf "GET / HTTP/1.1\r\n\r\n") | nc localhost 4221 &
```

All three should receive `200 OK` responses simultaneously.

---

## How it works internally

### Request parsing
Every HTTP request looks like this:
```
GET /echo/abc HTTP/1.1\r\n
Host: localhost:4221\r\n
User-Agent: curl/8.5.0\r\n
\r\n
```

The server reads it in three steps:
1. **Request line** — first line, gives method and path
2. **Headers** — subsequent lines until an empty line
3. **Body** — remaining bytes (only for POST, read using `Content-Length`)

### Concurrency
Each incoming connection is handled in its own thread using `std::thread::spawn`, so multiple clients can connect at the same time without blocking each other.

### File serving
The `--directory` flag sets the base directory at startup. All file reads and writes are relative to this directory.
