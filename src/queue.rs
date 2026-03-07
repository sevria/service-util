use async_trait::async_trait;
use redis::{
    AsyncTypedCommands,
    aio::MultiplexedConnection,
    streams::{StreamMaxlen, StreamReadOptions},
};
use serde::Serialize;
use serde_json::Value;

use crate::error::Error;

#[derive(Clone, Debug)]
pub struct QueueProducerConfig {
    pub stream_name: String,
    pub max_len: usize,
}

impl QueueProducerConfig {
    pub fn new(stream_name: String) -> Self {
        Self {
            stream_name,
            max_len: 1000,
        }
    }
}

#[async_trait]
pub trait QueueProducer: Send + Sync {
    async fn add(&self, name: &str, data: &Value) -> Result<(), Error>;

    async fn add_serialized<T>(&self, name: &str, data: &T) -> Result<(), Error>
    where
        T: Serialize + Send + Sync,
        Self: Sized,
    {
        let data = serde_json::to_value(data).map_err(|err| {
            log::error!("Failed to serialize queue data: {}", err);
            Error::Internal
        })?;

        self.add(name, &data).await
    }
}

pub struct RedisQueueProducer {
    config: QueueProducerConfig,
    redis: MultiplexedConnection,
}

impl RedisQueueProducer {
    pub fn new(stream_name: String, redis: MultiplexedConnection) -> Self {
        Self {
            config: QueueProducerConfig::new(stream_name),
            redis,
        }
    }

    pub fn with_config(config: QueueProducerConfig, redis: MultiplexedConnection) -> Self {
        Self { config, redis }
    }
}

#[async_trait]
impl QueueProducer for RedisQueueProducer {
    async fn add(&self, name: &str, data: &Value) -> Result<(), Error> {
        let mut redis = self.redis.clone();
        let payload = data.to_string();

        redis
            .xadd_maxlen(
                &self.config.stream_name,
                StreamMaxlen::Approx(self.config.max_len),
                "*",
                &[("name", name), ("data", payload.as_str())],
            )
            .await
            .map_err(|err| {
                log::error!("Failed to add job to queue: {}", err);
                Error::Internal
            })?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct QueueWorkerConfig {
    pub stream_name: String,
    pub group_name: String,
    pub consumer_name: String,
    pub read_count: usize,
    pub block_millis: usize,
}

impl QueueWorkerConfig {
    pub fn new(stream_name: String) -> Self {
        Self {
            stream_name,
            group_name: "worker".to_string(),
            consumer_name: "worker-1".to_string(),
            read_count: 10,
            block_millis: 5000,
        }
    }
}

#[async_trait]
pub trait QueueJobHandler: Send + Sync {
    async fn handle(&self, name: &str, data: Value) -> Result<(), Error>;
}

pub struct RedisQueueWorker {
    config: QueueWorkerConfig,
    redis: MultiplexedConnection,
}

impl RedisQueueWorker {
    pub fn new(config: QueueWorkerConfig, redis: MultiplexedConnection) -> Self {
        Self { config, redis }
    }

    pub async fn run<H>(&self, handler: &H) -> Result<(), Error>
    where
        H: QueueJobHandler,
    {
        let mut redis = self.redis.clone();

        redis
            .xgroup_create_mkstream(&self.config.stream_name, &self.config.group_name, "0")
            .await
            .ok();

        let opts = StreamReadOptions::default()
            .block(self.config.block_millis)
            .count(self.config.read_count)
            .group(&self.config.group_name, &self.config.consumer_name);

        log::info!("Running queue worker: {}", &self.config.stream_name);

        loop {
            let results = redis
                .xread_options(&[&self.config.stream_name], &[">"], &opts)
                .await
                .map_err(|err| {
                    log::error!("Failed to read queue stream: {}", err);
                    Error::Internal
                })?;

            if let Some(reply) = results {
                for stream in reply.keys {
                    for entry in stream.ids {
                        let job_name = entry
                            .get::<String>("name")
                            .unwrap_or_else(|| "unknown".to_string());
                        let raw_data = entry
                            .get::<String>("data")
                            .unwrap_or_else(|| "{}".to_string());

                        let parsed_data = serde_json::from_str::<Value>(&raw_data).map_err(|err| {
                            log::error!(
                                "Failed to parse queue payload as JSON, stream={}, id={}, err={}",
                                self.config.stream_name,
                                entry.id,
                                err
                            );
                            Error::Internal
                        });

                        if let Ok(data) = parsed_data {
                            if let Err(err) = handler.handle(&job_name, data).await {
                                log::error!(
                                    "Queue handler failed, stream={}, id={}, job={}, err={}",
                                    self.config.stream_name,
                                    entry.id,
                                    job_name,
                                    err
                                );
                            }
                        }

                        redis
                            .xack(
                                &self.config.stream_name,
                                &self.config.group_name,
                                &[entry.id.as_str()],
                            )
                            .await
                            .map_err(|err| {
                                log::error!(
                                    "Failed to acknowledge queue message, stream={}, id={}, err={}",
                                    self.config.stream_name,
                                    entry.id,
                                    err
                                );
                                Error::Internal
                            })?;
                    }
                }
            }
        }
    }
}
