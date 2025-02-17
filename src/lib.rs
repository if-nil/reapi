use axum::{
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse, Json, Response},
    routing::get,
    Router,
};
use redis_module::redisraw::bindings::RedisModule_SelectDb;
use redis_module::{
    redis_module, redisvalue::RedisValueKey, Context, RedisString, RedisValue, Status,
    ThreadSafeContext,
};
use serde::Serialize;
use serde_json::json;

use std::{collections::HashMap, thread};
use tokio::runtime::Runtime;

const RAW_HTML: &str = include_str!("index.html");
const RAW_ICON: &[u8] = include_bytes!("favicon.ico");

struct ErrorResponse {
    code: StatusCode,
    error: String,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        (self.code, Json(json!({ "error": self.error }))).into_response()
    }
}

#[derive(Serialize)]
struct ResponseData {
    #[serde(serialize_with = "serialize_redis_value")]
    result: RedisValue,
}

// 将 RedisValue 和 RedisValueKey 转换为 JSON
trait ToJson {
    fn to_json(&self) -> serde_json::Value;
}

impl ToJson for RedisValueKey {
    fn to_json(&self) -> serde_json::Value {
        match self {
            RedisValueKey::Integer(i) => serde_json::Value::Number((*i).into()),
            RedisValueKey::String(s) => serde_json::Value::String(s.to_string()),
            RedisValueKey::BulkRedisString(s) => serde_json::Value::String(s.to_string()),
            RedisValueKey::BulkString(s) => match String::from_utf8(s.clone()) {
                Ok(s) => serde_json::Value::String(s),
                Err(_) => serde_json::Value::String(format!("{:?}", s)),
            },
            RedisValueKey::Bool(b) => serde_json::Value::Bool(*b),
        }
    }
}

impl ToJson for RedisValue {
    fn to_json(&self) -> serde_json::Value {
        match self {
            RedisValue::SimpleStringStatic(s) => serde_json::Value::String(s.to_string()),
            RedisValue::SimpleString(s) => serde_json::Value::String(s.to_string()),
            RedisValue::BulkString(s) => serde_json::Value::String(s.to_string()),
            RedisValue::BulkRedisString(s) => serde_json::Value::String(s.to_string()),
            RedisValue::StringBuffer(s) => match String::from_utf8(s.clone()) {
                Ok(s) => serde_json::Value::String(s),
                Err(_) => serde_json::Value::String(format!("{:?}", s)),
            },
            RedisValue::Integer(i) => serde_json::Value::Number((*i).into()),
            RedisValue::Bool(b) => serde_json::Value::Bool(*b),
            RedisValue::Float(f) => match serde_json::Number::from_f64(*f) {
                Some(n) => serde_json::Value::Number(n),
                None => serde_json::Value::String(format!("{}", f)),
            },
            RedisValue::BigNumber(s) => serde_json::Value::String(s.to_string()),
            RedisValue::VerbatimString(s) => serde_json::Value::String(format!("{:?}", s)),
            RedisValue::Array(arr) => {
                serde_json::Value::Array(arr.iter().map(RedisValue::to_json).collect())
            }
            RedisValue::StaticError(s) => serde_json::Value::String(s.to_string()),
            RedisValue::Map(map) => {
                let mut obj = serde_json::Map::new();
                for (k, v) in map {
                    obj.insert(k.to_json().to_string(), v.to_json());
                }
                serde_json::Value::Object(obj)
            }
            RedisValue::Set(set) => {
                serde_json::Value::Array(set.iter().map(RedisValueKey::to_json).collect())
            }
            RedisValue::OrderedMap(map) => serde_json::Value::Object(
                map.iter()
                    .map(|(k, v)| (k.to_json().to_string(), v.to_json()))
                    .collect(),
            ),
            RedisValue::OrderedSet(set) => serde_json::Value::Array(
                set.iter()
                    .map(|i| i.to_json())
                    .collect::<Vec<serde_json::Value>>(),
            ),
            RedisValue::Null => serde_json::Value::Null,
            RedisValue::NoReply => serde_json::Value::Null,
        }
    }
}

// 自定义序列化函数
fn serialize_redis_value<S>(value: &RedisValue, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    value.to_json().serialize(serializer)
}

impl IntoResponse for ResponseData {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

async fn index() -> Html<&'static str> {
    Html(RAW_HTML)
}

async fn favicon() -> Vec<u8> {
    RAW_ICON.to_vec()
}

async fn command(Path(path): Path<String>) -> Result<ResponseData, ErrorResponse> {
    let mut args = path.split("/").collect::<Vec<&str>>();
    let mut db = 0;
    if let Ok(index) = args[0].parse::<u64>() {
        db = index;
        args.remove(0);
    }

    let thread_ctx = ThreadSafeContext::new();

    let ctx = thread_ctx.lock();
    ctx.log_notice(format!("command: {:?}", args).as_str());

    if db > 0 && unsafe { RedisModule_SelectDb.unwrap()(ctx.ctx, db as i32) } != 0 {
        drop(ctx);
        return Err(ErrorResponse {
            error: format!("切换数据库 {} 失败", db),
            code: StatusCode::BAD_REQUEST,
        });
    }
    let result = ctx.call(args[0], &args[1..]);
    drop(ctx);

    match result {
        Ok(s) => match s {
            RedisValue::StaticError(s) => Err(ErrorResponse {
                error: s.to_string(),
                code: StatusCode::BAD_REQUEST,
            }),
            _ => Ok(ResponseData { result: s }),
        },
        Err(_e) => Err(ErrorResponse {
            error: _e.to_string(),
            code: StatusCode::INTERNAL_SERVER_ERROR,
        }),
    }
}

fn parse_args(args: Vec<String>) -> Result<HashMap<String, String>, anyhow::Error> {
    let mut config = HashMap::new();

    // 遍历参数，每两个一组提取键值对
    let mut iter = args.into_iter();
    while let Some(key) = iter.next() {
        if let Some(value) = iter.next() {
            config.insert(key, value);
        } else {
            return Err(anyhow::anyhow!("参数不完整: 缺少键 `{}` 对应的值", key));
        }
    }
    Ok(config)
}

fn start_server(args: Vec<String>) -> Result<(), anyhow::Error> {
    let app = Router::<()>::new()
        .route("/favicon.ico", get(favicon))
        .route("/", get(index))
        .route("/index.html", get(index))
        .route("/{*path}", get(command));

    let config = parse_args(args)?;
    let default_host = String::from("127.0.0.1");
    let default_port = String::from("9098");
    let reapi_host = config.get("reapi_host").unwrap_or(&default_host);
    let reapi_port = config.get("reapi_port").unwrap_or(&default_port);

    // 打印启动信息
    let addr = format!("{}:{}", reapi_host, reapi_port);
    let thread_ctx = ThreadSafeContext::new();
    let ctx = thread_ctx.lock();
    ctx.log_notice(format!("ReApi Server 启动中... {}", addr).as_str());
    drop(ctx);

    let rt = Runtime::new()?;
    rt.block_on(async {
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app)
            .await
            .map_err(|e| anyhow::Error::from(e))
    })?;
    Ok(())
}

fn init(_ctx: &Context, _args: &[RedisString]) -> Status {
    let args = _args.iter().map(|s| s.to_string()).collect::<Vec<String>>();
    _ctx.log_notice(format!("ReApi Server 启动参数: {:?}", args).as_str());
    thread::spawn(move || {
        if let Err(e) = start_server(args) {
            let thread_ctx = ThreadSafeContext::new();
            let ctx = thread_ctx.lock();
            ctx.log_warning(format!("ReApi Server 启动失败: {}", e).as_str());
            drop(ctx);
        }
    });
    Status::Ok
}

fn deinit(_: &Context) -> Status {
    Status::Ok
}

redis_module! {
    name: "reapi",
    version: 1,
    allocator: (redis_module::alloc::RedisAlloc, redis_module::alloc::RedisAlloc),
    data_types: [],
    init: init,
    deinit: deinit,
    commands: []
}
