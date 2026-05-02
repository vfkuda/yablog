use reqwest::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum BlogClientError {
    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("grpc request failed: {0}")]
    #[allow(clippy::result_large_err)]
    GrpcStatus(#[from] tonic::Status),
    #[error("grpc transport failed")]
    GrpcTransport(#[from] tonic::transport::Error),
    #[error("invalid grpc authorization metadata")]
    InvalidMetadata(#[from] tonic::metadata::errors::InvalidMetadataValue),
    #[error("transport does not support this operation")]
    InvalidTransport,
    #[error(
        "missing JWT token, reqeust wasn't sent to server. call login/register or set_token first"
    )]
    MissingToken,
    #[error("unexpected empty grpc response payload")]
    UnexpectedGrpcPayload,
    #[error("resource not found")]
    NotFound,
    #[error("unauthorized")]
    Unauthorized,
    #[error("permission denied/forbidden")]
    Forbidden,
    #[error("invalid request: {0}")]
    InvalidRequest(String),
    #[error("http API error ({status}): {body}")]
    HttpApi { status: StatusCode, body: String },
}
