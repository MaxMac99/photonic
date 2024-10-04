use crate::ksqldb::{error::StreamSnafu, Error, KsqlDb, Result};
use bytes::Bytes;
use futures::Stream;
use futures_util::StreamExt;
use pin_project_lite::pin_project;
use reqwest::header::CONTENT_TYPE;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};

impl KsqlDb {
    /// This method lets you stream the output records of a `SELECT` statement
    /// via HTTP/2 streams. The response is streamed back until the
    /// `LIMIT` specified in the statement is reached, or the client closes the connection.
    ///
    /// If no `LIMIT` is specified in the statement, then the response is streamed until the client closes the connection.
    ///
    /// This method requires the `http2` feature be enabled.
    ///
    /// This crate also offers a HTTP/1 compatible approach to streaming results via
    /// `Transfer-Encoding: chunked`. To enable this turn off default features and enable the
    /// `http1` feature.
    ///
    /// ## Notes
    ///
    /// - The `T` provided, must be able to directly [`serde::Deserialize`] the response, it will
    /// error if there are missing mandatory fields
    /// - In the example below, if you were to change the query to be `SELECT ID FROM
    /// EVENT_REPLAY_STREAM EMIT CHANGES`, the query would error, because all of the other fields
    /// within the struct are mandatory fields.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// use futures_util::StreamExt;
    /// use reqwest::Client;
    /// use serde::Deserialize;
    /// use common::ksqldb::KsqlDb;
    ///
    /// #[derive(Debug, Deserialize)]
    /// struct Response {
    ///     id: String,
    ///     is_keyframe: bool,
    ///     sequence_number: u32,
    ///     events_since_keyframe: u32,
    ///     event_data: String,
    /// }
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let ksql = KsqlDb::new("http://localhost:8087".into());
    ///     let query = "SELECT * FROM EVENT_REPLAY_STREAM EMIT CHANGES;";
    ///
    ///     let mut stream = ksql
    ///         .query::<Response>(&query, &Default::default())
    ///         .await
    ///         .unwrap();
    ///     while let Some(r) = stream.next().await {
    ///         match r {
    ///             Ok(data) => {
    ///                 println!("{:#?}", data);
    ///             }
    ///             Err(e) => {
    ///                 eprintln!("Found Error {}", e);
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// [API Docs](https://docs.ksqldb.io/en/0.13.0-ksqldb/developer-guide/ksqldb-rest-api/streaming-endpoint/)
    pub async fn query<T>(
        &self,
        statement: &str,
        properties: &HashMap<String, String>,
    ) -> Result<impl Stream<Item = Result<T>>>
    where
        T: DeserializeOwned,
    {
        let url = format!("{}/query-stream", self.url);
        let payload = json!({
            "sql": statement,
            "properties": properties
        });
        let mut response = self
            .client
            .post(&url)
            .header(CONTENT_TYPE, "application/vnd.ksqlapi.delimited.v1")
            .json(&payload)
            .send()
            .await?
            .bytes_stream();

        let columns = match response.next().await {
            Some(data) => Ok(data?),
            None => StreamSnafu {
                message: ("Expected to receive data about the schema".to_string()),
            }
            .fail(),
        }?;
        let mut json = serde_json::from_slice::<Value>(&columns)?;
        if let Some(error_code) = json.get("error_code") {
            if let Some(error) = json.get("message") {
                return StreamSnafu {
                    message: format!("Error code: {}, message: {}", error_code, error),
                }
                .fail();
            }
        }
        let schema = json["columnNames"].take();
        let columns: Vec<String> = serde_json::from_value::<Vec<String>>(schema)?
            .into_iter()
            .map(|c| c.to_lowercase())
            .collect();
        let stream: QueryStream<T, _> = QueryStream::new(response, columns);
        Ok(stream)
    }
}

pin_project! {
    #[derive(Default)]
    struct QueryStream<T, S>
    where
        S: Stream,
        T: DeserializeOwned,
    {
        columns: Vec<String>,
        #[pin]
        stream: S,
        _marker: PhantomData<T>,
    }
}

impl<T, S> QueryStream<T, S>
where
    T: DeserializeOwned,
    S: Stream<Item = std::result::Result<Bytes, reqwest::Error>>,
{
    pub fn new(stream: S, columns: Vec<String>) -> Self {
        Self {
            columns,
            stream,
            _marker: PhantomData::default(),
        }
    }
}

impl<T, S> Stream for QueryStream<T, S>
where
    T: DeserializeOwned,
    S: Stream<Item = std::result::Result<Bytes, reqwest::Error>>,
{
    type Item = Result<T>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        Pin::new(&mut this.stream).poll_next(cx).map(|data| {
            let data = data?;
            let data = match data {
                Ok(data) => data,
                Err(e) => return Some(Err(Error::from(e))),
            };
            let json = match serde_json::from_slice::<Value>(&*data) {
                Ok(data) => data,
                Err(e) => return Some(Err(Error::from(e))),
            };
            if json.get("error_code").is_some() {
                if let Some(error) = json.get("message") {
                    return Some(
                        StreamSnafu {
                            message: error.to_string(),
                        }
                        .fail(),
                    );
                }
            }
            let arr = match json.as_array() {
                Some(data) => data.to_owned(),
                None => {
                    return Some(
                        StreamSnafu {
                            message: "Expected an array of column data".to_string(),
                        }
                        .fail(),
                    );
                }
            };
            let resp =
                this.columns
                    .iter()
                    .zip(arr.into_iter())
                    .fold(json!({}), |mut acc, (k, v)| {
                        acc[k] = v;
                        acc
                    });
            let resp = match serde_json::from_value::<T>(resp) {
                Ok(data) => data,
                Err(e) => return Some(Err(Error::from(e))),
            };
            Some(Ok(resp))
        })
    }
}
