use crate::ksqldb::{
    error::{KsqlDbError, KsqlDbSnafu},
    types::{CreateResponse, ListStreamsResponse, ListTablesResponse},
    KsqlDb, Result,
};
use serde::de::DeserializeOwned;
use serde_json::{from_value, json};
use std::collections::HashMap;

impl KsqlDb {
    pub async fn execute_statement<T>(
        &self,
        statement: &str,
        properties: &HashMap<String, String>,
        command_sequence_number: Option<u32>,
    ) -> Result<Vec<T>>
    where
        T: DeserializeOwned,
    {
        let url = format!("{}/ksql", self.url);
        let mut payload = json!({
            "ksql": statement,
            "streamProperties": properties
        });
        if let Some(sequence) = command_sequence_number {
            payload["commandSequenceNumber"] = sequence.into();
        }
        let response: serde_json::Value = self
            .client
            .post(&url)
            .json(&payload)
            .send()
            .await?
            .json()
            .await?;

        let has_error = response.get("error_code").is_some();
        if has_error {
            let result = from_value::<KsqlDbError>(response)?;
            return KsqlDbSnafu { source: result }.fail();
        }

        let result = from_value::<Vec<T>>(response)?;
        Ok(result)
    }

    /// Runs a `CREATE` statement
    ///
    /// ## Examples
    ///
    /// ```no_run
    /// use reqwest::Client;
    /// use common::ksqldb::KsqlDb;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let ksql = KsqlDb::new("http://localhost:8087".into());
    ///
    /// let query = r#"
    /// CREATE STREAM MY_STREAM (
    ///     id VARCHAR KEY
    /// ) WITH (
    ///     kafka_topic = 'my_topic',
    ///     partitions = 1,
    ///     value_format = 'JSON'
    /// );
    /// "#;
    ///
    /// let response = ksql.create(&query, &Default::default(), None).await;
    /// # }
    /// ```
    pub async fn create(
        &self,
        statement: &str,
        stream_properties: &HashMap<String, String>,
        command_sequence_number: Option<u32>,
    ) -> Result<Vec<CreateResponse>> {
        self.execute_statement::<CreateResponse>(
            statement,
            stream_properties,
            command_sequence_number,
        )
        .await
    }

    /// Runs a `LIST STREAMS` or `SHOW STREAMS` statement. They both have the same
    /// response structure so this method can be used to execute either.
    ///
    /// ## Examples
    ///
    /// ```no_run
    /// use reqwest::Client;
    /// use common::ksqldb::KsqlDb;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let ksql = KsqlDb::new("http://localhost:8087".into());
    ///
    /// let query = r#"SHOW STREAMS;"#;
    ///
    /// let response = ksql.list_streams(&query, &Default::default(), None).await;
    /// # }
    /// ```
    pub async fn list_streams(
        &self,
        statement: &str,
        stream_properties: &HashMap<String, String>,
        command_sequence_number: Option<u32>,
    ) -> Result<Vec<ListStreamsResponse>> {
        self.execute_statement::<ListStreamsResponse>(
            statement,
            stream_properties,
            command_sequence_number,
        )
        .await
    }

    /// Runs a `LIST TABLES` or `SHOW TABLES` statement. They both have the same
    /// response structure so this method can be used to execute either.
    ///
    /// ## Examples
    ///
    /// ```no_run
    /// use reqwest::Client;
    /// use common::ksqldb::KsqlDb;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let ksql = KsqlDb::new("http://localhost:8087".into());
    ///
    /// let query = r#"SHOW TABLES EXTENDED;"#;
    ///
    /// let response = ksql.list_tables(&query, &Default::default(), None).await;
    /// # }
    /// ```
    pub async fn list_tables(
        &self,
        statement: &str,
        stream_properties: &HashMap<String, String>,
        command_sequence_number: Option<u32>,
    ) -> Result<Vec<ListTablesResponse>> {
        self.execute_statement::<ListTablesResponse>(
            statement,
            stream_properties,
            command_sequence_number,
        )
        .await
    }
}
