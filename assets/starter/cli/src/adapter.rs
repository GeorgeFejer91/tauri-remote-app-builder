use serde::Serialize;

use crate::intent::{
    DeviceFilter, DeviceListRequest, DeviceRevokeRequest, DoctorRequest, RemoteExposure,
    RemoteStartRequest, RemoteStopRequest, SetActiveRequest, SetLevelRequest,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Revisioned<T> {
    pub command_id: Option<String>,
    pub authority_generation: String,
    pub revision: u64,
    pub value: T,
}

impl<T> Revisioned<T> {
    pub fn new(authority_generation: impl Into<String>, revision: u64, value: T) -> Self {
        Self {
            command_id: None,
            authority_generation: authority_generation.into(),
            revision,
            value,
        }
    }

    /// Build the acknowledged outcome of a consequential idempotent command.
    pub fn for_command(
        command_id: impl Into<String>,
        authority_generation: impl Into<String>,
        revision: u64,
        value: T,
    ) -> Self {
        Self {
            command_id: Some(command_id.into()),
            authority_generation: authority_generation.into(),
            revision,
            value,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RemoteRunState {
    Stopped,
    Discovering,
    Authenticating,
    ControlReady,
    MediaReady,
    Degraded,
    Reconnecting,
    Revoked,
    Stopping,
}

impl RemoteRunState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stopped => "stopped",
            Self::Discovering => "discovering",
            Self::Authenticating => "authenticating",
            Self::ControlReady => "control_ready",
            Self::MediaReady => "media_ready",
            Self::Degraded => "degraded",
            Self::Reconnecting => "reconnecting",
            Self::Revoked => "revoked",
            Self::Stopping => "stopping",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteStatus {
    pub state: RemoteRunState,
    pub exposure: Option<RemoteExposure>,
    pub allow_new_devices: bool,
    pub connected_devices: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceTrust {
    Trusted,
    Revoked,
}

impl DeviceTrust {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Trusted => "trusted",
            Self::Revoked => "revoked",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSummary {
    pub device_id: String,
    pub trust: DeviceTrust,
    pub connected: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceList {
    pub filter: DeviceFilter,
    pub devices: Vec<DeviceSummary>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceRevocation {
    pub device_id: String,
    pub revoked: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DoctorComponent {
    Authority,
    Transport,
    Browser,
}

impl DoctorComponent {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Authority => "authority",
            Self::Transport => "transport",
            Self::Browser => "browser",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

impl CheckStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Warn => "warn",
            Self::Fail => "fail",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctorFinding {
    pub component: DoctorComponent,
    pub status: CheckStatus,
    pub required: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctorReport {
    pub healthy: bool,
    pub findings: Vec<DoctorFinding>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductState {
    pub active: bool,
    pub level: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdapterErrorCode {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdapterError {
    code: AdapterErrorCode,
}

impl AdapterError {
    pub const fn new(code: AdapterErrorCode) -> Self {
        Self { code }
    }

    pub const fn code(self) -> AdapterErrorCode {
        self.code
    }

    pub const fn retryable(self) -> bool {
        matches!(
            self.code,
            AdapterErrorCode::Conflict
                | AdapterErrorCode::Busy
                | AdapterErrorCode::Unavailable
                | AdapterErrorCode::TimedOutUnknownOutcome
        )
    }
}

pub type AdapterResult<T> = Result<Revisioned<T>, AdapterError>;

/// A narrow adapter to the application's running Rust authority.
///
/// Implement these methods by calling the same in-process service, local IPC
/// boundary, or authenticated daemon used by the Tauri application. An
/// implementation must not create a second state store or a second revision
/// counter. There is deliberately no generic `invoke`, `exec`, or raw-payload
/// method on this trait. Generated product methods must map one-to-one to the
/// same typed semantic actions accepted from a paired browser. Local-only
/// exclusions are limited to commands classified as `LocalAdminOnly` by
/// `CommandName::availability`. Consequential product requests carry a stable
/// command id and bounded timeout. On acknowledgement loss, return
/// `TimedOutUnknownOutcome`; callers may retry only with the same command id
/// and logically identical request bytes.
pub trait AdminAdapter {
    fn remote_start(&mut self, request: RemoteStartRequest) -> AdapterResult<RemoteStatus>;
    fn remote_status(&mut self) -> AdapterResult<RemoteStatus>;
    fn remote_stop(&mut self, request: RemoteStopRequest) -> AdapterResult<RemoteStatus>;
    fn devices_list(&mut self, request: DeviceListRequest) -> AdapterResult<DeviceList>;
    fn devices_revoke(&mut self, request: DeviceRevokeRequest) -> AdapterResult<DeviceRevocation>;
    fn doctor(&mut self, request: DoctorRequest) -> AdapterResult<DoctorReport>;
    fn product_status(&mut self) -> AdapterResult<ProductState>;
    fn product_set_level(&mut self, request: SetLevelRequest) -> AdapterResult<ProductState>;
    fn product_set_active(&mut self, request: SetActiveRequest) -> AdapterResult<ProductState>;
}

/// Safe placeholder used by the example binary until the application wires
/// `AdminAdapter` to its running authority.
#[derive(Debug, Default)]
pub struct UnwiredAdapter;

fn not_connected<T>() -> AdapterResult<T> {
    Err(AdapterError::new(AdapterErrorCode::NotConnected))
}

impl AdminAdapter for UnwiredAdapter {
    fn remote_start(&mut self, _request: RemoteStartRequest) -> AdapterResult<RemoteStatus> {
        not_connected()
    }

    fn remote_status(&mut self) -> AdapterResult<RemoteStatus> {
        not_connected()
    }

    fn remote_stop(&mut self, _request: RemoteStopRequest) -> AdapterResult<RemoteStatus> {
        not_connected()
    }

    fn devices_list(&mut self, _request: DeviceListRequest) -> AdapterResult<DeviceList> {
        not_connected()
    }

    fn devices_revoke(&mut self, _request: DeviceRevokeRequest) -> AdapterResult<DeviceRevocation> {
        not_connected()
    }

    fn doctor(&mut self, _request: DoctorRequest) -> AdapterResult<DoctorReport> {
        not_connected()
    }

    fn product_status(&mut self) -> AdapterResult<ProductState> {
        not_connected()
    }

    fn product_set_level(&mut self, _request: SetLevelRequest) -> AdapterResult<ProductState> {
        not_connected()
    }

    fn product_set_active(&mut self, _request: SetActiveRequest) -> AdapterResult<ProductState> {
        not_connected()
    }
}
