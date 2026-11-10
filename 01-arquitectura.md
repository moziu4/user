# `user-service`: propósito y arquitectura

## Para qué se usa

Es el servicio central de identidad global, perfil, autenticación, memberships y hechos administrativos (`UserFact`) de una plataforma SaaS multi-tenant.

No debe contener lógica de trámites, expedientes, RAG, IA ni datos propiedad exclusiva de un tenant. La pertenencia a tenants u organizaciones se expresa mediante `Membership`; `source_tenant_id` de un fact solo indica procedencia.

## Capas

```text
handlers/http       Actix-web: rutas públicas e internas
handlers/message    NATS: productores y consumidores
core/operation      casos de uso y orquestación
core/domain         aggregates, DTOs, repositorios y errores
data/access         repositorios MongoDB y migraciones
data/proxy          clientes de servicios externos, actualmente Tenant
```

Los entry points principales son `src/main.rs`, `src/context.rs` y `src/handlers/http`. Los casos de uso son `UserOps`, `MembershipOps` y `FactOps`.

## Flujo típico

1. Actix valida ruta/payload y construye el caso de uso.
2. `core/operation` aplica reglas y usa traits de `core/domain`.
3. `data/access` persiste en MongoDB.
4. `handlers/message/nats_service.rs` publica eventos de integración o consume comandos/facts.

## Dependencias de ejecución

- Rust + Actix Web.
- MongoDB para usuarios, memberships y facts.
- NATS opcional para mensajería; la URL por defecto es `nats://localhost:4222`.
- `perms` para `UserID` y permisos.
- Servicio Tenant vía HTTP para consultas de tenant.
