# NATS y contratos de integración

Implementación en `src/handlers/message/nats_service.rs`. Los payloads son JSON.

## Publica

| Subject | Payload | Uso |
|---|---|---|
| `user.created` / `user.updated` | `UserMessage` | ciclo de vida básico (`user_id`, `username`, `email`, `event_type`) |
| `user.verification.email_requested` | `VerificationEmailEvent` | solicitar correo de verificación |
| `user.invitation.deeplink_requested` | `InvitationDeeplinkEvent` | solicitar deeplink de invitación |
| `user.fact.created` | `FactEventMessage` | fact creado |
| `user.fact.updated` | `FactEventMessage` | fact actualizado |
| `user.fact.revoked` | `FactEventMessage` | fact revocado |
| `membership.command[.<id>]` | `MembershipCommand` | ejecutar comando de membership |

`FactEventMessage` lleva `event_id` opcional, `event`, `user_id`, tipo, valor, estado, verificación, procedencia, validez, `fact_id` y `timestamp`.

## Consume

- `user.fact.>`: crea facts, o revoca por `fact_id` cuando el subject/evento es `revoked`.
- `membership.>`: deserializa `MembershipCommand`; un sufijo adicional se interpreta como ID de membership.

## Idempotencia y acuerdos

Los productores deben enviar `event_id` estable. El consumidor lo propaga al caso de uso para deduplicación. El contrato recomendado es no reprocesar un mismo `event_id`; si se usa un productor legacy sin ID, debe acordarse un hash de usuario, tipo, valor y servicio.

Los subjects `user.membership.created/updated` del diseño objetivo aún no aparecen como publicadores explícitos en este archivo; no deben darse por disponibles sin implementar o acordar ese cambio.
