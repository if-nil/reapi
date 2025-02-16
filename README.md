
![ReAPI](docs/image/logo.svg)

English | [中文](README_ZH.md)

# Redis HTTP API Module (ReAPI)

A Redis module that provides HTTP interfaces to access Redis commands. It allows interaction with Redis through RESTful APIs, making it convenient for non-Redis clients to communicate with Redis.

## Features

- HTTP interface for Redis commands
- JSON formatted responses
- Support for all Redis commands
- Built-in Web interface
- Lightweight and easy to deploy

## Installation

1. Make sure your Redis supports modules
2. Build the module:
```shell
cargo build --release
```
3. Load the module into Redis:
```shell
redis-server --loadmodule /path/to/libreapi.so
```

## Usage Examples

### HTTP API

```shell
# Set key-value
curl http://127.0.0.1:9098/set/mykey/hello
{"result":"OK"}
```

```shell
# Get value
curl http://127.0.0.1:9098/get/mykey
{"result":"hello"}
```

```shell
# Execute other Redis commands
curl http://127.0.0.1:9098/hset/myhash/field1/value1
```

### Web Interface

Visit `http://127.0.0.1:9098` to use the built-in web console.