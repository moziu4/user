# Proxy, configuración y operación

## `TenantProxy`

`src/data/proxy/tenant_proxy.rs` hace `GET {TENANT_SERVICE_URL}/api/tenant/{tenant_id}` y deserializa `Tenant`. Si `TENANT_SERVICE_URL` no existe usa `http://localhost:4002`. `get_registration_type` devuelve `SelfService` si la consulta falla.

El proxy solo obtiene configuración/estado de tenant; no convierte al tenant en propietario de `User` o `UserFact`.

## Variables

| Variable | Uso | Valor local indicado |
|---|---|---|
| `MONGO_URI` | conexión Mongo | `mongodb://dev:dev_pass@localhost:57017/user` |
| `MONGO_DATABASE` | base | `user` |
| `NATS_URL` | NATS | `nats://localhost:4222` |
| `HTTP_BIND` | bind HTTP | `0.0.0.0:2030` |
| `CATALOGS_PATH` | catálogos | URL GitHub de `user-messages` |
| `SECRET_KEY` | secretos de aplicación | definir fuera del repositorio |

## Docker

`docker-compose.yml` levanta Mongo, NATS, Redis y `user`. Antes de desplegar, alinear los hosts `mongo_service`, `nats_service` y `redis_service` con los nombres reales de servicio (`mongo`, `nats`, `redis`) o con aliases de red. Es un acuerdo de infraestructura pendiente y no se cambia aquí porque afecta al entorno compartido.
