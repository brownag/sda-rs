use axum::{
    body::{Body, Bytes},
    extract::{FromRequest, Request, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::{
    executor,
    request::QueryRequest,
    serializer::{self, Format},
    transpiler, AppState,
};

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

fn error(status: StatusCode, message: impl Into<String>) -> Response {
    (
        status,
        Json(ErrorBody {
            error: message.into(),
        }),
    )
        .into_response()
}

pub async fn post(State(state): State<AppState>, request: Request) -> Response {
    let content_type = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_owned();
    let input = if content_type.starts_with("application/json") {
        match Json::<QueryRequest>::from_request(request, &()).await {
            Ok(v) => v.0,
            Err(_) => return error(StatusCode::BAD_REQUEST, "Invalid request body"),
        }
    } else if content_type.starts_with("application/x-www-form-urlencoded") {
        let bytes = match Bytes::from_request(request, &()).await {
            Ok(bytes) => bytes,
            Err(_) => return error(StatusCode::BAD_REQUEST, "Invalid request body"),
        };
        match crate::request::parse_form(&bytes) {
            Ok(input) => input,
            Err(_) => return error(StatusCode::BAD_REQUEST, "Invalid form encoding"),
        }
    } else {
        return error(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Unsupported content type",
        );
    };
    let Some(query) = input.query.filter(|query| !query.trim().is_empty()) else {
        return error(StatusCode::BAD_REQUEST, "Missing query");
    };
    if input.format.as_deref() == Some("~") {
        let mut response = Response::new(Body::from(query));
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain; charset=utf-8"),
        );
        return response;
    }
    let format = match input
        .format
        .as_deref()
        .unwrap_or("JSON")
        .to_ascii_uppercase()
        .as_str()
    {
        "JSON" => Format::Json,
        "JSON+COLUMNNAME" => Format::JsonColumnName,
        "JSON+COLUMNNAME+METADATA" => Format::JsonColumnNameMetadata,
        "XML" => Format::Xml,
        _ => return error(StatusCode::BAD_REQUEST, "Unsupported format"),
    };
    let statements = match transpiler::transpile(&query) {
        Ok(statements) => statements,
        Err(err) => return error(StatusCode::BAD_REQUEST, err.to_string()),
    };
    let results = match executor::execute_blocking(state.database_path, statements).await {
        Ok(results) => results,
        Err(err) => return error(StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
    };
    let body = serializer::serialize(&results, format);
    let content_type = if format == Format::Xml {
        "application/xml; charset=utf-8"
    } else {
        "application/json; charset=utf-8"
    };
    let mut response = Response::new(Body::from(body));
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::to_bytes, http::Request as HttpRequest};
    use rusqlite::Connection;
    use std::{
        fs,
        sync::atomic::{AtomicUsize, Ordering},
    };

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    async fn send(path: &str, content_type: &str, body: &str) -> Response {
        let request = HttpRequest::builder()
            .header(header::CONTENT_TYPE, content_type)
            .body(Body::from(body.to_owned()))
            .unwrap();
        post(
            State(AppState {
                database_path: path.to_owned(),
            }),
            request,
        )
        .await
    }

    #[tokio::test]
    async fn parses_encodings_echoes_and_executes_against_configured_database() {
        let path = std::env::temp_dir().join(format!(
            "sda-routes-{}-{}.sqlite",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let connection = Connection::open(&path).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE sample(value TEXT); INSERT INTO sample VALUES ('configured')",
            )
            .unwrap();
        drop(connection);
        let path = path.to_string_lossy().into_owned();
        let parsed =
            crate::request::parse_form(b"QuErY=SELECT%20value%20FROM%20sample&FoRmAt=JSON")
                .unwrap();
        assert_eq!(parsed.query.as_deref(), Some("SELECT value FROM sample"));
        assert_eq!(parsed.format.as_deref(), Some("JSON"));

        for (content_type, body) in [
            (
                "application/json",
                r#"{"QuErY":"SELECT value FROM sample","FoRmAt":"JSON"}"#,
            ),
            (
                "application/x-www-form-urlencoded",
                "QuErY=SELECT%20value%20FROM%20sample&FoRmAt=JSON",
            ),
        ] {
            let response = send(&path, content_type, body).await;
            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(
                to_bytes(response.into_body(), usize::MAX).await.unwrap(),
                r#"{"Table":[["configured"]]}"#
            );
        }

        let echo = send(
            &path,
            "application/json",
            r#"{"query":"DROP TABLE sample","format":"~"}"#,
        )
        .await;
        assert_eq!(
            echo.headers()[header::CONTENT_TYPE],
            "text/plain; charset=utf-8"
        );
        assert_eq!(
            to_bytes(echo.into_body(), usize::MAX).await.unwrap(),
            "DROP TABLE sample"
        );

        let missing = send(&path, "application/json", r#"{"format":"JSON"}"#).await;
        assert_eq!(missing.status(), StatusCode::BAD_REQUEST);
        let unsafe_query = send(
            &path,
            "application/json",
            r#"{"query":"DROP TABLE sample"}"#,
        )
        .await;
        assert_eq!(unsafe_query.status(), StatusCode::BAD_REQUEST);

        fs::remove_file(path).unwrap();
    }
}
