//! Service bus publisher for the Bundu ecosystem.
//!
//! Implements the v4.0.4 service bus contract:
//! - Standard envelope (event_id, event_type, occurred_at, source, actor,
//!   subject, payload, trace, schema_org_type)
//! - Topic naming: `<schema>.<entity>.<event>` (e.g. `identity.person.created`)
//! - Cloudflare Queues as Phase 1 transport
//! - Transport-independent contract: swap to Redpanda in Phase 3 without
//!   changing the [`Publisher`] interface
//!
//! The [`Publisher`] trait is what the gateway calls. The
//! [`CloudflareQueuesPublisher`] is the concrete Phase 1 transport.
//! Adding a [`RedpandaPublisher`] later is purely additive.

#![forbid(unsafe_code)]
#![warn(missing_docs, clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

/// Errors from the service bus layer.
#[derive(Debug, Error)]
pub enum ServiceBusError {
    /// Configuration error.
    #[error("service bus config error: {0}")]
    Config(String),
    /// HTTP error talking to the queue transport.
    #[error("service bus http error: {0}")]
    Http(#[from] reqwest::Error),
    /// JSON serialisation error.
    #[error("service bus serialise error: {0}")]
    Serialise(#[from] serde_json::Error),
    /// Event_type was not registered in the schema registry.
    #[error("unknown event_type: {0}")]
    UnknownEventType(String),
    /// Queue transport returned an error.
    #[error("queue transport error: {0}")]
    Transport(String),
}

/// Result alias.
pub type ServiceBusResult<T> = std::result::Result<T, ServiceBusError>;

/// The actor section of the envelope — who initiated this event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Actor {
    /// The person who performed the action, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub person_id: Option<Uuid>,
    /// The active entity context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_id: Option<Uuid>,
    /// The active role.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_role: Option<String>,
}

/// The subject section — what the event is about.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Subject {
    /// E.g. `identity.person`, `identity.entity`, `entity.membership`.
    #[serde(rename = "type")]
    pub subject_type: String,
    /// The id of the subject row.
    pub id: Uuid,
}

/// The source section — which service emitted the event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Source {
    /// Service identifier, e.g. `bundu-identity`.
    pub service: String,
    /// Instance identifier — useful for debugging multi-region.
    pub instance: String,
    /// Environment: `production`, `staging`, `dev`.
    pub environment: String,
}

/// The trace section — for distributed tracing across consumers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Trace {
    /// OpenTelemetry-compatible trace id.
    pub trace_id: Uuid,
    /// Causation chain — which event caused this one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_event_id: Option<Uuid>,
    /// Origin of the action.
    pub causation: Causation,
}

/// Why this event happened.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Causation {
    /// Triggered by a user request.
    UserRequest,
    /// Triggered by another system event.
    System,
    /// Triggered by a sync replication operation.
    Sync,
}

/// The full v4.0.4 service bus event envelope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventEnvelope {
    /// UUIDv7 — time-ordered.
    pub event_id: Uuid,
    /// Topic — `schema.entity.event`.
    pub event_type: String,
    /// Schema version, e.g. `v1`.
    pub event_version: String,
    /// When the event occurred (not when it was published).
    pub occurred_at: DateTime<Utc>,
    /// Source service.
    pub source: Source,
    /// Actor.
    pub actor: Actor,
    /// Subject.
    pub subject: Subject,
    /// Event-specific payload. JSON object.
    pub payload: serde_json::Value,
    /// Trace.
    pub trace: Trace,
    /// Schema.org type for the subject (e.g. `Person`, `Organization`).
    pub schema_org_type: String,
}

impl EventEnvelope {
    /// Returns the Cloudflare Queue name this event routes to.
    /// Queues are bucketed by top-level event prefix per v4.0.4.
    #[must_use]
    pub fn queue_name(&self) -> &str {
        // `identity.person.created` → `identity`
        self.event_type.split_once('.').map_or(&self.event_type, |(p, _)| p)
    }
}

/// The behaviour the gateway needs from a service-bus publisher.
///
/// Implementations:
/// - [`CloudflareQueuesPublisher`] — Phase 1 production transport.
/// - `MockPublisher` (test) — captures published events for assertions.
/// - `RedpandaPublisher` (future, Phase 2/3) — Kafka-protocol transport.
///
/// Cheap to clone (wrap inner state in `Arc`).
#[async_trait]
pub trait Publisher: Send + Sync {
    /// Publish a single event. Returns once the transport has accepted it.
    /// "Acceptance" semantics depend on the transport — Cloudflare Queues
    /// returns once the message is durably enqueued.
    async fn publish(&self, envelope: &EventEnvelope) -> ServiceBusResult<()>;

    /// Publish a batch atomically. Implementations may fall back to
    /// per-message publishes if the transport does not support batching.
    async fn publish_batch(&self, envelopes: &[EventEnvelope]) -> ServiceBusResult<()> {
        for env in envelopes {
            self.publish(env).await?;
        }
        Ok(())
    }
}

/// Cloudflare Queues HTTP API publisher — Phase 1 transport.
///
/// Calls `https://api.cloudflare.com/client/v4/accounts/{account}/queues/{queue}/messages`
/// with the envelope as the JSON body.
#[derive(Clone)]
pub struct CloudflareQueuesPublisher {
    http: reqwest::Client,
    account_id: String,
    api_token: String,
    base_url: String,
}

impl CloudflareQueuesPublisher {
    /// Construct a new Cloudflare Queues publisher.
    pub fn new(
        account_id: impl Into<String>,
        api_token: impl Into<String>,
    ) -> ServiceBusResult<Self> {
        let account_id = account_id.into();
        let api_token = api_token.into();
        if account_id.is_empty() || api_token.is_empty() {
            return Err(ServiceBusError::Config(
                "account_id and api_token must be non-empty".into(),
            ));
        }
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;
        Ok(Self {
            http,
            account_id,
            api_token,
            base_url: "https://api.cloudflare.com/client/v4".to_string(),
        })
    }

    /// Override the API base URL (for tests).
    #[must_use]
    pub fn with_base_url(mut self, base: impl Into<String>) -> Self {
        self.base_url = base.into();
        self
    }
}

#[async_trait]
impl Publisher for CloudflareQueuesPublisher {
    async fn publish(&self, envelope: &EventEnvelope) -> ServiceBusResult<()> {
        let queue = envelope.queue_name();
        let url = format!(
            "{}/accounts/{}/queues/{}/messages",
            self.base_url, self.account_id, queue
        );
        let response = self
            .http
            .post(&url)
            .bearer_auth(&self.api_token)
            .json(envelope)
            .send()
            .await?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ServiceBusError::Transport(format!(
                "cloudflare queues responded {status}: {body}"
            )));
        }
        tracing::debug!(
            event_type = %envelope.event_type,
            event_id = %envelope.event_id,
            queue = %queue,
            "event published"
        );
        Ok(())
    }
}

/// Builder helpers for common identity events. The gateway uses these
/// directly rather than constructing envelopes by hand.
pub mod events {
    use super::{Actor, Causation, EventEnvelope, Source, Subject, Trace};
    use chrono::Utc;
    use serde_json::json;
    use uuid::Uuid;

    /// Build an `identity.person.created` event.
    #[must_use]
    pub fn person_created(
        person_id: Uuid,
        email: &str,
        name: &str,
        source_project: Option<&str>,
        instance: &str,
        environment: &str,
    ) -> EventEnvelope {
        EventEnvelope {
            event_id: Uuid::now_v7(),
            event_type: "identity.person.created".into(),
            event_version: "v1".into(),
            occurred_at: Utc::now(),
            source: Source {
                service: "bundu-identity".into(),
                instance: instance.into(),
                environment: environment.into(),
            },
            actor: Actor {
                person_id: Some(person_id),
                entity_id: None,
                active_role: None,
            },
            subject: Subject {
                subject_type: "identity.person".into(),
                id: person_id,
            },
            payload: json!({
                "email": email,
                "name": name,
                "source_project": source_project,
            }),
            trace: Trace {
                trace_id: Uuid::now_v7(),
                parent_event_id: None,
                causation: Causation::UserRequest,
            },
            schema_org_type: "Person".into(),
        }
    }

    /// Build an `identity.entity.created` event for a family-of-one bootstrap.
    #[must_use]
    pub fn family_entity_created(
        entity_id: Uuid,
        person_id: Uuid,
        name: &str,
        source_project: Option<&str>,
        instance: &str,
        environment: &str,
    ) -> EventEnvelope {
        EventEnvelope {
            event_id: Uuid::now_v7(),
            event_type: "identity.entity.created".into(),
            event_version: "v1".into(),
            occurred_at: Utc::now(),
            source: Source {
                service: "bundu-identity".into(),
                instance: instance.into(),
                environment: environment.into(),
            },
            actor: Actor {
                person_id: Some(person_id),
                entity_id: Some(entity_id),
                active_role: Some("head".into()),
            },
            subject: Subject {
                subject_type: "identity.entity".into(),
                id: entity_id,
            },
            payload: json!({
                "entity_type": "family",
                "additional_type": "bundu:Family",
                "name": name,
                "auto_created_for_person_id": person_id,
                "source_project": source_project,
            }),
            trace: Trace {
                trace_id: Uuid::now_v7(),
                parent_event_id: None,
                causation: Causation::UserRequest,
            },
            schema_org_type: "Organization".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_name_uses_top_level_prefix() {
        let env = events::person_created(
            Uuid::now_v7(),
            "a@b.com",
            "A",
            None,
            "test-1",
            "test",
        );
        assert_eq!(env.queue_name(), "identity");
    }

    #[test]
    fn rejects_empty_account_id() {
        let r = CloudflareQueuesPublisher::new("", "tok");
        assert!(matches!(r, Err(ServiceBusError::Config(_))));
    }

    #[test]
    fn person_created_event_shape() {
        let pid = Uuid::now_v7();
        let env = events::person_created(pid, "a@b.com", "A", Some("mukoko_news_db"), "i1", "test");
        assert_eq!(env.event_type, "identity.person.created");
        assert_eq!(env.subject.id, pid);
        assert_eq!(env.schema_org_type, "Person");
    }
}
