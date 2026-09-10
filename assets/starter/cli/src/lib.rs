//! Typed command-line adapter for a running Tauri Rust authority.
//!
//! This crate intentionally owns no mutable application state. Parse with
//! [`parse_args`] or call [`run_cli`], then implement [`AdminAdapter`] against
//! the same service instance that owns the desktop application's authority and
//! revisions.

mod adapter;
mod intent;
mod output;

use std::ffi::OsString;

pub use adapter::{
    AdapterError, AdapterErrorCode, AdapterResult, AdminAdapter, CheckStatus, DeviceList,
    DeviceRevocation, DeviceSummary, DeviceTrust, DoctorComponent, DoctorFinding, DoctorReport,
    ProductState, RemoteRunState, RemoteStatus, Revisioned, UnwiredAdapter,
};
pub use intent::{
    parse_args, CliInvocation, CliParseError, CommandAvailability, CommandName, DeviceFilter,
    DeviceListRequest, DeviceRevokeRequest, DoctorArea, DoctorRequest, Intent, OutputFormat,
    ParseErrorCode, RemoteExposure, RemoteStartRequest, RemoteStopRequest, RevokeReason,
    SetActiveRequest, SetLevelRequest,
};
pub use output::{
    CliRun, FailureEnvelope, OutputData, PublicError, PublicErrorCode, SuccessEnvelope,
    OUTPUT_SCHEMA_VERSION,
};

use crate::intent::detect_output_format;
use crate::output::{failure_run, success_run};

const USAGE_EXIT_CODE: u8 = 2;

/// Parse, dispatch, and render a CLI invocation.
///
/// `arguments` must omit the executable name. Parse errors never call the
/// adapter. Successful parsing dispatches through exactly one typed method.
pub fn run_cli<I, S, A>(arguments: I, adapter: &mut A) -> CliRun
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
    A: AdminAdapter,
{
    let arguments: Vec<OsString> = arguments.into_iter().map(Into::into).collect();
    let fallback_format = detect_output_format(&arguments);
    let invocation = match parse_args(arguments) {
        Ok(invocation) => invocation,
        Err(error) => {
            return failure_run(
                fallback_format,
                USAGE_EXIT_CODE,
                &FailureEnvelope::from_parse(error),
            )
        }
    };

    let format = invocation.output;
    match dispatch(invocation.intent, adapter) {
        Ok(success) => success_run(format, &success),
        Err(failure) => failure_run(format, failure.error.code.exit_code(), &failure),
    }
}

/// Dispatch an already parsed intent through one typed adapter method.
pub fn dispatch<A: AdminAdapter>(
    intent: Intent,
    adapter: &mut A,
) -> Result<SuccessEnvelope, FailureEnvelope> {
    let command = intent.command_name();
    let command_id = intent.command_id().map(str::to_owned);
    let result = match intent {
        Intent::RemoteStart(request) => adapter
            .remote_start(request)
            .map(|result| result.map(OutputData::RemoteStatus)),
        Intent::RemoteStatus => adapter
            .remote_status()
            .map(|result| result.map(OutputData::RemoteStatus)),
        Intent::RemoteStop(request) => adapter
            .remote_stop(request)
            .map(|result| result.map(OutputData::RemoteStatus)),
        Intent::DevicesList(request) => adapter
            .devices_list(request)
            .map(|result| result.map(OutputData::DeviceList)),
        Intent::DevicesRevoke(request) => adapter
            .devices_revoke(request)
            .map(|result| result.map(OutputData::DeviceRevocation)),
        Intent::Doctor(request) => adapter
            .doctor(request)
            .map(|result| result.map(OutputData::DoctorReport)),
        Intent::ProductStatus => adapter
            .product_status()
            .map(|result| result.map(OutputData::ProductState)),
        Intent::ProductSetLevel(request) => adapter
            .product_set_level(request)
            .map(|result| result.map(OutputData::ProductState)),
        Intent::ProductSetActive(request) => adapter
            .product_set_active(request)
            .map(|result| result.map(OutputData::ProductState)),
    };

    match result {
        Ok(revisioned) => {
            if revisioned.command_id.as_deref() != command_id.as_deref() {
                return Err(FailureEnvelope::from_adapter(
                    command,
                    command_id,
                    AdapterError::new(AdapterErrorCode::Internal),
                ));
            }
            Ok(SuccessEnvelope {
                schema_version: OUTPUT_SCHEMA_VERSION,
                ok: true,
                command,
                command_id: revisioned.command_id,
                authority_generation: revisioned.authority_generation,
                revision: revisioned.revision,
                data: revisioned.value,
            })
        }
        Err(error) => Err(FailureEnvelope::from_adapter(command, command_id, error)),
    }
}

trait MapRevisioned<T> {
    fn map<U>(self, map: impl FnOnce(T) -> U) -> Revisioned<U>;
}

impl<T> MapRevisioned<T> for Revisioned<T> {
    fn map<U>(self, map: impl FnOnce(T) -> U) -> Revisioned<U> {
        Revisioned {
            command_id: self.command_id,
            authority_generation: self.authority_generation,
            revision: self.revision,
            value: map(self.value),
        }
    }
}
