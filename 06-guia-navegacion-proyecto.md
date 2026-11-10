# Guía para encontrar información del proyecto

Este documento sirve como mapa de navegación para localizar rápidamente información en todo el repositorio `user-service`. Empieza por el índice de [documentación del servicio](user_service_spec.md) y después consulta la zona que corresponda.

## Punto de entrada recomendado

Para entender una funcionalidad concreta, sigue este orden:

1. Busca su ruta HTTP en `src/handlers/http/` o su subject NATS en `src/handlers/message/nats_service.rs`.
2. Sigue la llamada hasta `src/core/operation/`, donde se encuentra la orquestación del caso de uso.
3. Consulta los tipos, comandos, traits y errores en `src/core/domain/`.
4. Revisa la implementación MongoDB correspondiente en `src/data/access/`.
5. Comprueba configuración, dependencias externas y despliegue en `Cargo.toml`, `.env` y `docker-compose.yml`.

## Mapa del repositorio

| Necesidad | Dónde buscar | Qué contiene |
|---|---|---|
| Arranque del servicio | `src/main.rs`, `src/lib.rs` | Inicialización de configuración, MongoDB, HTTP y NATS |
| Estado compartido | `src/context.rs` | Contexto y dependencias usadas por handlers y operaciones |
| Modelos de dominio | `src/core/domain/` | Structs, enums, comandos, repositorios y errores |
| Usuarios | `src/core/domain/user/` | `User`, `UserData`, documentos, teléfonos, direcciones y `UserRepo` |
| Memberships | `src/core/domain/membership/` | Estados, comandos, DTOs y contratos de membership |
| User facts | `src/core/domain/fact/` | `UserFact`, `FactType`, estados, verificación y eventos |
| Autenticación | `src/core/domain/auth/` | Credenciales, sesiones y contratos de autenticación |
| Permisos | `src/core/domain/perm/` | Errores y modelos relacionados con permisos |
| Casos de uso | `src/core/operation/` | `UserOps`, `MembershipOps`, `FactOps`, `AuthOps` y operaciones de catálogos |
| Rutas HTTP | `src/handlers/http/` | Enrutadores y handlers Actix-web |
| API interna | `src/handlers/http/internal/` | Perfil, contexto y operaciones de `UserFact` |
| API de usuarios | `src/handlers/http/users/` | Alta y consulta de usuarios |
| API de memberships | `src/handlers/http/membership/` | Creación, consulta y comandos de memberships |
| Mensajería | `src/handlers/message/nats_service.rs` | Publicación, suscripciones y serialización NATS |
| Persistencia | `src/data/access/` | Repositorios MongoDB por aggregate |
| Migraciones e índices | `src/data/access/migration/` | Creación y actualización de estructuras MongoDB |
| Catálogos | `src/data/catalog_importer/`, `src/core/operation/catalogs_ops.rs` | Carga y acceso a mensajes de catálogo |
| Proxy de tenants | `src/data/proxy/tenant_proxy.rs`, `src/data/proxy/tenant.rs` | Cliente HTTP y modelos del servicio de tenants |
| IDs compartidos | `src/utils/domains_ids.rs`, dependencia `perms` | Tipos fuertes como `UserID`, `TenantID` y `OrganizationID` |
| Errores generales | `src/error.rs` | Errores compartidos y conversión de errores |

## Cómo localizar una funcionalidad

### Si parte de una petición HTTP

Busca primero el prefijo de ruta en `src/handlers/http/`. Los módulos se registran desde `src/handlers/http/mod.rs`. Después sigue el handler hasta el método de operación y finalmente hasta el repositorio.

Ejemplo para facts:

```text
src/handlers/http/internal/internal_routes.rs
  -> src/core/operation/fact_ops.rs
  -> src/core/domain/fact/fact_type.rs
  -> src/core/domain/fact/fact_repo.rs
  -> src/data/access/fact_repo.rs
```

El contrato resumido está en [`03-api-openapi.yaml`](03-api-openapi.yaml).

### Si parte de un mensaje NATS

Busca el subject y los métodos `publish` o `subscribe` en `src/handlers/message/nats_service.rs`. Consulta después el enum de comandos o el DTO usado como payload.

Ejemplos:

```text
user.fact.*   -> src/core/domain/fact/fact_commands.rs
              -> src/core/operation/fact_ops.rs

membership.*  -> src/core/domain/membership/membership_commands.rs
              -> src/core/operation/membership_ops.rs
```

Los subjects y contratos están resumidos en [`04-nats.md`](04-nats.md).

### Si parte de un cambio en MongoDB

Busca el trait de repositorio en `src/core/domain/<aggregate>/*_repo.rs`, la implementación en `src/data/access/<aggregate>_repo.rs` y los índices o migraciones en `src/data/access/migration/`.

Las colecciones se conectan desde el arranque/contexto; revisa también `src/db.rs` y `src/context.rs`.

## Configuración y ejecución

- `Cargo.toml`: versión de Rust package y dependencias.
- `.env`: valores locales como `MONGO_URI`, `MONGO_DATABASE`, `NATS_URL`, `HTTP_BIND`, `CATALOGS_PATH` y `SECRET_KEY`.
- `docker-compose.yml`: servicios MongoDB, NATS, Redis y `user`, puertos, red y variables Docker.
- `Dockerfile`: construcción de la imagen del servicio.
- `docker.sh`: utilidades de ejecución Docker.
- `rustfmt.toml`: formato del código Rust.
- `init-mongo/`: inicialización relacionada con MongoDB.

Para conexiones externas consulta [`05-proxy-configuracion.md`](05-proxy-configuracion.md), especialmente `TenantProxy` y los nombres de host Docker.

## Pruebas y validación

Las pruebas del repositorio están en `tests/`. Para entender qué debe validarse, relaciona cada prueba con:

- el handler o endpoint que ejercita;
- el caso de uso de `src/core/operation/`;
- el repositorio o dependencia externa que sustituye;
- el contrato HTTP o NATS documentado.

Si no existe una prueba para una ruta o un flujo NATS, la implementación y sus DTOs son la fuente primaria actual; cualquier contrato nuevo debe documentarse aquí y cubrirse con una prueba proporcional.

## Búsqueda eficiente

Desde el IDE, usa la búsqueda de texto sobre `src/` con el nombre del endpoint, subject, struct o método. Como alternativa, busca por estas cadenas:

| Quiero encontrar | Texto útil |
|---|---|
| Registro de rutas | `configure`, `service`, `route`, `scope` |
| Publicación NATS | `publish`, `user.fact.`, `membership.command` |
| Suscripciones NATS | `subscribe`, `user.fact.>`, `membership.>` |
| Repositorio Mongo | `Mongo`, `Collection`, `get_collection` |
| Variables de entorno | `std::env`, `var(`, nombre de la variable |
| Identificadores | `UserID`, `TenantID`, `OrganizationID`, `FactID` |

Cuando una funcionalidad afecte a otro servicio, busca primero el contrato aquí (`03-api-openapi.yaml` o `04-nats.md`) y luego verifica el consumidor/productor externo. No copies lógica vertical de negocio al `user-service`; registra únicamente el acuerdo de integración necesario.