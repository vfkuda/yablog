use std::fmt::Display;

use tracing::{error, info, warn};

#[derive(Debug, Clone, Copy)]
pub enum RequestOutcome {
    Success,
    ClientError,
    ServerError,
}

impl RequestOutcome {
    fn as_str(self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::ClientError => "client_error",
            Self::ServerError => "server_error",
        }
    }
}

pub fn log_request_handled<E>(
    protocol: &'static str,
    operation: &'static str,
    response_status: &'static str,
    outcome: RequestOutcome,
    error: Option<&E>,
) where
    E: Display + ?Sized,
{
    match (outcome, error) {
        (RequestOutcome::Success, _) => {
            info!(
                protocol,
                operation,
                outcome = outcome.as_str(),
                response_status,
                "request handled"
            );
        }
        (RequestOutcome::ClientError, Some(err)) => {
            warn!(
                protocol,
                operation,
                outcome = outcome.as_str(),
                response_status,
                error = %err,
                "request handled with client error"
            );
        }
        (RequestOutcome::ClientError, None) => {
            warn!(
                protocol,
                operation,
                outcome = outcome.as_str(),
                response_status,
                "request handled with client error"
            );
        }
        (RequestOutcome::ServerError, Some(err)) => {
            error!(
                protocol,
                operation,
                outcome = outcome.as_str(),
                response_status,
                error = %err,
                "request handled with server error"
            );
        }
        (RequestOutcome::ServerError, None) => {
            error!(
                protocol,
                operation,
                outcome = outcome.as_str(),
                response_status,
                "request handled with server error"
            );
        }
    }
}
