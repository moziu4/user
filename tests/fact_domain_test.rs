use std::str::FromStr;
use chrono::Utc;
use mongodb::bson::oid::ObjectId;
use perms::UserID;
use serde_json::json;
use user::core::domain::fact::{
    fact_commands::FactEventMessage,
    fact_type::{
        FactStatus, FactType, FactVerification,
        UserContactsDTO, UserContextDTO, UserFact, UserFactSummaryDTO, UserSummaryDTO,
    },
};
use user::utils::domains_ids::{FactID, TenantID};

#[test]
fn test_fact_type_parsing_and_display() {
    assert_eq!(FactType::from_str("NIE").unwrap(), FactType::Nie);
    assert_eq!(FactType::from_str("passport").unwrap(), FactType::Passport);
    assert_eq!(FactType::from_str("NATIONALITY").unwrap(), FactType::Nationality);
    assert_eq!(FactType::from_str("date_of_birth").unwrap(), FactType::DateOfBirth);
    assert_eq!(FactType::from_str("residence_permit").unwrap(), FactType::ResidencePermit);
    assert_eq!(FactType::from_str("TIE").unwrap(), FactType::Tie);
    assert_eq!(FactType::from_str("municipal_registration").unwrap(), FactType::MunicipalRegistration);
    assert_eq!(FactType::from_str("custom_fact").unwrap(), FactType::Other("custom_fact".to_string()));

    assert_eq!(FactType::Nie.to_string(), "NIE");
    assert_eq!(FactType::Passport.to_string(), "PASSPORT");
    assert_eq!(FactType::ResidencePermit.to_string(), "RESIDENCE_PERMIT");
}

#[test]
fn test_fact_status_and_verification_serialization() {
    let status = FactStatus::Active;
    let serialized_status = serde_json::to_string(&status).unwrap();
    assert_eq!(serialized_status, "\"ACTIVE\"");

    let verification = FactVerification::Verified;
    let serialized_verification = serde_json::to_string(&verification).unwrap();
    assert_eq!(serialized_verification, "\"VERIFIED\"");
}

#[test]
fn test_user_fact_serialization_and_deserialization() {
    let user_oid = ObjectId::new();
    let tenant_oid = ObjectId::new();
    let now = Utc::now();

    let fact = UserFact {
        _id: Some(FactID::new()),
        user_id: UserID::from_object_id(user_oid),
        fact_type: FactType::Nie,
        value: json!("X1234567A"),
        status: FactStatus::Active,
        verification: FactVerification::Verified,
        source_service: "migration-service".to_string(),
        source_tenant_id: Some(TenantID::from_object_id(tenant_oid)),
        source_case_id: Some("case_789".to_string()),
        evidence_reference: Some("doc_hash_123".to_string()),
        valid_from: Some(now),
        valid_until: None,
        created_at: now,
        updated_at: now,
    };

    let serialized = serde_json::to_string(&fact).expect("Debe serializar UserFact");
    assert!(serialized.contains("X1234567A"));
    assert!(serialized.contains("migration-service"));
    assert!(serialized.contains("NIE"));
    assert!(serialized.contains("ACTIVE"));
    assert!(serialized.contains("VERIFIED"));

    let deserialized: UserFact = serde_json::from_str(&serialized).expect("Debe deserializar UserFact");
    assert_eq!(deserialized.fact_type, FactType::Nie);
    assert_eq!(deserialized.value, json!("X1234567A"));
    assert_eq!(deserialized.source_service, "migration-service");
}

#[test]
fn test_nats_fact_event_message() {
    let user_oid = ObjectId::new();
    let event = FactEventMessage {
        event_id: Some("evt_123".to_string()),
        event: "user.fact.created".to_string(),
        user_id: UserID::from_object_id(user_oid),
        fact_type: FactType::ResidenceStatus,
        value: json!({"status": "GRANTED", "law": "14/2013"}),
        fact_id: Some(FactID::new()),
        status: FactStatus::Active,
        verification: FactVerification::Verified,
        source_service: "migration-service".to_string(),
        source_tenant_id: None,
        source_case_id: Some("exp_999".to_string()),
        evidence_reference: None,
        valid_from: None,
        valid_until: None,
        timestamp: Some(Utc::now()),
    };

    let serialized = serde_json::to_string(&event).expect("Debe serializar FactEventMessage");
    let deserialized: FactEventMessage = serde_json::from_str(&serialized).expect("Debe deserializar FactEventMessage");
    assert_eq!(deserialized.event, "user.fact.created");
    assert_eq!(deserialized.fact_type, FactType::ResidenceStatus);
    assert_eq!(deserialized.source_case_id, Some("exp_999".to_string()));
}

#[test]
fn test_user_context_dto() {
    let context_dto = UserContextDTO {
        user: UserSummaryDTO {
            id: "60c72b2f9b1d8b2bad000001".to_string(),
            username: "john_doe".to_string(),
            name: "John Doe".to_string(),
            surname_1: Some("Doe".to_string()),
            surname_2: None,
        },
        contacts: UserContactsDTO {
            email: "john@example.com".to_string(),
            phone: Some("+34600112233".to_string()),
        },
        facts: vec![
            UserFactSummaryDTO {
                id: Some("60c72b2f9b1d8b2bad000002".to_string()),
                fact_type: FactType::Nie,
                value: json!("Y9876543Z"),
                status: FactStatus::Active,
                verification: FactVerification::Verified,
                valid_until: None,
            },
        ],
    };

    let serialized = serde_json::to_string_pretty(&context_dto).expect("Debe serializar UserContextDTO");
    assert!(serialized.contains("john_doe"));
    assert!(serialized.contains("john@example.com"));
    assert!(serialized.contains("Y9876543Z"));
}
