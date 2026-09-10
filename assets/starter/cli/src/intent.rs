use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fmt;

use serde::Serialize;

const MAX_ARGUMENTS: usize = 32;
const MAX_ARGUMENT_BYTES: usize = 256;
const MAX_DEVICE_ID_BYTES: usize = 128;
const MIN_COMMAND_ID_BYTES: usize = 8;
const MAX_COMMAND_ID_BYTES: usize = 128;
const MIN_TIMEOUT_MS: u64 = 100;
const MAX_TIMEOUT_MS: u64 = 120_000;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OutputFormat {
    #[default]
    Human,
    Json,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum CommandName {
    #[serde(rename = "remote.start")]
    RemoteStart,
    #[serde(rename = "remote.status")]
    RemoteStatus,
    #[serde(rename = "remote.stop")]
    RemoteStop,
    #[serde(rename = "devices.list")]
    DevicesList,
    #[serde(rename = "devices.revoke")]
    DevicesRevoke,
    #[serde(rename = "doctor")]
    Doctor,
    #[serde(rename = "product.status")]
    ProductStatus,
    #[serde(rename = "product.set_level")]
    ProductSetLevel,
    #[serde(rename = "product.set_active")]
    ProductSetActive,
}

/// Declares whether a command belongs to the shared semantic control surface
/// or is an intentional local-administration exclusion.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandAvailability {
    PairedBrowserAndCli,
    LocalAdminOnly,
}

impl CommandName {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RemoteStart => "remote.start",
            Self::RemoteStatus => "remote.status",
            Self::RemoteStop => "remote.stop",
            Self::DevicesList => "devices.list",
            Self::DevicesRevoke => "devices.revoke",
            Self::Doctor => "doctor",
            Self::ProductStatus => "product.status",
            Self::ProductSetLevel => "product.set_level",
            Self::ProductSetActive => "product.set_active",
        }
    }

    /// Every ordinary product action must be available to both control edges.
    /// Only lifecycle, device-trust, and diagnostic administration is local.
    pub const fn availability(self) -> CommandAvailability {
        match self {
            Self::RemoteStart
            | Self::RemoteStatus
            | Self::RemoteStop
            | Self::DevicesList
            | Self::DevicesRevoke
            | Self::Doctor => CommandAvailability::LocalAdminOnly,
            Self::ProductStatus | Self::ProductSetLevel | Self::ProductSetActive => {
                CommandAvailability::PairedBrowserAndCli
            }
        }
    }
}

impl fmt::Display for CommandName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemoteExposure {
    LocalNetwork,
    ConfiguredRelay,
}

impl RemoteExposure {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LocalNetwork => "local-network",
            Self::ConfiguredRelay => "configured-relay",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeviceFilter {
    Current,
    Revoked,
    All,
}

impl DeviceFilter {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Revoked => "revoked",
            Self::All => "all",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RevokeReason {
    UserRequested,
    Lost,
    Compromised,
}

impl RevokeReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UserRequested => "user-requested",
            Self::Lost => "lost",
            Self::Compromised => "compromised",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DoctorArea {
    All,
    Authority,
    Transport,
    Browser,
}

impl DoctorArea {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Authority => "authority",
            Self::Transport => "transport",
            Self::Browser => "browser",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoteStartRequest {
    pub exposure: RemoteExposure,
    pub allow_new_devices: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RemoteStopRequest {
    pub force: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeviceListRequest {
    pub filter: DeviceFilter,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeviceRevokeRequest {
    pub device_id: String,
    pub reason: RevokeReason,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DoctorRequest {
    pub area: DoctorArea,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SetLevelRequest {
    pub value: u8,
    pub expected_revision: u64,
    pub command_id: String,
    pub timeout_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SetActiveRequest {
    pub active: bool,
    pub expected_revision: u64,
    pub command_id: String,
    pub timeout_ms: u64,
}

/// Closed, generated control vocabulary for both the paired browser and CLI.
///
/// The starter's `product` variants cover every action in its example Rust
/// reducer. A generated application must replace them with a complete
/// one-to-one mapping of its own semantic action enum. Do not leave ordinary
/// product actions browser-only or CLI-only, and do not add a raw fallback.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Intent {
    RemoteStart(RemoteStartRequest),
    RemoteStatus,
    RemoteStop(RemoteStopRequest),
    DevicesList(DeviceListRequest),
    DevicesRevoke(DeviceRevokeRequest),
    Doctor(DoctorRequest),
    ProductStatus,
    ProductSetLevel(SetLevelRequest),
    ProductSetActive(SetActiveRequest),
}

impl Intent {
    pub const fn command_name(&self) -> CommandName {
        match self {
            Self::RemoteStart(_) => CommandName::RemoteStart,
            Self::RemoteStatus => CommandName::RemoteStatus,
            Self::RemoteStop(_) => CommandName::RemoteStop,
            Self::DevicesList(_) => CommandName::DevicesList,
            Self::DevicesRevoke(_) => CommandName::DevicesRevoke,
            Self::Doctor(_) => CommandName::Doctor,
            Self::ProductStatus => CommandName::ProductStatus,
            Self::ProductSetLevel(_) => CommandName::ProductSetLevel,
            Self::ProductSetActive(_) => CommandName::ProductSetActive,
        }
    }

    /// Stable idempotency identity carried by consequential product actions.
    pub fn command_id(&self) -> Option<&str> {
        match self {
            Self::ProductSetLevel(request) => Some(&request.command_id),
            Self::ProductSetActive(request) => Some(&request.command_id),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CliInvocation {
    pub output: OutputFormat,
    pub intent: Intent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParseErrorCode {
    InvalidArguments,
    UnknownCommand,
    UnknownOption,
    MissingOption,
    DuplicateOption,
    InvalidValue,
    NonUnicodeArgument,
    TooManyArguments,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CliParseError {
    pub code: ParseErrorCode,
    pub command: Option<CommandName>,
}

impl CliParseError {
    const fn new(code: ParseErrorCode, command: Option<CommandName>) -> Self {
        Self { code, command }
    }
}

impl fmt::Display for CliParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("command-line arguments were rejected")
    }
}

impl std::error::Error for CliParseError {}

/// Parse arguments after the executable name into the closed CLI vocabulary.
///
/// No variant contains an arbitrary method name or an opaque payload. Unknown,
/// duplicate, malformed, non-Unicode, and excessively long arguments fail
/// closed before an adapter is called.
pub fn parse_args<I, S>(arguments: I) -> Result<CliInvocation, CliParseError>
where
    I: IntoIterator<Item = S>,
    S: Into<OsString>,
{
    let mut tokens = Vec::new();
    for argument in arguments {
        if tokens.len() >= MAX_ARGUMENTS {
            return Err(CliParseError::new(ParseErrorCode::TooManyArguments, None));
        }
        let token = argument
            .into()
            .into_string()
            .map_err(|_| CliParseError::new(ParseErrorCode::NonUnicodeArgument, None))?;
        if token.is_empty()
            || token.len() > MAX_ARGUMENT_BYTES
            || token.chars().any(char::is_control)
        {
            return Err(CliParseError::new(ParseErrorCode::InvalidValue, None));
        }
        tokens.push(token);
    }

    let mut index = 0;
    let output = if tokens.first().map(String::as_str) == Some("--output") {
        let value = tokens
            .get(1)
            .ok_or_else(|| CliParseError::new(ParseErrorCode::MissingOption, None))?;
        index = 2;
        match value.as_str() {
            "human" => OutputFormat::Human,
            "json" => OutputFormat::Json,
            _ => return Err(CliParseError::new(ParseErrorCode::InvalidValue, None)),
        }
    } else {
        OutputFormat::Human
    };

    let group = tokens
        .get(index)
        .ok_or_else(|| CliParseError::new(ParseErrorCode::InvalidArguments, None))?;
    let intent = match group.as_str() {
        "remote" => parse_remote(&tokens[index + 1..])?,
        "devices" => parse_devices(&tokens[index + 1..])?,
        "doctor" => parse_doctor(&tokens[index + 1..])?,
        "product" => parse_product(&tokens[index + 1..])?,
        _ => return Err(CliParseError::new(ParseErrorCode::UnknownCommand, None)),
    };

    Ok(CliInvocation { output, intent })
}

fn parse_remote(tokens: &[String]) -> Result<Intent, CliParseError> {
    let action = tokens
        .first()
        .ok_or_else(|| CliParseError::new(ParseErrorCode::InvalidArguments, None))?;
    match action.as_str() {
        "start" => {
            let command = CommandName::RemoteStart;
            let options = parse_options(
                &tokens[1..],
                &["--exposure", "--allow-new-devices"],
                command,
            )?;
            Ok(Intent::RemoteStart(RemoteStartRequest {
                exposure: parse_exposure(required(&options, "--exposure", command)?, command)?,
                allow_new_devices: parse_bool(
                    required(&options, "--allow-new-devices", command)?,
                    command,
                )?,
            }))
        }
        "status" => {
            ensure_empty(&tokens[1..], CommandName::RemoteStatus)?;
            Ok(Intent::RemoteStatus)
        }
        "stop" => {
            let command = CommandName::RemoteStop;
            let options = parse_options(&tokens[1..], &["--force"], command)?;
            Ok(Intent::RemoteStop(RemoteStopRequest {
                force: parse_bool(required(&options, "--force", command)?, command)?,
            }))
        }
        _ => Err(CliParseError::new(ParseErrorCode::UnknownCommand, None)),
    }
}

fn parse_devices(tokens: &[String]) -> Result<Intent, CliParseError> {
    let action = tokens
        .first()
        .ok_or_else(|| CliParseError::new(ParseErrorCode::InvalidArguments, None))?;
    match action.as_str() {
        "list" => {
            let command = CommandName::DevicesList;
            let options = parse_options(&tokens[1..], &["--filter"], command)?;
            Ok(Intent::DevicesList(DeviceListRequest {
                filter: parse_device_filter(required(&options, "--filter", command)?, command)?,
            }))
        }
        "revoke" => {
            let command = CommandName::DevicesRevoke;
            let options = parse_options(&tokens[1..], &["--device-id", "--reason"], command)?;
            let device_id = required(&options, "--device-id", command)?;
            validate_device_id(device_id, command)?;
            Ok(Intent::DevicesRevoke(DeviceRevokeRequest {
                device_id: device_id.to_owned(),
                reason: parse_revoke_reason(required(&options, "--reason", command)?, command)?,
            }))
        }
        _ => Err(CliParseError::new(ParseErrorCode::UnknownCommand, None)),
    }
}

fn parse_doctor(tokens: &[String]) -> Result<Intent, CliParseError> {
    let command = CommandName::Doctor;
    let options = parse_options(tokens, &["--area"], command)?;
    Ok(Intent::Doctor(DoctorRequest {
        area: parse_doctor_area(required(&options, "--area", command)?, command)?,
    }))
}

fn parse_product(tokens: &[String]) -> Result<Intent, CliParseError> {
    let action = tokens
        .first()
        .ok_or_else(|| CliParseError::new(ParseErrorCode::InvalidArguments, None))?;
    match action.as_str() {
        "status" => {
            ensure_empty(&tokens[1..], CommandName::ProductStatus)?;
            Ok(Intent::ProductStatus)
        }
        "set-level" => {
            let command = CommandName::ProductSetLevel;
            let options = parse_options(
                &tokens[1..],
                &[
                    "--value",
                    "--expected-revision",
                    "--command-id",
                    "--timeout-ms",
                ],
                command,
            )?;
            let value = parse_u8(required(&options, "--value", command)?, command)?;
            if value > 100 {
                return Err(CliParseError::new(
                    ParseErrorCode::InvalidValue,
                    Some(command),
                ));
            }
            let command_id = required(&options, "--command-id", command)?;
            validate_command_id(command_id, command)?;
            Ok(Intent::ProductSetLevel(SetLevelRequest {
                value,
                expected_revision: parse_u64(
                    required(&options, "--expected-revision", command)?,
                    command,
                )?,
                command_id: command_id.to_owned(),
                timeout_ms: parse_timeout_ms(
                    required(&options, "--timeout-ms", command)?,
                    command,
                )?,
            }))
        }
        "set-active" => {
            let command = CommandName::ProductSetActive;
            let options = parse_options(
                &tokens[1..],
                &[
                    "--active",
                    "--expected-revision",
                    "--command-id",
                    "--timeout-ms",
                ],
                command,
            )?;
            let command_id = required(&options, "--command-id", command)?;
            validate_command_id(command_id, command)?;
            Ok(Intent::ProductSetActive(SetActiveRequest {
                active: parse_bool(required(&options, "--active", command)?, command)?,
                expected_revision: parse_u64(
                    required(&options, "--expected-revision", command)?,
                    command,
                )?,
                command_id: command_id.to_owned(),
                timeout_ms: parse_timeout_ms(
                    required(&options, "--timeout-ms", command)?,
                    command,
                )?,
            }))
        }
        _ => Err(CliParseError::new(ParseErrorCode::UnknownCommand, None)),
    }
}

fn parse_options<'a>(
    tokens: &'a [String],
    allowed: &[&str],
    command: CommandName,
) -> Result<BTreeMap<&'a str, &'a str>, CliParseError> {
    if tokens.len() % 2 != 0 {
        return Err(CliParseError::new(
            ParseErrorCode::MissingOption,
            Some(command),
        ));
    }

    let mut options = BTreeMap::new();
    for pair in tokens.chunks_exact(2) {
        let option = pair[0].as_str();
        let value = pair[1].as_str();
        if !option.starts_with("--") || !allowed.contains(&option) {
            return Err(CliParseError::new(
                ParseErrorCode::UnknownOption,
                Some(command),
            ));
        }
        if value.starts_with("--") {
            return Err(CliParseError::new(
                ParseErrorCode::MissingOption,
                Some(command),
            ));
        }
        if options.insert(option, value).is_some() {
            return Err(CliParseError::new(
                ParseErrorCode::DuplicateOption,
                Some(command),
            ));
        }
    }
    Ok(options)
}

fn required<'a>(
    options: &'a BTreeMap<&str, &'a str>,
    name: &str,
    command: CommandName,
) -> Result<&'a str, CliParseError> {
    options
        .get(name)
        .copied()
        .ok_or_else(|| CliParseError::new(ParseErrorCode::MissingOption, Some(command)))
}

fn ensure_empty(tokens: &[String], command: CommandName) -> Result<(), CliParseError> {
    if tokens.is_empty() {
        Ok(())
    } else {
        Err(CliParseError::new(
            ParseErrorCode::InvalidArguments,
            Some(command),
        ))
    }
}

fn parse_bool(value: &str, command: CommandName) -> Result<bool, CliParseError> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(CliParseError::new(
            ParseErrorCode::InvalidValue,
            Some(command),
        )),
    }
}

fn parse_exposure(value: &str, command: CommandName) -> Result<RemoteExposure, CliParseError> {
    match value {
        "local-network" => Ok(RemoteExposure::LocalNetwork),
        "configured-relay" => Ok(RemoteExposure::ConfiguredRelay),
        _ => Err(CliParseError::new(
            ParseErrorCode::InvalidValue,
            Some(command),
        )),
    }
}

fn parse_device_filter(value: &str, command: CommandName) -> Result<DeviceFilter, CliParseError> {
    match value {
        "current" => Ok(DeviceFilter::Current),
        "revoked" => Ok(DeviceFilter::Revoked),
        "all" => Ok(DeviceFilter::All),
        _ => Err(CliParseError::new(
            ParseErrorCode::InvalidValue,
            Some(command),
        )),
    }
}

fn parse_revoke_reason(value: &str, command: CommandName) -> Result<RevokeReason, CliParseError> {
    match value {
        "user-requested" => Ok(RevokeReason::UserRequested),
        "lost" => Ok(RevokeReason::Lost),
        "compromised" => Ok(RevokeReason::Compromised),
        _ => Err(CliParseError::new(
            ParseErrorCode::InvalidValue,
            Some(command),
        )),
    }
}

fn parse_doctor_area(value: &str, command: CommandName) -> Result<DoctorArea, CliParseError> {
    match value {
        "all" => Ok(DoctorArea::All),
        "authority" => Ok(DoctorArea::Authority),
        "transport" => Ok(DoctorArea::Transport),
        "browser" => Ok(DoctorArea::Browser),
        _ => Err(CliParseError::new(
            ParseErrorCode::InvalidValue,
            Some(command),
        )),
    }
}

fn parse_u8(value: &str, command: CommandName) -> Result<u8, CliParseError> {
    value
        .parse()
        .map_err(|_| CliParseError::new(ParseErrorCode::InvalidValue, Some(command)))
}

fn parse_u64(value: &str, command: CommandName) -> Result<u64, CliParseError> {
    value
        .parse()
        .map_err(|_| CliParseError::new(ParseErrorCode::InvalidValue, Some(command)))
}

fn parse_timeout_ms(value: &str, command: CommandName) -> Result<u64, CliParseError> {
    let timeout_ms = parse_u64(value, command)?;
    if (MIN_TIMEOUT_MS..=MAX_TIMEOUT_MS).contains(&timeout_ms) {
        Ok(timeout_ms)
    } else {
        Err(CliParseError::new(
            ParseErrorCode::InvalidValue,
            Some(command),
        ))
    }
}

fn validate_device_id(value: &str, command: CommandName) -> Result<(), CliParseError> {
    let mut bytes = value.bytes();
    let valid = value.len() <= MAX_DEVICE_ID_BYTES
        && bytes
            .next()
            .is_some_and(|byte| byte.is_ascii_alphanumeric())
        && bytes
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b':' | b'-'));
    if valid {
        Ok(())
    } else {
        Err(CliParseError::new(
            ParseErrorCode::InvalidValue,
            Some(command),
        ))
    }
}

fn validate_command_id(value: &str, command: CommandName) -> Result<(), CliParseError> {
    let mut bytes = value.bytes();
    let valid = (MIN_COMMAND_ID_BYTES..=MAX_COMMAND_ID_BYTES).contains(&value.len())
        && bytes
            .next()
            .is_some_and(|byte| byte.is_ascii_alphanumeric())
        && bytes
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b':' | b'-'));
    if valid {
        Ok(())
    } else {
        Err(CliParseError::new(
            ParseErrorCode::InvalidValue,
            Some(command),
        ))
    }
}

pub(crate) fn detect_output_format(arguments: &[OsString]) -> OutputFormat {
    if arguments.first().and_then(|value| value.to_str()) == Some("--output")
        && arguments.get(1).and_then(|value| value.to_str()) == Some("json")
    {
        OutputFormat::Json
    } else {
        OutputFormat::Human
    }
}
