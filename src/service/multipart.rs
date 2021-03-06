use actix_multipart::Field;
use actix_web::web;
use actix_web::web::BytesMut;
use bytes::buf::{Buf, BufMut};
use bytes::Bytes;
use futures::TryStreamExt;
use mime::Mime;
use serde_json::Value as JsonValue;
use serde_json::{from_slice, Error as JsonError};
use std::io::{Error as IoError, Write};
use std::path::{Path, PathBuf};
use std::string::FromUtf8Error;
use uuid::Uuid;

pub async fn field_to_bytes(mut field: Field) -> Bytes {
    let mut bytes = BytesMut::new();
    while let Ok(Some(field)) = field.try_next().await {
        bytes.put(field);
    }
    bytes.to_bytes()
}

pub async fn field_to_string(field: Field) -> Result<String, FromUtf8Error> {
    String::from_utf8(field_to_bytes(field).await.to_vec())
}

pub async fn field_to_json_value(field: Field) -> Result<JsonValue, JsonError> {
    let bytes = field_to_bytes(field).await;
    from_slice(&bytes)
}

fn get_filename(field: &Field) -> Option<String> {
    if let Some(content) = field.content_disposition() {
        if let Some(filename) = content.get_filename() {
            return Some(filename.to_string());
        }
    }
    None
}

#[derive(Debug)]
pub struct MultipartFile {
    pub filename: Option<String>,
    pub filepath: PathBuf,
    pub content_type: Mime,
}

impl MultipartFile {
    fn from_field(root: &Path, field: &Field) -> Self {
        let filename = get_filename(&field);
        let filepath = match filename.clone() {
            Some(value) => root.join(value),
            None => root.join(Uuid::new_v4().to_string()),
        };
        let content_type = field.content_type().clone();
        Self {
            filename,
            filepath,
            content_type,
        }
    }
}

pub async fn field_to_file(root: &Path, mut field: Field) -> Result<MultipartFile, IoError> {
    let multipart_file = MultipartFile::from_field(root, &field);
    let filepath = multipart_file.filepath.clone();
    let mut file = web::block(|| std::fs::File::create(filepath))
        .await
        .unwrap();
    while let Ok(Some(chunk)) = field.try_next().await {
        file = web::block(move || file.write_all(&chunk).map(|_| file))
            .await
            .unwrap();
    }
    Ok(multipart_file)
}
