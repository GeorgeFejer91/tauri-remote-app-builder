use std::fmt::Write as _;

use serde::Serialize;

use crate::adapter::{
    AdapterError, AdapterErrorCode, DeviceList, DeviceRevocation, DoctorReport, ProductState,
    RemoteStatus,
};
use crate::intent::{CliParseError, CommandName, OutputFormat, ParseErrorCode};

pub const OUTPUT_SCHEMA_VERSION: &str = "appctl.v1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum OutputData {
    RemoteStatus(RemoteStatus),
    DeviceList(DeviceList),
    DeviceRevocation(DeviceRevocation),
    DoctorReport(DoctorReport),
    ProductState(ProductState),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SuccessEnvelope {
    pub schema_version: &'static str,
    pub ok: bool,
    pub command: CommandName,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_id: Option<String>,
    pub authority_generation: String,
    pub revision: u64,
    pub data: OutputData,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicErrorCode {
    InvalidArguments,
    UnknownCommand,
    UnknownOption,
    MissingOption,
    DuplicateOption,
    InvalidValue,
    NonUnicodeArgument,
    TooManyArguments,
    NotConnected,
    Unauthorized,
    Forbidden,
    Conflict,
    Busy,
    InvalidState,
    Unavailable,
    TimedOutUnknownOutcome,
    Internal,
}

impl PublicErrorCode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidArguments => "invalid_arguments",
            Self::UnknownCommand => "unknown_command",
            Self::UnknownOption => "unknown_option",
            Self::MissingOption => "missing_option",
            Self::DuplicateOption => "duplicate_option",
            Self::InvalidValue => "invalid_value",
            Self::NonUnicodeArgument => "non_unicode_argument",
            Self::TooManyArguments => "too_many_arguments",
            Self::NotConnected => "not_connected",
            Self::Unauthorized => "unauthorized",
            Self::Forbidden => "forbidden",
            Self::Conflict => "conflict",
            Self::Busy => "busy",
            Self::InvalidState => "invalid_state",
            Self::Unavailable => "unavailable",
            Self::TimedOutUnknownOutcome => "timed_out_unknown_outcome",
            Self::Internal => "internal",
        }
    }

    const fn message(self) -> &'static str {
        match self {
            Self::InvalidArguments => "arguments do not match the fixed command grammar",
            Self::UnknownCommand => "command is not in the allowed command set",
            Self::UnknownOption => "option is not allowed for this command",
            Self::MissingOption => "a required option or value is missing",
            Self::DuplicateOption => "an option was supplied more than once",
            Self::InvalidValue => "an option value is outside its allowed type or range",
            Self::NonUnicodeArgument => "arguments must be valid Unicode",
            Self::TooManyArguments => "too many command-line arguments were supplied",
            Self::NotConnected => "the CLI is not connected to the running application service",
            Self::Unauthorized => "authentication is required",
            Self::Forbidden => "the operation is not authorized",
            Self::Conflict => "the requested revision conflicts with current state",
            Self::Busy => "the application service is busy",
            Self::InvalidState => "the operation is not valid in the current state",
            Self::Unavailable => "the application service is temporarily unavailable",
            Self::TimedOutUnknownOutcome => {
                "the timeout elapsed before the authority outcome was known; retry only with the same command ID"
            }
            Self::Internal => "the application service could not complete the operation",
        }
    }

    /// Stable process status for scripts and authorized automation.
    pub const fn exit_code(self) -> u8 {
        match self {
            Self::InvalidArguments
            | Self::UnknownCommand
            | Self::UnknownOption
            | Self::MissingOption
            | Self::DuplicateOption
            | Self::InvalidValue
            | Self::NonUnicodeArgument
            | Self::TooManyArguments => 2,
            Self::Unauthorized => 3,
            Self::Forbidden => 4,
            Self::Conflict | Self::InvalidState => 5,
            Self::NotConnected | Self::Busy | Self::Unavailable => 6,
            Self::TimedOutUnknownOutcome => 7,
            Self::Internal => 8,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicError {
    pub code: PublicErrorCode,
    pub message: &'static str,
    pub retryable: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FailureEnvelope {
    pub schema_version: &'static str,
    pub ok: bool,
    pub command: Option<CommandName>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_id: Option<String>,
    pub error: PublicError,
}

impl FailureEnvelope {
    pub(crate) fn from_parse(error: CliParseError) -> Self {
        let code = PublicErrorCode::from(error.code);
        Self {
            schema_version: OUTPUT_SCHEMA_VERSION,
            ok: false,
            command: error.command,
            command_id: None,
            error: PublicError {
                code,
                message: code.message(),
                retryable: false,
            },
        }
    }

    pub(crate) fn from_adapter(
        command: CommandName,
        command_id: Option<String>,
        error: AdapterError,
    ) -> Self {
        let code = PublicErrorCode::from(error.code());
        Self {
            schema_version: OUTPUT_SCHEMA_VERSION,
            ok: false,
            command: Some(command),
            command_id,
            error: PublicError {
                code,
                message: code.message(),
                retryable: error.retryable(),
            },
        }
    }
}

impl From<ParseErrorCode> for PublicErrorCode {
    fn from(code: ParseErrorCode) -> Self {
        match code {
            ParseErrorCode::InvalidArguments => Self::InvalidArguments,
            ParseErrorCode::UnknownCommand => Self::UnknownCommand,
            ParseErrorCode::UnknownOption => Self::UnknownOption,
            ParseErrorCode::MissingOption => Self::MissingOption,
            ParseErrorCode::DuplicateOption => Self::DuplicateOption,
            ParseErrorCode::InvalidValue => Self::InvalidValue,
            ParseErrorCode::NonUnicodeArgument => Self::NonUnicodeArgument,
            ParseErrorCode::TooManyArguments => Self::TooManyArguments,
        }
    }
}

impl From<AdapterErrorCode> for PublicErrorCode {
    fn from(code: AdapterErrorCode) -> Self {
        match code {
            AdapterErrorCode::NotConnected => Self::NotConnected,
            AdapterErrorCode::Unauthorized => Self::Unauthorized,
            AdapterErrorCode::Forbidden => Self::Forbidden,
            AdapterErrorCode::Conflict => Self::Conflict,
            AdapterErrorCode::Busy => Self::Busy,
            AdapterErrorCode::InvalidState => Self::InvalidState,
            AdapterErrorCode::Unavailable => Self::Unavailable,
            AdapterErrorCode::TimedOutUnknownOutcome => Self::TimedOutUnknownOutcome,
            AdapterErrorCode::Internal => Self::Internal,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CliRun {
    pub exit_code: u8,
    pub stdout: String,
    pub stderr: String,
}

pub(crate) fn success_run(format: OutputFormat, envelope: &SuccessEnvelope) -> CliRun {
    CliRun {
        exit_code: 0,
        stdout: render_success(format, envelope),
        stderr: String::new(),
    }
}

pub(crate) fn failure_run(
    format: OutputFormat,
    exit_code: u8,
    envelope: &FailureEnvelope,
) -> CliRun {
    CliRun {
        exit_code,
        stdout: String::new(),
        stderr: render_failure(format, envelope),
    }
}

fn render_success(format: OutputFormat, envelope: &SuccessEnvelope) -> String {
    if format == OutputFormat::Json {
        return json_line(envelope);
    }

    let mut output = String::new();
    write_header(
        &mut output,
        true,
        Some(envelope.command),
        envelope.command_id.as_deref(),
        Some((&envelope.authority_generation, envelope.revision)),
    );
    match &envelope.data {
        OutputData::RemoteStatus(status) => {
            let exposure = status.exposure.map_or("none", |value| value.as_str());
            writeln!(output, "data.type=remote_status").ok();
            writeln!(output, "data.state={}", status.state.as_str()).ok();
            writeln!(output, "data.exposure={exposure}").ok();
            writeln!(
                output,
                "data.allow_new_devices={}",
                status.allow_new_devices
            )
            .ok();
            writeln!(
                output,
                "data.connected_devices={}",
                status.connected_devices
            )
            .ok();
        }
        OutputData::DeviceList(list) => {
            writeln!(output, "data.type=device_list").ok();
            writeln!(output, "data.filter={}", list.filter.as_str()).ok();
            writeln!(output, "data.device_count={}", list.devices.len()).ok();
            for (index, device) in list.devices.iter().enumerate() {
                writeln!(
                    output,
                    "data.devices.{index}.device_id={}",
                    quoted(&device.device_id)
                )
                .ok();
                writeln!(
                    output,
                    "data.devices.{index}.trust={}",
                    device.trust.as_str()
                )
                .ok();
                writeln!(
                    output,
                    "data.devices.{index}.connected={}",
                    device.connected
                )
                .ok();
            }
        }
        OutputData::DeviceRevocation(revocation) => {
            writeln!(output, "data.type=device_revocation").ok();
            writeln!(output, "data.device_id={}", quoted(&revocation.device_id)).ok();
            writeln!(output, "data.revoked={}", revocation.revoked).ok();
        }
        OutputData::DoctorReport(report) => {
            writeln!(output, "data.type=doctor_report").ok();
            writeln!(output, "data.healthy={}", report.healthy).ok();
            writeln!(output, "data.finding_count={}", report.findings.len()).ok();
            for (index, finding) in report.findings.iter().enumerate() {
                writeln!(
                    output,
                    "data.findings.{index}.component={}",
                    finding.component.as_str()
                )
                .ok();
                writeln!(
                    output,
                    "data.findings.{index}.status={}",
                    finding.status.as_str()
                )
                .ok();
                writeln!(
                    output,
                    "data.findings.{index}.required={}",
                    finding.required
                )
                .ok();
            }
        }
        OutputData::ProductState(state) => {
            writeln!(output, "data.type=product_state").ok();
            writeln!(output, "data.active={}", state.active).ok();
            writeln!(output, "data.level={}", state.level).ok();
        }
    }
    output
}

fn render_failure(format: OutputFormat, envelope: &FailureEnvelope) -> String {
    if format == OutputFormat::Json {
        return json_line(envelope);
    }

    let mut output = String::new();
    write_header(
        &mut output,
        false,
        envelope.command,
        envelope.command_id.as_deref(),
        None,
    );
    writeln!(output, "error.code={}", envelope.error.code.as_str()).ok();
    writeln!(output, "error.message={}", quoted(envelope.error.message)).ok();
    writeln!(output, "error.retryable={}", envelope.error.retryable).ok();
    output
}

fn write_header(
    output: &mut String,
    ok: bool,
    command: Option<CommandName>,
    command_id: Option<&str>,
    revision: Option<(&str, u64)>,
) {
    writeln!(output, "schema_version={OUTPUT_SCHEMA_VERSION}").ok();
    writeln!(output, "ok={ok}").ok();
    writeln!(
        output,
        "command={}",
        command.map_or("none", CommandName::as_str)
    )
    .ok();
    if let Some(command_id) = command_id {
        writeln!(output, "command_id={}", quoted(command_id)).ok();
    }
    if let Some((generation, value)) = revision {
        writeln!(output, "authority_generation={}", quoted(generation)).ok();
        writeln!(output, "revision={value}").ok();
    }
}

fn quoted(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| "\"<unavailable>\"".to_owned())
}

fn json_line<T: Serialize>(value: &T) -> String {
    let mut output = serde_json::to_string(value).unwrap_or_else(|_| {
        concat!(
            "{\"schemaVersion\":\"appctl.v1\",\"ok\":false,",
            "\"command\":null,\"error\":{\"code\":\"internal\",",
            "\"message\":\"output serialization failed\",\"retryable\":false}}"
        )
        .to_owned()
    });
    output.push('\n');
    output
}
