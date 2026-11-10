# Dominio y structs de interés

## `User`

Identidad global, identificada por `perms::UserID` (`ObjectId`). Incluye `username`, `email`, nombre, apellidos y documentos de identidad. No debe tener `tenant_id`.

`NewUser` añade la contraseña y permite un membership inicial. `IdentityDocument` contiene `doc_type` (`DNI`, `NIE`, `Passport`, `Other`), número, país y caducidad.

## `Membership`

Relación del usuario con un `TenantID` o `OrganizationID`, con `role_id` y `MembershipStatus`: `Active`, `Inactive`, `Pending`, `Unverified` o `Suspended`. La unicidad lógica recomendada es `(user_id, target)`.

Los comandos actuales son `create_tenant`, `create_organization`, `update_status` y `deactivate_user_from_organization`.

## `UserFact`

Hecho global trazable. Campos: `_id`, `user_id`, `fact_type`, `value` (`serde_json::Value`), `status`, `verification`, `source_service`, procedencia opcional (`source_tenant_id`, `source_case_id`), evidencia y periodo de validez.

`FactType` incluye `NIE`, `PASSPORT`, `NATIONALITY`, `DATE_OF_BIRTH`, `GENDER`, `RESIDENCE_PERMIT`, `RESIDENCE_STATUS`, `TIE`, `WORK_PERMIT`, `VISA_NUMBER`, `MUNICIPAL_REGISTRATION`, `SOCIAL_SECURITY_NUMBER`, `TAX_IDENTIFICATION` y `Other(String)`.

`FactStatus`: `ACTIVE`, `SUPERSEDED`, `CONFLICTING`, `REVOKED`. `FactVerification`: `VERIFIED`, `UNVERIFIED`, `PENDING_VERIFICATION`, `REJECTED`.

## DTO de contexto

`UserContextDTO` devuelve `user`, `contacts` y una lista resumida de facts. Está pensado para mínimo privilegio: consumidores como `ai-service` no necesitan acceder a colecciones Mongo directamente.

## Nota de alineación

El objetivo arquitectónico declara teléfono y email globales. El código actual todavía modela `tenant_id` opcional en `Phone` y `Address`; eliminarlo requiere una migración y coordinación, por lo que no se modifica en esta documentación.
