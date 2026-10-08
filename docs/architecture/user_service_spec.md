# User Service - Arquitectura, Dominio y Especificación de UserFacts

Este documento describe la arquitectura conceptual y técnica del `user-service`, sus responsabilidades de dominio, el modelo de datos de aggregates (incluyendo `UserFact`), la estrategia de comunicación asíncrona mediante NATS y los contratos de API internos.

---

## 1. Misión y Responsabilidades

El `user-service` es el microservicio central de la plataforma SaaS multi-tenant responsable de:
1. **Identidad y autenticación global del usuario**.
2. **Perfil global del usuario** (datos personales, emails, teléfonos, documentos identificativos de la persona).
3. **Memberships** (relación y roles del usuario con respecto a Tenants u Organizaciones).
4. **User Facts** (almacenamiento y exposición de hechos administrativos globales verificados y reportados por otros dominios).
5. **APIs internas y publicación/suscripción de eventos** de ciclo de vida del usuario.

### Límites de Dominio (Lo que NO debe contener)
- Lógica de negocio de dominios verticales (trámites de extranjería, expedientes, RAG, IA, procedimientos judiciales/administrativos).
- Datos o documentos específicos pertenecientes exclusivamente a la operativa interna de un tenant.

---

## 2. Arquitectura DDD y Estructura del Proyecto

El servicio sigue Domain-Driven Design (DDD) adaptado al ecosistema Rust:

```text
src/
├── core/
│   ├── domain/
│   │   ├── user/          # Aggregate User (perfil e identidad global)
│   │   ├── auth/          # Autenticación, tokens, credenciales
│   │   ├── membership/    # Aggregate Membership (vínculo User <-> Tenant/Org)
│   │   ├── perm/          # Roles y permisos
│   │   └── fact/          # Aggregate UserFact (hechos administrativos)
│   └── operation/         # Casos de uso / Orquestación (UserOps, MembershipOps, FactOps)
├── data/
│   ├── access/            # Repositorios MongoDB (MongoUserRepo, MongoFactRepo, etc.)
│   └── proxy/             # Clientes o adaptadores a servicios externos
├── handlers/
│   ├── http/              # Controladores y rutas HTTP Actix-web (/api, /internal)
│   └── message/           # Consumidores y productores NATS
└── utils/                 # Tipos fuertes de IDs y helpers
```

---

## 3. Modelo de Dominio y Aggregates

### 3.1. Aggregate `User`
- **Naturaleza**: Global en toda la plataforma. **No contiene `tenant_id`**.
- **Entidades asociadas**: Email, Teléfono, Documentos identificativos base.

### 3.2. Aggregate `Membership`
- **Naturaleza**: Modela la relación `User <-> Tenant / Organización`.
- **Identidad**: `MembershipID` (`_id`).
- **Unicidad Lógica**: Par `(user_id, target)`.
- **Ciclo de vida**: Estados `Active`, `Inactive`, `Pending`, `Unverified`, `Suspended`.

### 3.3. Aggregate `UserFact`
- **Naturaleza**: Hecho administrativo global y trazable sobre el usuario (ej. obtención de NIE, estado de residencia, empadronamiento).
- **Procedencia**: Producido y emitido por servicios de dominio externos (ej. `migration-service`) mediante eventos.
- **Campos del Aggregate**:
  - `_id`: Identificador único del fact.
  - `user_id`: Identificador del usuario al que pertenece el hecho.
  - `fact_type`: Tipo de hecho administrativo catalogado (`NIE`, `PASSPORT`, `NATIONALITY`, `RESIDENCE_PERMIT`, `RESIDENCE_STATUS`, `TIE`, `MUNICIPAL_REGISTRATION`, etc.).
  - `value`: Valor o payload estructurado del hecho.
  - `status`: Estado del fact (`ACTIVE`, `SUPERSEDED`, `CONFLICTING`, `REVOKED`).
  - `verification`: Nivel de verificación (`VERIFIED`, `UNVERIFIED`, `PENDING_VERIFICATION`, `REJECTED`).
  - `source_service`: Servicio emisor (ej. `"migration-service"`).
  - `source_tenant_id`: Tenant desde el que se originó/verificó la información (para auditoría/trazabilidad).
  - `source_case_id`: Identificador del expediente o caso origen (opcional).
  - `evidence_reference`: Referencia opcional al documento o evidencia.
  - `valid_from` / `valid_until`: Periodo de validez temporal (opcional).
  - `created_at` / `updated_at`: Marcas temporales.

---

## 4. Mensajería NATS

### Consumo de Eventos
- `user.fact.created`: Ingesta de hechos administrativos emitidos por otros servicios.
- `user.fact.updated`: Actualización de estado o corrección de hechos.
- `user.fact.revoked`: Revocación de un hecho previamente válido.
- **Idempotencia**: Deduplicación por `event_id` o control de versión/hash para evitar duplicidad de hechos ante reenvíos de mensajes.

### Publicación de Eventos
- Eventos de identidad y membership: `user.created`, `user.updated`, `user.membership.created`, `user.membership.updated`.
- Eventos de fact: `user.fact.created`, `user.fact.updated`, `user.fact.revoked`.

---

## 5. Contratos de API HTTP (Endpoints Internos)

- `GET /internal/users/{user_id}`: Información y perfil básico.
- `GET /internal/users/{user_id}/facts`: Lista de hechos administrativos asociados al usuario.
- `GET /internal/users/{user_id}/facts/{fact_type}`: Hechos específicos por tipo.
- `GET /internal/users/{user_id}/context`: DTO optimizado para servicios de IA / RAG (`ai-service`), siguiendo el principio de mínimo privilegio.
