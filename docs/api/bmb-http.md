# bmb-http API Reference

HTTP client for BMB using curl backend.

## Overview

This package provides HTTP client functionality for BMB. Since BMB doesn't have native socket support, it uses `exec_output` to invoke curl for actual network I/O.

## HTTP Methods

| Function | Description |
|----------|-------------|
| `METHOD_GET() -> String` | Returns "GET" |
| `METHOD_POST() -> String` | Returns "POST" |
| `METHOD_PUT() -> String` | Returns "PUT" |
| `METHOD_DELETE() -> String` | Returns "DELETE" |
| `METHOD_PATCH() -> String` | Returns "PATCH" |
| `METHOD_HEAD() -> String` | Returns "HEAD" |

## HTTP Status Codes

| Function | Value | Description |
|----------|-------|-------------|
| `STATUS_OK()` | 200 | Success |
| `STATUS_CREATED()` | 201 | Resource created |
| `STATUS_NO_CONTENT()` | 204 | No content |
| `STATUS_BAD_REQUEST()` | 400 | Bad request |
| `STATUS_UNAUTHORIZED()` | 401 | Unauthorized |
| `STATUS_FORBIDDEN()` | 403 | Forbidden |
| `STATUS_NOT_FOUND()` | 404 | Not found |
| `STATUS_SERVER_ERROR()` | 500 | Server error |

## Request Functions

### http_get

```bmb
pub fn http_get(url: String) -> String
    pre url.len() > 0
```

Perform a GET request. Returns response body on success.

**Example:**
```bmb
let response = http_get("https://api.example.com/data");
```

### http_get_with_headers

```bmb
pub fn http_get_with_headers(url: String, headers: String) -> String
    pre url.len() > 0
```

Perform a GET request with custom headers. Headers format: `"Header1: Value1\nHeader2: Value2"`.

**Example:**
```bmb
let response = http_get_with_headers(
    "https://api.example.com/data",
    "Authorization: Bearer token123\nAccept: application/json"
);
```

### http_post

```bmb
pub fn http_post(url: String, body: String) -> String
    pre url.len() > 0
```

Perform a POST request with body.

**Example:**
```bmb
let response = http_post("https://api.example.com/users", "name=John&email=john@example.com");
```

### http_post_json

```bmb
pub fn http_post_json(url: String, json_body: String) -> String
    pre url.len() > 0
```

Perform a POST request with JSON body. Automatically sets `Content-Type: application/json`.

**Example:**
```bmb
let response = http_post_json(
    "https://api.example.com/users",
    "{\"name\": \"John\", \"email\": \"john@example.com\"}"
);
```

### http_put

```bmb
pub fn http_put(url: String, body: String) -> String
    pre url.len() > 0
```

Perform a PUT request with body.

### http_delete

```bmb
pub fn http_delete(url: String) -> String
    pre url.len() > 0
```

Perform a DELETE request.

### http_head

```bmb
pub fn http_head(url: String) -> String
    pre url.len() > 0
```

Perform a HEAD request. Returns headers only.

### http_request

```bmb
pub fn http_request(method: String, url: String, body: String) -> String
    pre url.len() > 0
```

Generic HTTP request with method, URL, and optional body. Returns response with status code in `"...\nSTATUS"` format.

## Response Parsing

### parse_status

```bmb
pub fn parse_status(response: String) -> i64
    post ret >= 0
```

Parse HTTP status code from response (assumes `"...\nSTATUS"` format from `http_request`).

**Example:**
```bmb
let response = http_request("GET", "https://api.example.com/data", "");
let status = parse_status(response);
if status == STATUS_OK() { ... }
```

### parse_body

```bmb
pub fn parse_body(response: String) -> String
```

Parse response body (everything except last line with status).

### parse_header

```bmb
pub fn parse_header(headers: String, name: String) -> String
```

Parse a header value from response headers.

**Example:**
```bmb
let headers = http_head("https://api.example.com/data");
let content_type = parse_header(headers, "Content-Type");
```

## URL Encoding

### url_encode

```bmb
pub fn url_encode(s: String) -> String
    post ret.len() >= s.len()
```

Encode a string for use in URLs. Reserved characters are percent-encoded.

**Example:**
```bmb
let encoded = url_encode("hello world");  // "hello%20world"
```

## Query String Building

### build_query

```bmb
pub fn build_query(key1: String, val1: String) -> String
```

Build a query string from a key-value pair.

**Example:**
```bmb
let query = build_query("name", "John Doe");  // "name=John%20Doe"
```

### add_param

```bmb
pub fn add_param(query: String, key: String, val: String) -> String
```

Add a parameter to an existing query string.

**Example:**
```bmb
let query = build_query("name", "John");
let query2 = add_param(query, "age", "30");  // "name=John&age=30"
```

## Status Code Helpers

### is_success

```bmb
pub fn is_success(status: i64) -> bool
    post ret == (status >= 200 and status < 300)
```

Check if status code indicates success (2xx).

### is_client_error

```bmb
pub fn is_client_error(status: i64) -> bool
    post ret == (status >= 400 and status < 500)
```

Check if status code indicates client error (4xx).

### is_server_error

```bmb
pub fn is_server_error(status: i64) -> bool
    post ret == (status >= 500 and status < 600)
```

Check if status code indicates server error (5xx).

## Complete Example

```bmb
use bmb_http::http_request;
use bmb_http::http_post_json;
use bmb_http::parse_status;
use bmb_http::parse_body;
use bmb_http::is_success;
use bmb_http::STATUS_OK;

fn fetch_user(user_id: i64) -> String =
    let url = "https://api.example.com/users/" + int_to_string(user_id);
    let response = http_request("GET", url, "");
    let status = parse_status(response);
    if is_success(status) { parse_body(response) }
    else { "" };

fn create_user(name: String, email: String) -> i64 =
    let json = "{\"name\": \"" + name + "\", \"email\": \"" + email + "\"}";
    let response = http_request("POST", "https://api.example.com/users", json);
    parse_status(response);
```

## Dependencies

- Requires `curl` to be available in PATH
- Uses `exec_output` builtin for process execution
