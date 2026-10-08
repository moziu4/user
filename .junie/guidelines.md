# Junie Guidelines: User Service Domain, Architecture & Fact Specifications

## 1. Contexto y Misión del `user-service`
`user-service` es el servicio central y genérico de identidad, perfil global, memberships (organizaciones/tenants) y hechos administrativos (`UserFact`) para la plataforma SaaS multi-tenant.

### Principios Fundamentales
- **Agnóstico del dominio vertical**: No debe contener lógica de inmigración, trámites, procedimientos, expedientes, RAG, ni IA.
- **Usuario Global**: El `User`, su `Email` y su `Phone` son globales en la plataforma. Nunca llevan `tenant_id`.
- **Relación con Tenants**: Se gestiona única y exclusivamente a través de `Membership` (`user_id` + `tenant_id` / `organization_id`).
- **User Facts**: Hechos administrativos globales asociados a un usuario, que registran procedencia (`source_tenant_id`, `source_service`, `source_case_id`) sin pertenecer exclusivamente a un tenant.

---

## 2. Convenciones de Arquitectura (DDD)
Estructura de capas existente en `src/`:
- `src/core/domain/<aggregate>/`: Modelos de dominio (`*_type.rs`), repositorios/traits (`*_repo.rs`), errores (`*_error.rs`), comandos (`*_commands.rs`).
- `src/core/operation/<aggregate>_ops.rs`: Casos de uso y orquestación de operaciones de negocio.
- `src/data/access/<aggregate>_repo.rs`: Implementación en MongoDB (`Mongo<Aggregate>Repo`).
- `src/data/access/migration/mongo/`: Migraciones e índices de MongoDB.
- `src/handlers/http/<aggregate>/`: Enrutadores y endpoints Actix-web (`*_routes.rs`).
- `src/handlers/message/`: Productores y consumidores NATS (`nats_service.rs`).

---

## 3. Modelo de Dominio: Aggregates y Entidades

### 3.1. `User` (Identidad / Perfil Global)
- **ID**: `perms::UserID` (`ObjectId`).
- **Datos**: `username`, `email`, `name`, `surname_1`, `surname_2`, `documents: Vec<IdentityDocument>`, `status`, marcas de tiempo.
- **Regla**: Sin `tenant_id`.

### 3.2. `Membership` (Relación User <-> Tenant / Organización)
- **Identidad propia**: `MembershipID`.
- **Unicidad lógica**: `(user_id, target)` (donde `target` es `Tenant(TenantID)` u `Organization(OrganizationID)`).
- **Campos**: `_id`, `user_id`, `target`, `role_id`, `status: MembershipStatus` (`Active`, `Inactive`, `Pending`, `Unverified`, `Suspended`).

### 3.3. `UserFact` (Nuevo Aggregate para Hechos Administrativos)
- **Propósito**: Hecho administrativo global y verificable sobre un usuario, originado por servicios externos (ej. `migration-service`).
- **Campos**:
  - `_id`: `Option<FactID>` (`ObjectId`)
  - `user_id`: `UserID`
  - `fact_type`: Enum `FactType` (`NIE`, `Passport`, `Nationality`, `DateOfBirth`, `ResidencePermit`, `ResidenceStatus`, `TIE`, `MunicipalRegistration`, etc.)
  - `value`: `String` / `serde_json::Value`
  - `status`: Enum `FactStatus` (`Active`, `Superseded`, `Conflicting`, `Revoked`)
  - `verification`: Enum `FactVerification` (`Verified`, `Unverified`, `PendingVerification`, `Rejected`)
  - `source_service`: `String` (ej. `"migration-service"`)
  - `source_tenant_id`: `Option<TenantID>` (procedencia/origen, no ownership)
  - `source_case_id`: `Option<String>`
  - `evidence_reference`: `Option<String>`
  - `created_at`, `updated_at`: `DateTime<Utc>`
  - `valid_from`, `valid_until`: `Option<DateTime<Utc>>`

---

## 4. Estrategia de Eventos NATS & Idempotencia
- **Consumo NATS**:
  - Subject principal: `user.fact.created`, `user.fact.updated`, `user.fact.revoked`.
  - Debe ser idempotente: deduplicación basada en `event_id` / hash de `(user_id, fact_type, value, source)`.
- **Publicación NATS**:
  - `user.created`, `user.updated`
  - `user.membership.created`, `user.membership.updated`
  - `user.fact.created`, `user.fact.updated`, `user.fact.revoked`

---

## 5. Endpoints HTTP Internos
- `GET /internal/users/{user_id}`: Perfil básico del usuario.
- `GET /internal/users/{user_id}/facts`: Lista de facts asociados (con filtros opcionales de status).
- `GET /internal/users/{user_id}/facts/{fact_type}`: Fact específico o histórico de dicho tipo.
- `GET /internal/users/{user_id}/context`: DTO optimizado con principio de mínimo privilegio para consumidores como `ai-service`.
