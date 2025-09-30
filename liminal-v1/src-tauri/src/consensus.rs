use std::sync::Arc;

use blake3::hash as blake3_hash;
use serde_json::to_vec;
use tokio::sync::Mutex;

use crate::ledger::{
    ConsensusEvent, ConsensusSignal, LedgerEvent, LedgerWriter, QuorumVector, QuorumVote,
};
use crate::metrics::{MetricsCollector, QuorumMetricsUpdate};

#[derive(Clone)]
pub struct ConsensusBroker {
    ledger: Option<LedgerWriter>,
    metrics: MetricsCollector,
    default_threshold: f32,
    inflight: Arc<Mutex<()>>,
}

impl ConsensusBroker {
    pub fn new(
        ledger: Option<LedgerWriter>,
        metrics: MetricsCollector,
        default_threshold: f32,
    ) -> Self {
        Self {
            ledger,
            metrics,
            default_threshold,
            inflight: Arc::new(Mutex::new(())),
        }
    }

    pub async fn record_quorum(
        &self,
        resource_id: &str,
        mut votes: Vec<QuorumVote>,
        reason: &str,
    ) -> bool {
        let _guard = self.inflight.lock().await;
        if votes.is_empty() {
            return true;
        }
        for vote in votes.iter_mut() {
            if vote.weight <= 0.0 {
                vote.weight = 1.0;
            }
        }
        let total_weight: f32 = votes.iter().map(|vote| vote.weight).sum();
        let agree_weight: f32 = votes
            .iter()
            .filter(|vote| vote.vote)
            .map(|vote| vote.weight)
            .sum();
        let threshold = self.default_threshold.max(0.0).min(1.0);
        let achieved = if total_weight > f32::EPSILON {
            (agree_weight / total_weight) >= threshold
        } else {
            false
        };
        let vector = QuorumVector {
            resource_id: resource_id.to_string(),
            threshold,
            total_weight,
            agree_weight,
            achieved,
            reason: reason.to_string(),
            votes,
        };
        self.append_consensus_event(ConsensusEvent::Proposal(
            self.build_signal("proposal", &vector),
        ))
        .await;
        self.append_consensus_event(ConsensusEvent::Vote(self.build_signal("vote", &vector)))
            .await;
        self.append_consensus_event(ConsensusEvent::Commit(self.build_signal("commit", &vector)))
            .await;
        self.metrics.record_quorum_metrics(QuorumMetricsUpdate {
            resource_id: resource_id.to_string(),
            achieved,
            threshold,
            reason: reason.to_string(),
        });
        achieved
    }

    fn build_signal(&self, phase: &str, vector: &QuorumVector) -> ConsensusSignal {
        let digest = to_vec(vector)
            .ok()
            .map(|bytes| blake3_hash(&bytes).to_hex().to_string());
        ConsensusSignal {
            topic: format!("consensus:{}", vector.resource_id),
            phase: phase.to_string(),
            agent_id: None,
            territory_id: Some(vector.resource_id.clone()),
            quorum_threshold: Some(vector.threshold),
            payload_digest: digest,
            vector: Some(vector.clone()),
        }
    }

    async fn append_consensus_event(&self, event: ConsensusEvent) {
        if let Some(writer) = &self.ledger {
            let start = std::time::Instant::now();
            if writer
                .append_async(LedgerEvent::Consensus(event))
                .await
                .is_ok()
            {
                self.metrics.record_ledger_append(start.elapsed());
            } else {
                self.metrics.record_ledger_error();
            }
        }
    }
}

pub fn quorum_vote(agent_id: &str, weight: f32, vote: bool) -> QuorumVote {
    QuorumVote {
        agent_id: agent_id.to_string(),
        weight,
        vote,
    }
}
