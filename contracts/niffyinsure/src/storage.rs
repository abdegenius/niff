use soroban_sdk::{contracttype, Address, Env};

#[contracttype]
pub enum DataKey {
    Admin,
    Token,
    /// (holder, policy_id) — policy_id is per-holder u32
    Policy(Address, u32),
    /// Per-holder policy counter; next policy_id = counter + 1
    PolicyCounter(Address),
    Claim(u64),
    /// (claim_id, voter_address) → VoteOption
    Vote(u64, Address),
    /// Vec<Address> of all current active policyholders (voters)
    Voters,
    /// Global monotonic claim id counter
    ClaimCounter,

    // ═══════════════════════════════════════════════════════════════════════════
    // ORACLE / PARAMETRIC TRIGGER STORAGE (experimental only)
    //
    // ⚠️  LEGAL / COMPLIANCE REVIEW GATE: These storage keys are reserved for
    // future oracle-triggered parametric insurance functionality.
    //
    // Do NOT use these in production without:
    //   • Completed regulatory classification review
    //   • Legal review of smart contract-triggered payouts
    //   • Game-theoretic analysis of oracle incentivization
    //   • Cryptographic design review for signature verification
    //
    // Compilation gated by `#[cfg(feature = "experimental")]`.
    // Default builds have no code path to write these keys.
    // ═══════════════════════════════════════════════════════════════════════════

    /// Global monotonic trigger counter; next trigger_id = counter + 1
    #[cfg(feature = "experimental")]
    TriggerCounter,
    /// (trigger_id) → OracleTrigger
    #[cfg(feature = "experimental")]
    OracleTrigger(u64),
    /// (trigger_id) → TriggerStatus
    #[cfg(feature = "experimental")]
    TriggerStatus(u64),
    /// Admin-configured oracle source whitelist
    #[cfg(feature = "experimental")]
    OracleWhitelist,
    /// Whether oracle triggers are globally enabled (admin toggle)
    /// Default: false (disabled).  Must be explicitly enabled after review.
    #[cfg(feature = "experimental")]
    OracleEnabled,
    /// Reserved slot for future oracle configuration (keys, thresholds, etc.)
    /// DO NOT write to this slot until cryptographic design is finalized.
    #[cfg(feature = "experimental")]
    OracleConfig,
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&DataKey::Admin, admin);
}

/// Used by initialize and admin drain (feat/admin).
#[allow(dead_code)]
pub fn get_admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}

pub fn set_token(env: &Env, token: &Address) {
    env.storage().instance().set(&DataKey::Token, token);
}

/// Used by claim payout (feat/claim-voting).
#[allow(dead_code)]
pub fn get_token(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Token).unwrap()
}

/// Returns the next policy_id for `holder` and increments the counter.
/// Used by feat/policy-lifecycle.
#[allow(dead_code)]
pub fn next_policy_id(env: &Env, holder: &Address) -> u32 {
    let key = DataKey::PolicyCounter(holder.clone());
    let next: u32 = env.storage().persistent().get(&key).unwrap_or(0) + 1;
    env.storage().persistent().set(&key, &next);
    next
}

/// Returns the next global claim_id and increments the counter.
/// Used by feat/claim-voting.
#[allow(dead_code)]
pub fn next_claim_id(env: &Env) -> u64 {
    let next: u64 = env
        .storage()
        .instance()
        .get(&DataKey::ClaimCounter)
        .unwrap_or(0u64)
        + 1;
    env.storage().instance().set(&DataKey::ClaimCounter, &next);
    next
}

pub fn get_claim_counter(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::ClaimCounter)
        .unwrap_or(0u64)
}

pub fn get_policy_counter(env: &Env, holder: &Address) -> u32 {
    env.storage()
        .persistent()
        .get(&DataKey::PolicyCounter(holder.clone()))
        .unwrap_or(0u32)
}

pub fn has_policy(env: &Env, holder: &Address, policy_id: u32) -> bool {
    env.storage()
        .persistent()
        .has(&DataKey::Policy(holder.clone(), policy_id))
}

// ═════════════════════════════════════════════════════════════════════════════
// ORACLE / PARAMETRIC TRIGGER STORAGE HELPERS (experimental only)
//
// ⚠️  LEGAL / COMPLIANCE REVIEW GATE: These functions are non-operational
// stubs.  They panic in default builds and must NOT be called until:
//   • Regulatory classification is complete
//   • Legal review approves automatic trigger-to-claim flow
//   • Game-theoretic safeguards are implemented
//   • Cryptographic signature verification is designed and audited
//
// PRODUCTION SAFETY: Default builds (without `experimental` feature)
// will panic if any of these functions are called, ensuring oracle
// triggers cannot be processed accidentally.
// ═════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "experimental")]
use crate::types::{OracleTrigger, TriggerStatus};

/// Returns whether oracle triggers are globally enabled.
///
/// ⚠️  DEFAULT IS FALSE: Oracle triggers must be explicitly enabled by admin
/// after completing all required reviews (see DESIGN-ORACLE.md).
#[cfg(feature = "experimental")]
pub fn is_oracle_enabled(env: &Env) -> bool {
    env.storage()
        .instance()
        .get(&DataKey::OracleEnabled)
        .unwrap_or(false)
}

/// Enable or disable oracle triggers globally.
///
/// ⚠️  ADMIN ACTION REQUIRED: This should remain false until:
///   • Cryptographic design review is complete
///   • Legal/compliance has approved parametric triggers
///   • Game-theoretic safeguards are implemented
#[cfg(feature = "experimental")]
pub fn set_oracle_enabled(env: &Env, enabled: bool) {
    env.storage().instance().set(&DataKey::OracleEnabled, &enabled);
}

/// Returns the next trigger_id and increments the counter.
///
/// ⚠️  PRODUCTION NOTE: Trigger ID generation must include replay protection.
/// Current implementation is a placeholder.
#[cfg(feature = "experimental")]
pub fn next_trigger_id(env: &Env) -> u64 {
    let key = DataKey::TriggerCounter;
    let next: u64 = env
        .storage()
        .instance()
        .get(&key)
        .unwrap_or(0u64)
        + 1;
    env.storage().instance().set(&key, &next);
    next
}

/// Store an oracle trigger.
///
/// ⚠️  SECURITY: Signature verification must be performed BEFORE calling
/// this function.  See validate_oracle_trigger() in validate.rs.
#[cfg(feature = "experimental")]
pub fn set_oracle_trigger(env: &Env, trigger_id: u64, trigger: &OracleTrigger) {
    env.storage()
        .persistent()
        .set(&DataKey::OracleTrigger(trigger_id), trigger);
}

/// Retrieve an oracle trigger by ID.
#[cfg(feature = "experimental")]
pub fn get_oracle_trigger(env: &Env, trigger_id: u64) -> Option<OracleTrigger> {
    env.storage()
        .persistent()
        .get(&DataKey::OracleTrigger(trigger_id))
}

/// Update trigger status.
#[cfg(feature = "experimental")]
pub fn set_trigger_status(env: &Env, trigger_id: u64, status: TriggerStatus) {
    env.storage()
        .persistent()
        .set(&DataKey::TriggerStatus(trigger_id), &status);
}

/// Get trigger status.
#[cfg(feature = "experimental")]
pub fn get_trigger_status(env: &Env, trigger_id: u64) -> Option<TriggerStatus> {
    env.storage()
        .persistent()
        .get(&DataKey::TriggerStatus(trigger_id))
}

// ═════════════════════════════════════════════════════════════════════════════
// STUB IMPLEMENTATIONS FOR DEFAULT (NON-EXPERIMENTAL) BUILDS
//
// These functions ensure that default builds CANNOT process oracle triggers.
// If called in a non-experimental build, they will panic at runtime.
// This is intentional: it creates a hard failure mode that prevents accidental
// oracle trigger processing in production.
// ═════════════════════════════════════════════════════════════════════════════

#[cfg(not(feature = "experimental"))]
use crate::types::{OracleTrigger, TriggerStatus};

/// Stub: Panics in default builds to prevent oracle trigger processing.
///
/// ⚠️  DO NOT REMOVE THIS FUNCTION.  It ensures production safety by
/// creating a compile-time guarantee that oracle triggers cannot be
/// processed without the experimental feature flag.
#[cfg(not(feature = "experimental"))]
#[allow(dead_code)]
pub fn is_oracle_enabled(_env: &Env) -> bool {
    panic!(
        "ORACLE_TRIGGERS_DISABLED: Oracle trigger processing is not enabled in this build. \
         Default production builds cannot process oracle triggers. \
         See DESIGN-ORACLE.md for activation requirements."
    )
}

/// Stub: Panics in default builds.
#[cfg(not(feature = "experimental"))]
#[allow(dead_code)]
pub fn set_oracle_enabled(_env: &Env, _enabled: bool) {
    panic!(
        "ORACLE_TRIGGERS_DISABLED: Oracle trigger processing is not enabled in this build. \
         Default production builds cannot process oracle triggers. \
         See DESIGN-ORACLE.md for activation requirements."
    )
}

/// Stub: Panics in default builds.
#[cfg(not(feature = "experimental"))]
#[allow(dead_code)]
pub fn next_trigger_id(_env: &Env) -> u64 {
    panic!(
        "ORACLE_TRIGGERS_DISABLED: Oracle trigger ID generation is not enabled in this build. \
         Default production builds cannot process oracle triggers. \
         See DESIGN-ORACLE.md for activation requirements."
    )
}

/// Stub: Panics in default builds.
#[cfg(not(feature = "experimental"))]
#[allow(dead_code)]
pub fn set_oracle_trigger(_env: &Env, _trigger_id: u64, _trigger: &OracleTrigger) {
    panic!(
        "ORACLE_TRIGGERS_DISABLED: Oracle trigger storage is not enabled in this build. \
         Default production builds cannot process oracle triggers. \
         See DESIGN-ORACLE.md for activation requirements."
    )
}

/// Stub: Panics in default builds.
#[cfg(not(feature = "experimental"))]
#[allow(dead_code)]
pub fn get_oracle_trigger(_env: &Env, _trigger_id: u64) -> Option<OracleTrigger> {
    panic!(
        "ORACLE_TRIGGERS_DISABLED: Oracle trigger retrieval is not enabled in this build. \
         Default production builds cannot process oracle triggers. \
         See DESIGN-ORACLE.md for activation requirements."
    )
}

/// Stub: Panics in default builds.
#[cfg(not(feature = "experimental"))]
#[allow(dead_code)]
pub fn set_trigger_status(_env: &Env, _trigger_id: u64, _status: TriggerStatus) {
    panic!(
        "ORACLE_TRIGGERS_DISABLED: Oracle trigger status updates are not enabled in this build. \
         Default production builds cannot process oracle triggers. \
         See DESIGN-ORACLE.md for activation requirements."
    )
}

/// Stub: Panics in default builds.
#[cfg(not(feature = "experimental"))]
#[allow(dead_code)]
pub fn get_trigger_status(_env: &Env, _trigger_id: u64) -> Option<TriggerStatus> {
    panic!(
        "ORACLE_TRIGGERS_DISABLED: Oracle trigger status retrieval is not enabled in this build. \
         Default production builds cannot process oracle triggers. \
         See DESIGN-ORACLE.md for activation requirements."
    )
}
