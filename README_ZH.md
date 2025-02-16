
![ReAPI](docs/image/logo.svg)

[English](README.md) | 中文

# Redis HTTP API Module (ReAPI)

一个Redis模块，提供HTTP接口来访问Redis命令。通过RESTful API方式调用Redis命令，使得非Redis客户端也能方便地与Redis进行交互。

## 功能特点

- 提供HTTP接口访问Redis命令
- JSON格式响应数据
- 支持所有Redis命令
- 内置Web界面
- 轻量级，易于部署

## 安装

1. 确保你的Redis支持模块功能
2. 编译模块:
```shell
cargo build --release
```
3. 将模块加载到Redis:
```shell
redis-server --loadmodule /path/to/libreapi.so
```
## 使用示例

### HTTP API

```shell
# 设置键值
curl http://127.0.0.1:9098/set/mykey/hello
{"result":"OK"}
```

```shell
# 获取键值
curl http://127.0.0.1:9098/get/mykey
{"result":"hello"}
```

```shell
# 执行其他Redis命令
curl http://127.0.0.1:9098/hset/myhash/field1/value1
```

### Web界面

访问 `http://127.0.0.1:9098` 可以使用内置的Web控制台。
