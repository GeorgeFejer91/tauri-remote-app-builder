use std::collections::BTreeSet;
use std::process::Command;

use remote_control_core_starter::{AppAction, AppState, Authority, CommandRequest, Grant};
use serde_json::{json, Value};
use tauri_remote_app_cli::{
    run_cli, AdapterError, AdapterErrorCode, AdapterResult, AdminAdapter, CheckStatus,
    CommandAvailability, CommandName, DeviceFilter, DeviceList, DeviceListRequest,
    DeviceRevocation, DeviceRevokeRequest, DeviceSummary, DeviceTrust, DoctorArea, DoctorComponent,
    DoctorFinding, DoctorReport, DoctorRequest, Intent, ProductState, RemoteExposure,
    RemoteRunState, RemoteStartRequest, RemoteStatus, RemoteStopRequest, Revisioned, RevokeReason,
    SetActiveRequest, SetLevelRequest,
};

const GENERATION: &str = "authority_test_01";

#[derive(Debug)]
struct SameServiceMock {
    calls: Vec<Intent>,
    revision: u64,
    remote_status: RemoteStatus,
    product_state: ProductState,
    next_error: Option<AdapterErrorCode>,
    private_diagnostic: String,
}

impl Default for SameServiceMock {
    fn default() -> Self {
        Self {
            calls: Vec::new(),
            revision: 40,
            remote_status: RemoteStatus {
                state: RemoteRunState::Stopped,
                exposure: None,
                allow_new_devices: false,
                connected_devices: 0,
            },
            product_state: ProductState {
                active: false,
                level: 12,
            },
            next_error: None,
            private_diagnostic: String::new(),
        }
    }
}

impl SameServiceMock {
    fn fail_if_requested(&mut self) -> Result<(), AdapterError> {
        if let Some(code) = self.next_error.take() {
            Err(AdapterError::new(code))
        } else {
            Ok(())
        }
    }

    fn reply<T>(&mut self, value: T) -> AdapterResult<T> {
        self.fail_if_requested()?;
        Ok(Revisioned::new(GENERATION, self.revision, value))
    }

    fn commit(&mut self) {
        self.revision += 1;
    }

    fn check_revision(&mut self, expected_revision: u64) -> Result<(), AdapterError> {
        if expected_revision == self.revision {
            Ok(())
        } else {
            Err(AdapterError::new(AdapterErrorCode::Conflict))
        }
    }
}

impl AdminAdapter for SameServiceMock {
    fn remote_start(&mut self, request: RemoteStartRequest) -> AdapterResult<RemoteStatus> {
        self.calls.push(Intent::RemoteStart(request.clone()));
        self.remote_status = RemoteStatus {
            state: RemoteRunState::ControlReady,
            exposure: Some(request.exposure),
            allow_new_devices: request.allow_new_devices,
            connected_devices: 0,
        };
        self.commit();
        self.reply(self.remote_status.clone())
    }

    fn remote_status(&mut self) -> AdapterResult<RemoteStatus> {
        self.calls.push(Intent::RemoteStatus);
        self.reply(self.remote_status.clone())
    }

    fn remote_stop(&mut self, request: RemoteStopRequest) -> AdapterResult<RemoteStatus> {
        self.calls.push(Intent::RemoteStop(request));
        self.remote_status = RemoteStatus {
            state: RemoteRunState::Stopped,
            exposure: None,
            allow_new_devices: false,
            connected_devices: 0,
        };
        self.commit();
        self.reply(self.remote_status.clone())
    }

    fn devices_list(&mut self, request: DeviceListRequest) -> AdapterResult<DeviceList> {
        self.calls.push(Intent::DevicesList(request.clone()));
        self.reply(DeviceList {
            filter: request.filter,
            devices: vec![DeviceSummary {
                device_id: "device_test_01".to_owned(),
                trust: DeviceTrust::Trusted,
                connected: false,
            }],
        })
    }

    fn devices_revoke(&mut self, request: DeviceRevokeRequest) -> AdapterResult<DeviceRevocation> {
        self.calls.push(Intent::DevicesRevoke(request.clone()));
        self.commit();
        self.reply(DeviceRevocation {
            device_id: request.device_id,
            revoked: true,
        })
    }

    fn doctor(&mut self, request: DoctorRequest) -> AdapterResult<DoctorReport> {
        self.calls.push(Intent::Doctor(request));
        self.reply(DoctorReport {
            healthy: true,
            findings: vec![DoctorFinding {
                component: DoctorComponent::Authority,
                status: CheckStatus::Pass,
                required: true,
            }],
        })
    }

    fn product_status(&mut self) -> AdapterResult<ProductState> {
        self.calls.push(Intent::ProductStatus);
        self.reply(self.product_state.clone())
    }

    fn product_set_level(&mut self, request: SetLevelRequest) -> AdapterResult<ProductState> {
        self.calls.push(Intent::ProductSetLevel(request.clone()));
        self.fail_if_requested()?;
        self.check_revision(request.expected_revision)?;
        self.product_state.level = request.value;
        self.commit();
        Ok(Revisioned::for_command(
            request.command_id,
            GENERATION,
            self.revision,
            self.product_state.clone(),
        ))
    }

    fn product_set_active(&mut self, request: SetActiveRequest) -> AdapterResult<ProductState> {
        self.calls.push(Intent::ProductSetActive(request.clone()));
        self.fail_if_requested()?;
        self.check_revision(request.expected_revision)?;
        self.product_state.active = request.active;
        self.commit();
        Ok(Revisioned::for_command(
            request.command_id,
            GENERATION,
            self.revision,
            self.product_state.clone(),
        ))
    }
}

fn run_json(arguments: &[&str], adapter: &mut SameServiceMock) -> Value {
    let mut command = vec!["--output", "json"];
    command.extend_from_slice(arguments);
    let run = run_cli(command, adapter);
    assert_eq!(run.exit_code, 0, "{}", run.stderr);
    assert!(run.stderr.is_empty());
    serde_json::from_str(&run.stdout).expect("valid JSON success")
}

#[test]
fn every_fixed_verb_reaches_one_typed_same_service_method() {
    let mut adapter = SameServiceMock::default();

    let start = run_json(
        &[
            "remote",
            "start",
            "--allow-new-devices",
            "false",
            "--exposure",
            "local-network",
        ],
        &mut adapter,
    );
    assert_eq!(start["command"], "remote.start");
    assert_eq!(start["authorityGeneration"], GENERATION);
    assert_eq!(start["revision"], 41);
    assert_eq!(start["data"]["value"]["allowNewDevices"], false);

    assert_eq!(
        run_json(&["remote", "status"], &mut adapter)["revision"],
        41
    );
    assert_eq!(
        run_json(&["remote", "stop", "--force", "false"], &mut adapter)["revision"],
        42
    );
    assert_eq!(
        run_json(&["devices", "list", "--filter", "current"], &mut adapter,)["revision"],
        42
    );
    assert_eq!(
        run_json(
            &[
                "devices",
                "revoke",
                "--device-id",
                "device_test_01",
                "--reason",
                "lost",
            ],
            &mut adapter,
        )["revision"],
        43
    );
    assert_eq!(
        run_json(&["doctor", "--area", "all"], &mut adapter)["revision"],
        43
    );
    assert_eq!(
        run_json(&["product", "status"], &mut adapter)["revision"],
        43
    );
    let level = run_json(
        &[
            "product",
            "set-level",
            "--value",
            "37",
            "--expected-revision",
            "43",
            "--command-id",
            "cli_level_0001",
            "--timeout-ms",
            "5000",
        ],
        &mut adapter,
    );
    assert_eq!(level["revision"], 44);
    assert_eq!(level["commandId"], "cli_level_0001");
    let active = run_json(
        &[
            "product",
            "set-active",
            "--active",
            "true",
            "--expected-revision",
            "44",
            "--command-id",
            "cli_active_0001",
            "--timeout-ms",
            "5000",
        ],
        &mut adapter,
    );
    assert_eq!(active["revision"], 45);
    assert_eq!(active["commandId"], "cli_active_0001");
    assert_eq!(
        active["data"]["value"],
        json!({"active": true, "level": 37})
    );

    assert_eq!(
        adapter.calls,
        vec![
            Intent::RemoteStart(RemoteStartRequest {
                exposure: RemoteExposure::LocalNetwork,
                allow_new_devices: false,
            }),
            Intent::RemoteStatus,
            Intent::RemoteStop(RemoteStopRequest { force: false }),
            Intent::DevicesList(DeviceListRequest {
                filter: DeviceFilter::Current,
            }),
            Intent::DevicesRevoke(DeviceRevokeRequest {
                device_id: "device_test_01".to_owned(),
                reason: RevokeReason::Lost,
            }),
            Intent::Doctor(DoctorRequest {
                area: DoctorArea::All,
            }),
            Intent::ProductStatus,
            Intent::ProductSetLevel(SetLevelRequest {
                value: 37,
                expected_revision: 43,
                command_id: "cli_level_0001".to_owned(),
                timeout_ms: 5_000,
            }),
            Intent::ProductSetActive(SetActiveRequest {
                active: true,
                expected_revision: 44,
                command_id: "cli_active_0001".to_owned(),
                timeout_ms: 5_000,
            }),
        ]
    );
}

#[test]
fn human_output_has_a_stable_revisioned_shape() {
    let mut adapter = SameServiceMock::default();
    let run = run_cli(["product", "status"], &mut adapter);

    assert_eq!(run.exit_code, 0);
    assert!(run.stderr.is_empty());
    assert_eq!(
        run.stdout,
        concat!(
            "schema_version=appctl.v1\n",
            "ok=true\n",
            "command=product.status\n",
            "authority_generation=\"authority_test_01\"\n",
            "revision=40\n",
            "data.type=product_state\n",
            "data.active=false\n",
            "data.level=12\n",
        )
    );
}

#[test]
fn malformed_and_generic_forwarding_inputs_fail_before_dispatch() {
    let rejected = [
        vec!["invoke", "secret.method"],
        vec!["remote", "invoke", "--payload", "{}"],
        vec!["remote", "status", "--", "secret.method"],
        vec![
            "remote",
            "start",
            "--exposure",
            "local-network",
            "--allow-new-devices",
            "yes",
        ],
        vec![
            "remote",
            "start",
            "--exposure",
            "local-network",
            "--exposure",
            "configured-relay",
            "--allow-new-devices",
            "true",
        ],
        vec![
            "devices",
            "revoke",
            "--device-id",
            "../private-key",
            "--reason",
            "lost",
        ],
        vec![
            "product",
            "set-level",
            "--value",
            "101",
            "--expected-revision",
            "40",
            "--command-id",
            "cli_invalid_level_0001",
            "--timeout-ms",
            "5000",
        ],
        vec!["product", "set-active", "--active", "true"],
        vec![
            "product",
            "set-active",
            "--active",
            "true",
            "--expected-revision",
            "40",
            "--command-id",
            "cli_payload_0001",
            "--timeout-ms",
            "5000",
            "--payload",
            "{}",
        ],
        vec![
            "product",
            "set-active",
            "--active",
            "true",
            "--expected-revision",
            "40",
            "--command-id",
            "../not-stable",
            "--timeout-ms",
            "5000",
        ],
        vec![
            "product",
            "set-active",
            "--active",
            "true",
            "--expected-revision",
            "40",
            "--command-id",
            "cli_timeout_too_short_0001",
            "--timeout-ms",
            "99",
        ],
        vec![
            "product",
            "set-active",
            "--active",
            "true",
            "--expected-revision",
            "40",
            "--command-id",
            "cli_timeout_too_long_0001",
            "--timeout-ms",
            "120001",
        ],
    ];

    for arguments in rejected {
        let mut adapter = SameServiceMock::default();
        let mut command = vec!["--output", "json"];
        command.extend(arguments);
        let run = run_cli(command, &mut adapter);
        assert_eq!(run.exit_code, 2, "unexpected output: {}", run.stdout);
        assert!(run.stdout.is_empty());
        let failure: Value = serde_json::from_str(&run.stderr).expect("valid JSON failure");
        assert_eq!(failure["schemaVersion"], "appctl.v1");
        assert_eq!(failure["ok"], false);
        assert!(failure["error"]["retryable"].is_boolean());
        assert!(adapter.calls.is_empty());
    }
}

#[test]
fn adapter_errors_are_sanitized_and_revision_conflicts_are_explicit() {
    let mut adapter = SameServiceMock {
        next_error: Some(AdapterErrorCode::Internal),
        private_diagnostic: "token=secret-value path=C:\\private\\authority.db".to_owned(),
        ..SameServiceMock::default()
    };
    let secret = adapter.private_diagnostic.clone();
    let run = run_cli(["--output", "json", "remote", "status"], &mut adapter);
    assert_eq!(run.exit_code, 8);
    assert!(run.stdout.is_empty());
    assert!(!run.stderr.contains(&secret));
    assert!(!run.stderr.contains("secret-value"));
    assert_eq!(
        serde_json::from_str::<Value>(&run.stderr).expect("error JSON"),
        json!({
            "schemaVersion": "appctl.v1",
            "ok": false,
            "command": "remote.status",
            "error": {
                "code": "internal",
                "message": "the application service could not complete the operation",
                "retryable": false
            }
        })
    );

    let mut adapter = SameServiceMock::default();
    let conflict = run_cli(
        [
            "--output",
            "json",
            "product",
            "set-level",
            "--value",
            "20",
            "--expected-revision",
            "39",
            "--command-id",
            "cli_conflict_0001",
            "--timeout-ms",
            "5000",
        ],
        &mut adapter,
    );
    assert_eq!(conflict.exit_code, 5);
    let failure: Value = serde_json::from_str(&conflict.stderr).expect("conflict JSON");
    assert_eq!(failure["commandId"], "cli_conflict_0001");
    assert_eq!(failure["error"]["code"], "conflict");
    assert_eq!(failure["error"]["retryable"], true);
    assert_eq!(adapter.product_state.level, 12);
}

#[test]
fn adapter_errors_map_to_stable_documented_exit_codes() {
    let cases = [
        (AdapterErrorCode::Unauthorized, 3),
        (AdapterErrorCode::Forbidden, 4),
        (AdapterErrorCode::Conflict, 5),
        (AdapterErrorCode::InvalidState, 5),
        (AdapterErrorCode::NotConnected, 6),
        (AdapterErrorCode::Busy, 6),
        (AdapterErrorCode::Unavailable, 6),
        (AdapterErrorCode::TimedOutUnknownOutcome, 7),
        (AdapterErrorCode::Internal, 8),
    ];

    for (code, exit_code) in cases {
        let mut adapter = SameServiceMock {
            next_error: Some(code),
            ..SameServiceMock::default()
        };
        let run = run_cli(["--output", "json", "remote", "status"], &mut adapter);
        assert_eq!(run.exit_code, exit_code, "unexpected mapping for {code:?}");
    }
}

#[test]
fn timeout_unknown_outcome_preserves_command_id_and_does_not_claim_failure() {
    let mut adapter = SameServiceMock {
        next_error: Some(AdapterErrorCode::TimedOutUnknownOutcome),
        ..SameServiceMock::default()
    };
    let run = run_cli(
        [
            "--output",
            "json",
            "product",
            "set-active",
            "--active",
            "true",
            "--expected-revision",
            "40",
            "--command-id",
            "cli_timeout_unknown_0001",
            "--timeout-ms",
            "750",
        ],
        &mut adapter,
    );

    assert_eq!(run.exit_code, 7);
    assert!(run.stdout.is_empty());
    let failure: Value = serde_json::from_str(&run.stderr).expect("timeout JSON");
    assert_eq!(failure["command"], "product.set_active");
    assert_eq!(failure["commandId"], "cli_timeout_unknown_0001");
    assert_eq!(failure["error"]["code"], "timed_out_unknown_outcome");
    assert_eq!(failure["error"]["retryable"], true);
    assert!(!adapter.product_state.active);
    assert_eq!(
        adapter.calls,
        vec![Intent::ProductSetActive(SetActiveRequest {
            active: true,
            expected_revision: 40,
            command_id: "cli_timeout_unknown_0001".to_owned(),
            timeout_ms: 750,
        })]
    );
}

#[test]
fn remote_status_vocabulary_distinguishes_readiness_and_recovery() {
    let cases = [
        (RemoteRunState::Stopped, "stopped"),
        (RemoteRunState::Discovering, "discovering"),
        (RemoteRunState::Authenticating, "authenticating"),
        (RemoteRunState::ControlReady, "control_ready"),
        (RemoteRunState::MediaReady, "media_ready"),
        (RemoteRunState::Degraded, "degraded"),
        (RemoteRunState::Reconnecting, "reconnecting"),
        (RemoteRunState::Revoked, "revoked"),
        (RemoteRunState::Stopping, "stopping"),
    ];

    for (state, expected) in cases {
        assert_eq!(state.as_str(), expected);
        assert_eq!(serde_json::to_value(state).expect("state JSON"), expected);
    }
}

#[test]
fn packaged_binary_fails_closed_until_the_real_adapter_is_wired() {
    let output = Command::new(env!("CARGO_BIN_EXE_appctl"))
        .args(["--output", "json", "remote", "status"])
        .output()
        .expect("run appctl binary");

    assert_eq!(output.status.code(), Some(6));
    assert!(output.stdout.is_empty());
    let failure: Value = serde_json::from_slice(&output.stderr).expect("binary JSON failure");
    assert_eq!(failure["command"], "remote.status");
    assert_eq!(failure["error"]["code"], "not_connected");
}

struct AuthorityProductAdapter {
    authority: Authority,
}

impl AuthorityProductAdapter {
    fn new() -> Self {
        let mut authority = Authority::new(
            GENERATION.to_owned(),
            AppState {
                active: false,
                level: 12,
            },
        )
        .expect("valid test authority");
        authority
            .issue_grant(Grant {
                grant_id: "grant_cli_test_01".to_owned(),
                principal_id: "principal_cli_test_01".to_owned(),
                role: "controller".to_owned(),
                scopes: BTreeSet::from(["example.control".to_owned()]),
                authority_generation: GENERATION.to_owned(),
                expires_at_ms: 10_000,
            })
            .expect("valid test grant");
        Self { authority }
    }

    fn apply(
        &mut self,
        command_id: String,
        _timeout_ms: u64,
        expected_revision: u64,
        action: AppAction,
    ) -> AdapterResult<ProductState> {
        let command = CommandRequest {
            authority_generation: GENERATION.to_owned(),
            principal_id: "principal_cli_test_01".to_owned(),
            grant_id: "grant_cli_test_01".to_owned(),
            command_id: command_id.clone(),
            scope: "example.control".to_owned(),
            expected_revision: Some(expected_revision),
            action,
        };
        let applied = self.authority.apply(command, 100).map_err(|rejected| {
            let code = match rejected.error {
                "stale_revision" => AdapterErrorCode::Conflict,
                "unauthenticated" | "grant_expired" => AdapterErrorCode::Unauthorized,
                "scope_denied" => AdapterErrorCode::Forbidden,
                "busy" => AdapterErrorCode::Busy,
                "invalid_action" => AdapterErrorCode::InvalidState,
                _ => AdapterErrorCode::Internal,
            };
            AdapterError::new(code)
        })?;
        Ok(Revisioned::for_command(
            command_id,
            GENERATION,
            applied.revision,
            ProductState {
                active: applied.state.active,
                level: applied.state.level,
            },
        ))
    }
}

impl AdminAdapter for AuthorityProductAdapter {
    fn remote_start(&mut self, _request: RemoteStartRequest) -> AdapterResult<RemoteStatus> {
        Err(AdapterError::new(AdapterErrorCode::NotConnected))
    }

    fn remote_status(&mut self) -> AdapterResult<RemoteStatus> {
        Err(AdapterError::new(AdapterErrorCode::NotConnected))
    }

    fn remote_stop(&mut self, _request: RemoteStopRequest) -> AdapterResult<RemoteStatus> {
        Err(AdapterError::new(AdapterErrorCode::NotConnected))
    }

    fn devices_list(&mut self, _request: DeviceListRequest) -> AdapterResult<DeviceList> {
        Err(AdapterError::new(AdapterErrorCode::NotConnected))
    }

    fn devices_revoke(&mut self, _request: DeviceRevokeRequest) -> AdapterResult<DeviceRevocation> {
        Err(AdapterError::new(AdapterErrorCode::NotConnected))
    }

    fn doctor(&mut self, _request: DoctorRequest) -> AdapterResult<DoctorReport> {
        Err(AdapterError::new(AdapterErrorCode::NotConnected))
    }

    fn product_status(&mut self) -> AdapterResult<ProductState> {
        let snapshot = self.authority.snapshot();
        Ok(Revisioned::new(
            snapshot.authority_generation,
            snapshot.revision,
            ProductState {
                active: snapshot.state.active,
                level: snapshot.state.level,
            },
        ))
    }

    fn product_set_level(&mut self, request: SetLevelRequest) -> AdapterResult<ProductState> {
        let SetLevelRequest {
            value,
            expected_revision,
            command_id,
            timeout_ms,
        } = request;
        self.apply(
            command_id,
            timeout_ms,
            expected_revision,
            AppAction::SetLevel { value },
        )
    }

    fn product_set_active(&mut self, request: SetActiveRequest) -> AdapterResult<ProductState> {
        let SetActiveRequest {
            active,
            expected_revision,
            command_id,
            timeout_ms,
        } = request;
        self.apply(
            command_id,
            timeout_ms,
            expected_revision,
            if active {
                AppAction::Activate
            } else {
                AppAction::Deactivate
            },
        )
    }
}

#[test]
fn product_intent_applies_through_the_starter_authority() {
    let mut adapter = AuthorityProductAdapter::new();
    let command = [
        "--output",
        "json",
        "product",
        "set-level",
        "--value",
        "37",
        "--expected-revision",
        "0",
        "--command-id",
        "cli_authority_0001",
        "--timeout-ms",
        "5000",
    ];
    let run = run_cli(command, &mut adapter);

    assert_eq!(run.exit_code, 0, "{}", run.stderr);
    let success: Value = serde_json::from_str(&run.stdout).expect("authority result JSON");
    assert_eq!(success["command"], "product.set_level");
    assert_eq!(success["commandId"], "cli_authority_0001");
    assert_eq!(success["authorityGeneration"], GENERATION);
    assert_eq!(success["revision"], 1);
    assert_eq!(
        success["data"],
        json!({
            "type": "product_state",
            "value": {"active": false, "level": 37}
        })
    );

    let snapshot = adapter.authority.snapshot();
    assert_eq!(snapshot.revision, 1);
    assert_eq!(snapshot.state.level, 37);

    let retry = run_cli(command, &mut adapter);
    assert_eq!(retry.exit_code, 0, "{}", retry.stderr);
    let retried: Value = serde_json::from_str(&retry.stdout).expect("deduplicated result JSON");
    assert_eq!(retried["commandId"], "cli_authority_0001");
    assert_eq!(retried["revision"], 1);
    assert_eq!(adapter.authority.snapshot().revision, 1);
    assert_eq!(
        CommandName::ProductSetLevel.availability(),
        CommandAvailability::PairedBrowserAndCli
    );
}
