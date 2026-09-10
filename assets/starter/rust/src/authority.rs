use std::collections::{BTreeSet, HashMap};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::protocol::{AppAction, AppSnapshot, AppState, Applied, CommandRequest, Rejected};

const MAX_GRANTS: usize = 256;
const MAX_DEDUPE_RESULTS: usize = 4096;
const MAX_SCOPES_PER_GRANT: usize = 64;
const MAX_TOKEN_BYTES: usize = 128;

#[derive(Clone, Debug)]
pub struct Grant {
    pub grant_id: String,
    pub principal_id: String,
    pub role: String,
    pub scopes: BTreeSet<String>,
    pub authority_generation: String,
    pub expires_at_ms: u64,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct DedupeKey {
    authority_generation: String,
    principal_id: String,
    command_id: String,
}

#[derive(Clone, Debug)]
struct CachedOutcome {
    fingerprint: [u8; 32],
    outcome: FinalOutcome,
}

#[derive(Clone, Debug)]
enum FinalOutcome {
    Applied(Applied),
    Rejected(Rejected),
}

impl FinalOutcome {
    fn result(&self) -> Result<Applied, Rejected> {
        match self {
            Self::Applied(applied) => Ok(applied.clone()),
            Self::Rejected(rejected) => Err(rejected.clone()),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CommandFingerprint<'a> {
    scope: &'a str,
    expected_revision: Option<u64>,
    action: &'a AppAction,
}

#[derive(Debug)]
pub struct Authority {
    generation: String,
    revision: u64,
    state: AppState,
    grants: HashMap<String, Grant>,
    outcomes: HashMap<DedupeKey, CachedOutcome>,
}

impl Authority {
    pub fn new(generation: String, state: AppState) -> Result<Self, &'static str> {
        validate_id(&generation).map_err(|_| "invalid_generation")?;
        if state.level > 100 {
            return Err("invalid_state");
        }
        Ok(Self {
            generation,
            revision: 0,
            state,
            grants: HashMap::new(),
            outcomes: HashMap::new(),
        })
    }

    /// Call only after a reviewed authentication adapter has authenticated the
    /// principal and locally authorized these exact role/scopes.
    pub fn issue_grant(&mut self, grant: Grant) -> Result<(), &'static str> {
        if self.grants.len() >= MAX_GRANTS {
            return Err("grant_limit_exceeded");
        }
        validate_id(&grant.grant_id).map_err(|_| "invalid_grant")?;
        validate_id(&grant.principal_id).map_err(|_| "invalid_principal")?;
        validate_token(&grant.role).map_err(|_| "invalid_role")?;
        if grant.authority_generation != self.generation
            || grant.scopes.is_empty()
            || grant.scopes.len() > MAX_SCOPES_PER_GRANT
            || self.grants.contains_key(&grant.grant_id)
        {
            return Err("invalid_grant");
        }
        if grant
            .scopes
            .iter()
            .any(|scope| validate_token(scope).is_err())
        {
            return Err("invalid_scope");
        }
        self.grants.insert(grant.grant_id.clone(), grant);
        Ok(())
    }

    pub fn revoke_grant(&mut self, grant_id: &str) {
        self.grants.remove(grant_id);
    }

    pub fn rotate_authority(&mut self, next_generation: String) -> Result<(), &'static str> {
        validate_id(&next_generation).map_err(|_| "invalid_generation")?;
        if next_generation == self.generation {
            return Err("generation_not_rotated");
        }
        self.generation = next_generation;
        self.revision = 0;
        self.grants.clear();
        self.outcomes.clear();
        Ok(())
    }

    pub fn snapshot(&self) -> AppSnapshot {
        AppSnapshot {
            authority_generation: self.generation.clone(),
            revision: self.revision,
            state: self.state.clone(),
        }
    }

    pub fn apply(&mut self, command: CommandRequest, now_ms: u64) -> Result<Applied, Rejected> {
        let reject = |code| Rejected::new(&command.command_id, self.revision, code);
        validate_id(&command.command_id).map_err(|_| reject("malformed_message"))?;
        validate_id(&command.principal_id).map_err(|_| reject("malformed_message"))?;
        validate_id(&command.grant_id).map_err(|_| reject("malformed_message"))?;
        validate_token(&command.scope).map_err(|_| reject("malformed_message"))?;
        if command.authority_generation != self.generation {
            return Err(reject("stale_generation"));
        }

        let grant = self
            .grants
            .get(&command.grant_id)
            .ok_or_else(|| reject("unauthenticated"))?;
        if grant.authority_generation != self.generation
            || grant.principal_id != command.principal_id
        {
            return Err(reject("unauthenticated"));
        }
        if grant.expires_at_ms <= now_ms {
            return Err(reject("grant_expired"));
        }
        let key = DedupeKey {
            authority_generation: self.generation.clone(),
            principal_id: command.principal_id.clone(),
            command_id: command.command_id.clone(),
        };
        let fingerprint = fingerprint(&CommandFingerprint {
            scope: &command.scope,
            expected_revision: command.expected_revision,
            action: &command.action,
        })
        .map_err(|_| reject("malformed_message"))?;
        if let Some(cached) = self.outcomes.get(&key) {
            return if cached.fingerprint == fingerprint {
                cached.outcome.result()
            } else {
                Err(reject("command_id_reused"))
            };
        }
        if self.outcomes.len() >= MAX_DEDUPE_RESULTS {
            return Err(reject("busy"));
        }
        if command.scope != command.action.required_scope()
            || !grant.scopes.contains(&command.scope)
        {
            let rejected = reject("scope_denied");
            self.outcomes.insert(
                key,
                CachedOutcome {
                    fingerprint,
                    outcome: FinalOutcome::Rejected(rejected.clone()),
                },
            );
            return Err(rejected);
        }
        if command
            .expected_revision
            .is_some_and(|expected| expected != self.revision)
        {
            let rejected = reject("stale_revision");
            self.outcomes.insert(
                key,
                CachedOutcome {
                    fingerprint,
                    outcome: FinalOutcome::Rejected(rejected.clone()),
                },
            );
            return Err(rejected);
        }
        if matches!(command.action, AppAction::SetLevel { value } if value > 100) {
            let rejected = reject("invalid_action");
            self.outcomes.insert(
                key,
                CachedOutcome {
                    fingerprint,
                    outcome: FinalOutcome::Rejected(rejected.clone()),
                },
            );
            return Err(rejected);
        }
        let Some(next_revision) = self.revision.checked_add(1) else {
            let rejected = reject("revision_exhausted");
            self.outcomes.insert(
                key,
                CachedOutcome {
                    fingerprint,
                    outcome: FinalOutcome::Rejected(rejected.clone()),
                },
            );
            return Err(rejected);
        };
        match command.action {
            AppAction::SetLevel { value } => self.state.level = value,
            AppAction::Activate => self.state.active = true,
            AppAction::Deactivate => self.state.active = false,
        }
        self.revision = next_revision;
        let applied = Applied {
            command_id: command.command_id,
            ok: true,
            revision: self.revision,
            state: self.state.clone(),
        };
        self.outcomes.insert(
            key,
            CachedOutcome {
                fingerprint,
                outcome: FinalOutcome::Applied(applied.clone()),
            },
        );
        Ok(applied)
    }
}

fn fingerprint<T: Serialize>(value: &T) -> Result<[u8; 32], serde_json::Error> {
    let bytes = serde_json::to_vec(value)?;
    Ok(Sha256::digest(bytes).into())
}

fn validate_id(value: &str) -> Result<(), ()> {
    if value.len() < 8 {
        return Err(());
    }
    validate_token(value)
}

fn validate_token(value: &str) -> Result<(), ()> {
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return Err(());
    };
    if value.len() > MAX_TOKEN_BYTES
        || !first.is_ascii_alphanumeric()
        || !bytes
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b':' | b'-'))
    {
        return Err(());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn authority() -> Authority {
        let mut authority = Authority::new(
            "authority_demo_01".into(),
            AppState {
                active: false,
                level: 12,
            },
        )
        .expect("valid authority");
        authority
            .issue_grant(Grant {
                grant_id: "grant_demo_01".into(),
                principal_id: "principal_demo_01".into(),
                role: "controller".into(),
                scopes: BTreeSet::from(["example.control".into()]),
                authority_generation: "authority_demo_01".into(),
                expires_at_ms: 10_000,
            })
            .expect("valid grant");
        authority
    }

    fn fixture_command() -> CommandRequest {
        let fixtures: serde_json::Value =
            serde_json::from_str(include_str!("../../contracts/command-fixtures.json"))
                .expect("fixture JSON");
        serde_json::from_value(fixtures["validCommand"].clone()).expect("command fixture")
    }

    #[test]
    fn applies_once_and_returns_the_cached_outcome() {
        let mut authority = authority();
        let command = fixture_command();
        let first = authority.apply(command.clone(), 100).expect("first apply");
        let second = authority.apply(command, 101).expect("deduplicated apply");
        assert_eq!(first, second);
        assert_eq!(authority.snapshot().revision, 1);
        assert_eq!(authority.snapshot().state.level, 37);
    }

    #[test]
    fn rejects_command_id_reuse_with_changed_bytes() {
        let mut authority = authority();
        let command = fixture_command();
        authority.apply(command.clone(), 100).expect("first apply");
        let mut changed = command;
        changed.action = AppAction::SetLevel { value: 38 };
        assert_eq!(
            authority
                .apply(changed, 101)
                .expect_err("must reject")
                .error,
            "command_id_reused"
        );
        assert_eq!(authority.snapshot().state.level, 37);
    }

    #[test]
    fn caches_deterministic_final_rejections() {
        let mut authority = authority();
        let mut stale = fixture_command();
        stale.expected_revision = Some(1);
        let first = authority
            .apply(stale.clone(), 100)
            .expect_err("stale command must be rejected");

        let mut advancing = fixture_command();
        advancing.command_id = "command_demo_02".into();
        authority
            .apply(advancing, 101)
            .expect("independent command advances state");

        let retried = authority
            .apply(stale, 102)
            .expect_err("final rejection must be deduplicated");
        assert_eq!(retried, first);
        assert_eq!(retried.revision, 0);
        assert_eq!(authority.snapshot().revision, 1);
    }

    #[test]
    fn recovers_an_outcome_after_a_same_principal_grant_refresh() {
        let mut authority = authority();
        let first = authority
            .apply(fixture_command(), 100)
            .expect("initial command");
        authority
            .issue_grant(Grant {
                grant_id: "grant_demo_02".into(),
                principal_id: "principal_demo_01".into(),
                role: "controller".into(),
                scopes: BTreeSet::from(["example.control".into()]),
                authority_generation: "authority_demo_01".into(),
                expires_at_ms: 20_000,
            })
            .expect("replacement grant");
        let mut retry = fixture_command();
        retry.grant_id = "grant_demo_02".into();
        assert_eq!(authority.apply(retry, 101).expect("dedupe recovery"), first);
        assert_eq!(authority.snapshot().revision, 1);
    }

    #[test]
    fn enforces_revision_expiry_scope_and_generation() {
        let mut stale_revision = fixture_command();
        stale_revision.expected_revision = Some(1);
        assert_eq!(
            authority()
                .apply(stale_revision, 100)
                .expect_err("stale revision")
                .error,
            "stale_revision"
        );

        assert_eq!(
            authority()
                .apply(fixture_command(), 10_000)
                .expect_err("expired")
                .error,
            "grant_expired"
        );

        let mut wrong_scope = fixture_command();
        wrong_scope.scope = "example.upload".into();
        assert_eq!(
            authority()
                .apply(wrong_scope, 100)
                .expect_err("scope")
                .error,
            "scope_denied"
        );

        let mut stale_generation = fixture_command();
        stale_generation.authority_generation = "authority_old_01".into();
        assert_eq!(
            authority()
                .apply(stale_generation, 100)
                .expect_err("generation")
                .error,
            "stale_generation"
        );
    }

    #[test]
    fn rotation_preserves_product_state_but_revokes_access() {
        let mut authority = authority();
        authority
            .apply(fixture_command(), 100)
            .expect("initial command");
        authority
            .rotate_authority("authority_demo_02".into())
            .expect("rotate");
        assert_eq!(authority.snapshot().state.level, 37);
        assert_eq!(authority.snapshot().revision, 0);
        let mut old = fixture_command();
        old.authority_generation = "authority_demo_02".into();
        assert_eq!(
            authority.apply(old, 101).expect_err("grant revoked").error,
            "unauthenticated"
        );
    }
}
