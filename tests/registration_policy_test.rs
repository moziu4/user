use user::data::proxy::tenant::{Tenant, RegistrationType, TenantState};
use user::core::domain::user::Status;
use user::core::domain::membership::MembershipStatus;
use user::handlers::message::nats_service::{VerificationEmailEvent, InvitationDeeplinkEvent};

#[test]
fn test_deserialize_tenant_self_service() {
    let json_data = r#"{
        "host": "tenant1.example.com",
        "name": "Tenant One",
        "registration_type": "self_service"
    }"#;

    let tenant: Tenant = serde_json::from_str(json_data).expect("Debe deserializar Tenant");
    assert_eq!(tenant.registration_type, RegistrationType::SelfService);
    assert_eq!(tenant.state, TenantState::Active);
}

#[test]
fn test_deserialize_tenant_deplinking() {
    let json_data = r#"{
        "host": "tenant2.example.com",
        "name": "Tenant Two",
        "registration_type": "deplinking"
    }"#;

    let tenant: Tenant = serde_json::from_str(json_data).expect("Debe deserializar Tenant con deplinking");
    assert_eq!(tenant.registration_type, RegistrationType::Deplinking);
}

#[test]
fn test_deserialize_tenant_deeplinking_alias() {
    let json_data = r#"{
        "host": "tenant3.example.com",
        "name": "Tenant Three",
        "registration_type": "deeplinking"
    }"#;

    let tenant: Tenant = serde_json::from_str(json_data).expect("Debe deserializar Tenant con deeplinking alias");
    assert_eq!(tenant.registration_type, RegistrationType::Deplinking);
}

#[test]
fn test_user_and_membership_statuses() {
    assert_eq!(Status::Pending, Status::Pending);
    assert_eq!(Status::Unverified, Status::Unverified);
    assert_eq!(MembershipStatus::Pending, MembershipStatus::Pending);
    assert_eq!(MembershipStatus::Unverified, MembershipStatus::Unverified);
}

#[test]
fn test_nats_verification_and_invitation_events() {
    let verification_event = VerificationEmailEvent {
        user_id: "user123".to_string(),
        username: "jdoe".to_string(),
        email: "jdoe@example.com".to_string(),
        token: "tok-12345".to_string(),
        verification_url: Some("/api/users/verify?token=tok-12345&user_id=user123".to_string()),
        tenant_id: Some("tenant-1".to_string()),
    };

    let serialized_verification = serde_json::to_string(&verification_event).expect("Serializa verification event");
    assert!(serialized_verification.contains("tok-12345"));
    assert!(serialized_verification.contains("jdoe@example.com"));

    let invitation_event = InvitationDeeplinkEvent {
        user_id: "user456".to_string(),
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
        token: "tok-67890".to_string(),
        deeplink_url: Some("/invite?token=tok-67890&user_id=user456".to_string()),
        tenant_id: Some("tenant-2".to_string()),
        organization_id: Some("org-1".to_string()),
    };

    let serialized_invitation = serde_json::to_string(&invitation_event).expect("Serializa invitation event");
    assert!(serialized_invitation.contains("tok-67890"));
    assert!(serialized_invitation.contains("alice@example.com"));
    assert!(serialized_invitation.contains("org-1"));
}
