# bmb-httpd

An HTTP request processor demonstrating HTTP protocol handling in BMB.

## Usage

```bash
bmb run main.bmb -- <method> <path> [body]
```

## API Routes

| Method | Path | Description | Body |
|--------|------|-------------|------|
| `GET` | `/` | Hello endpoint | - |
| `GET` | `/api/hello` | Returns greeting JSON | - |
| `GET` | `/api/time` | Returns timestamp | - |
| `GET` | `/api/status` | Server status | - |
| `POST` | `/api/echo` | Echoes request body | text |
| `POST` | `/api/add` | Adds two numbers | `a,b` |

## Examples

```bash
# Hello endpoint
bmb run main.bmb -- GET /api/hello
# Output:
# [INFO] GET /api/hello
# HTTP/1.1 200 OK
# Content-Type: application/json
# Content-Length: 27
#
# {"message":"Hello, World!"}

# Get server status
bmb run main.bmb -- GET /api/status
# Output:
# [INFO] GET /api/status
# HTTP/1.1 200 OK
# Content-Type: application/json
# Content-Length: 36
#
# {"status":"running","version":"0.1.0"}

# Echo endpoint
bmb run main.bmb -- POST /api/echo "hello world"
# Output:
# [INFO] POST /api/echo
# HTTP/1.1 200 OK
# Content-Type: application/json
# Content-Length: 22
#
# {"echo":"hello world"}

# Add numbers
bmb run main.bmb -- POST /api/add "10,25"
# Output:
# [INFO] POST /api/add
# HTTP/1.1 200 OK
# Content-Type: application/json
# Content-Length: 26
#
# {"a":10,"b":25,"sum":35}

# Invalid route
bmb run main.bmb -- GET /unknown
# Output:
# [INFO] GET /unknown
# [WARN] Route not found: /unknown
# HTTP/1.1 404 Not Found
# Content-Type: application/json
# Content-Length: 20
#
# {"error":"Not Found"}
```

## Architecture

```
Request (method, path, body)
          │
          ▼
    ┌───────────┐
    │  Parser   │  Parse method, validate path
    └─────┬─────┘
          │
          ▼
    ┌───────────┐
    │  Router   │  Match path to handler
    └─────┬─────┘
          │
          ▼
    ┌───────────┐
    │  Handler  │  Process request, generate response
    └─────┬─────┘
          │
          ▼
    ┌───────────┐
    │ Response  │  Build HTTP response
    └───────────┘
          │
          ▼
     HTTP Output
```

## Response Format

All responses follow standard HTTP/1.1 format:

```
HTTP/1.1 <status-code> <status-text>
Content-Type: <mime-type>
Content-Length: <length>

<body>
```

## HTTP Status Codes

| Code | Meaning | When Used |
|------|---------|-----------|
| 200 | OK | Successful request |
| 201 | Created | Resource created |
| 400 | Bad Request | Invalid input |
| 404 | Not Found | Route not found |
| 405 | Method Not Allowed | Wrong HTTP method |
| 500 | Internal Server Error | Server error |

## Features Demonstrated

1. **HTTP Protocol Handling**
   - Method parsing (GET, POST, PUT, DELETE)
   - Path validation
   - Status codes and headers
   - JSON response building

2. **Routing System**
   - Path matching
   - Method filtering
   - 404 handling

3. **Logging**
   - Log levels (INFO, WARN, ERROR)
   - Request logging

4. **Request Processing**
   - Body parsing
   - Parameter extraction
   - Error handling

5. **BMB Language Features**
   - Contract-based design (`pre`, `post`)
   - Functional composition
   - String manipulation
   - Pattern matching

## Code Structure

| Component | Functions |
|-----------|-----------|
| **Constants** | `method_*()`, `status_*()` |
| **Logging** | `log_info()`, `log_warn()`, `log_error()` |
| **Response** | `send_*_response()`, `send_header()` |
| **Handlers** | `handle_hello()`, `handle_echo()`, etc. |
| **Router** | `route_request()` |
| **CLI** | `main()`, `show_usage()` |

## Running Tests

```bmb
fn main() -> i64 = run_tests();  // Replace main to run tests
```

Expected output: `777` (parsing tests), `888` (routing tests), `999` (utility tests)

## Extension Ideas

1. Add query string parsing (`/api/search?q=term`)
2. Implement request body JSON parsing
3. Add middleware support (auth, CORS)
4. Support path parameters (`/api/users/:id`)
5. Add response caching
