<p align="center">
  <a href="https://github.com/if-nil/reapi" target="_blank" rel="noopener noreferrer">
    <img width="180" src="https://raw.githubusercontent.com/if-nil/reapi/refs/heads/master/docs/image/logo.svg" alt="ReAPI logo">
  </a>
</p>

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

## Configuration Parameters

When loading the module, you can specify the following parameters:

```shell
redis-server --loadmodule /path/to/libreapi.so reapi_host 127.0.0.1 reapi_port 9098
```

| Parameter | Description | Default Value |
|-----------|-------------|---------------|
| reapi_host | The host address that ReAPI server will bind to | 127.0.0.1 |
| reapi_port | The port number that ReAPI server will listen on | 9098 |


## Usage Examples

### HTTP API

```shell
# Set key-value in default database (DB 0)
curl http://127.0.0.1:9098/set/mykey/hello
{"result":"OK"}

# Set key-value in database 1
curl http://127.0.0.1:9098/1/set/mykey/hello
{"result":"OK"}
```

```shell
# Get value from default database (DB 0)
curl http://127.0.0.1:9098/get/mykey
{"result":"hello"}

# Get value from database 1
curl http://127.0.0.1:9098/1/get/mykey
{"result":"hello"}
```

```shell
# Execute other Redis commands
curl http://127.0.0.1:9098/hset/myhash/field1/value1
```

### Web Interface

Visit `http://127.0.0.1:9098` to use the built-in web console. You can:
- Input Redis commands in the command line
- Select database (DB 0-15) from the dropdown menu
- Execute commands and see results in real-time

[![](docs/image/web.png)](docs/image/web.png)
