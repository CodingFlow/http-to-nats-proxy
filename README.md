# Introduction

HTTP to NATS Proxy Service written in Rust. Useful as the entrypoint to
NATS-based microservices to provide REST APIs externally.

# Details

## HTTP To NATS Subject Conversion

The HTTP method and url path are used to create the NATS subject. The first part
is the method, and the rest are each of the path segments.

For example, `GET /animals/dog` becomes `get.animals.dog`.

## NATS Request Body Format

JSON in the following format:

```json
{
    originReplyTo: String,
    headers: Map<String, String>,
    body: Value,
}
```

- `originReplyTo` - NATS inbox reply subject.
- `headers` - HTTP headers sent in request.
- `body` - Request body.

Example:

```json
{
    "originReplyTo": "_INBOX.eA3HitirD384mypcqrLgDS",
    "headers": {
        "Content-Type": "application/json",
        "Host": "example.com"
    },
    "body": {
        "color": "blue"
    }
}
```

## NATS Response Body Format

JSON in the following format:

```json
{
    headers: Map<String, String>,
    body: Value,
    statusCode: integer,
}
```

- `headers` - HTTP response headers.
- `body` - response body.
- `statusCode` - HTTP response status code.

Example:

```json
{
    "headers": {
        "Content-Type": "application/json",
        "Content-Encoding": "br"
    },
    "body": {
        "color": "blue"
    },
    "statusCode": 200
}
```

# Usage

[Install Rust](https://www.rust-lang.org/learn/get-started)

The service reads the folowing environment variables on startup:

- `NATS_SERVICE_HOST` - NATS server url host e.g. 10.0.0.11
- `NATS_SERVICE_PORT` - NATS server url port e.g. 4222

Using the above example values, the service will try to connect to the NATS
server at `nats://10.0.0.11:4222`.

To run ther service:

```bash
cargo run
```
