use anyhow::Result;
use async_trait::async_trait;
use metrics::counter;
use serde::{Deserialize, Serialize};
use sqlparser::ast::Statement;

use shotover_transforms::ast::ASTHolder;
use shotover_transforms::Value::List;
use shotover_transforms::{ChainResponse, MessageDetails, QueryMessage};

use crate::config::topology::TopicHolder;
use crate::transforms::{InternalTransform, Wrapper};
use crate::transforms::{Transforms, TransformsFromConfig};

#[derive(Debug, Clone)]
pub struct QueryCounter {
    name: &'static str,
    counter_name: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct QueryCounterConfig {
    pub name: String,
}

#[async_trait]
impl InternalTransform for QueryCounter {
    async fn transform<'a>(&'a mut self, qd: Wrapper<'a>) -> ChainResponse {
        for m in &qd.message.messages {
            if let MessageDetails::Query(QueryMessage {
                query_string: _query_string,
                namespace: _namespace,
                primary_key: _primary_key,
                query_values: _query_values,
                projection: _projection,
                query_type: _query_type,
                ast: Some(ast),
            }) = &m.details
            {
                match ast {
                    ASTHolder::SQL(statement) => {
                        let query_type = match statement {
                            Statement::Query(_) => "SELECT",
                            Statement::Insert { .. } => "INSERT",
                            Statement::Copy { .. } => "COPY",
                            Statement::Update { .. } => "UPDATE",
                            Statement::Delete { .. } => "DELETE",
                            Statement::CreateTable { .. } => "CREATE TABLE",
                            Statement::AlterTable { .. } => "ALTER TABLE",
                            Statement::Drop { .. } => "DROP",
                            _ => "UNRECOGNISED CQL",
                        };
                        counter!("query_count", 1, "name" => self.counter_name.clone(), "query" => query_type, "type" => "cassandra");
                    }
                    ASTHolder::Commands(List(commands)) => {
                        if let Some(v) = commands.get(0) {
                            let command = format!("{:?}", v);
                            counter!("query_count", 1, "name" => self.counter_name.clone(), "query" => command.to_ascii_uppercase(), "type" => "redis");
                        } else {
                            counter!("query_count", 1, "name" => self.counter_name.clone(), "query" => "empty", "type" => "redis");
                        }
                    }
                    _ => {
                        counter!("query_count", 1, "name" => self.counter_name.clone(), "query" => "unknown", "type" => "unknown");
                    }
                }
            }
        }

        qd.call_next_transform().await
    }

    fn get_name(&self) -> &'static str {
        self.name
    }
}

#[async_trait]
impl TransformsFromConfig for QueryCounterConfig {
    async fn get_source(&self, _topics: &TopicHolder) -> Result<Transforms> {
        Ok(Transforms::QueryCounter(QueryCounter {
            name: "QueryCounter",
            counter_name: self.name.clone(),
        }))
    }
}
