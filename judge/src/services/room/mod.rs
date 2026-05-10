// Room runtime: event-log-driven, lease-based failover.
// AI-vs-AI matches share this runtime via the stdio bot transport.
// Spec: judge/protocols/architecture.md.

pub mod bot;
pub mod logic;
pub mod ws;
