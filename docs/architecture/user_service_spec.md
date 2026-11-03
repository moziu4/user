# Documentación del `user-service`

Índice de documentación modular. Lee solo el archivo necesario para ahorrar contexto:

- [Resumen y arquitectura](01-arquitectura.md): propósito, límites, capas y flujo de una petición.
- [Dominio y structs](02-dominio.md): `User`, `Membership`, `UserFact`, estados e IDs.
- [API HTTP y OpenAPI](03-api-openapi.yaml): contrato mínimo para consumidores internos y rutas públicas actuales.
- [NATS y contratos](04-nats.md): subjects, payloads, publicación, consumo e idempotencia.
- [Proxy, configuración y operación](05-proxy-configuracion.md): `TenantProxy`, variables de entorno, MongoDB y Docker.
- [Guía para encontrar información](06-guia-navegacion-proyecto.md): dónde buscar código, contratos, configuración, persistencia y pruebas.

## Estado del contrato

Los documentos separan **implementado** de **objetivo arquitectónico**. Antes de integrar otro servicio, usa el OpenAPI y los payloads NATS como referencia, y confirma los puntos marcados como pendientes.
