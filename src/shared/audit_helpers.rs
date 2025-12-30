use crate::{infrastructure::audit, pb::pb, shared::audit_action::AuditAction};

pub fn audit_input(
    tenant_id: String,
    action: AuditAction,
    target_type: Option<&str>,
    target_id: Option<String>,
    before_json: Option<serde_json::Value>,
    after_json: Option<serde_json::Value>,
    actor: Option<pb::ActorContext>,
) -> audit::AuditInput {
    let (actor_id, actor_type) = actor_fields(actor);
    audit::AuditInput {
        tenant_id,
        actor_id,
        actor_type,
        action,
        target_type: target_type.map(|v| v.to_string()),
        target_id,
        request_id: None,
        ip_address: None,
        user_agent: None,
        before_json,
        after_json,
        metadata_json: None,
    }
}

fn actor_fields(actor: Option<pb::ActorContext>) -> (Option<String>, String) {
    let actor_id = actor.as_ref().and_then(|a| {
        if a.actor_id.is_empty() {
            None
        } else {
            Some(a.actor_id.clone())
        }
    });
    let actor_type = actor
        .and_then(|a| {
            if a.actor_type.is_empty() {
                None
            } else {
                Some(a.actor_type)
            }
        })
        .unwrap_or_else(|| "system".to_string());
    (actor_id, actor_type)
}

pub fn to_json_opt<T: serde::Serialize>(value: Option<T>) -> Option<serde_json::Value> {
    value.and_then(|v| serde_json::to_value(v).ok())
}
