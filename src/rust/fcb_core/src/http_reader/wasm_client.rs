// use crate::HttpFcbReader;
// use anyhow::Result;
// use bytes::Bytes;
// #[cfg(feature = "wasm")]
// use gloo_net::http::Request;
// use http_range_client::AsyncHttpRangeClient;
// use std::fs::File;
// use std::io::{BufReader, Read, Seek, SeekFrom};
// use std::ops::Range;
// use std::path::PathBuf;
// use std::sync::{Arc, RwLock};
// use tracing::trace;
// #[cfg(feature = "wasm")]
// use wasm_bindgen::prelude::*;

// #[wasm_bindgen]
// #[cfg(feature = "wasm")]
// pub type HttpClient = GlooRequest;

// #[cfg(feature = "wasm")]
// impl HttpFcbReader<GlooRequest> {
//     pub async fn open(url: &str) -> Result<HttpFcbReader<GlooRequest>> {
//         trace!("starting: opening http reader, reading header");

//         // let stats = Arc::new(RwLock::new(RequestStats::new()));
//         // let http_client = MockHttpRangeClient::new(path, stats.clone());
//         // let client = http_range_client::AsyncBufferedHttpRangeClient::with(http_client, path);
//         // Ok((Self::_open(client).await?, stats))
//         todo!("implement me")
//     }
// }

// #[cfg(feature = "wasm")]
// impl AsyncHttpRangeClient for GlooRequest {
//     async fn get_range(&self, url: &str, range: &str) -> Result<Bytes> {
//         todo!("implement me")
//     }

//     async fn head_response_header(&self, url: &str, header: &str) -> Result<Option<String>> {
//         todo!("implement me")
//     }
// }
