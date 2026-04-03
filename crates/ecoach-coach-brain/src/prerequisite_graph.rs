use ecoach_substrate::{BasisPoints, EcoachError, EcoachResult, clamp_bp};
use rusqlite::{Connection, params};

/// Resolves topic-level prerequisites by traversing the node_edges graph.
/// Used by the journey route builder to order stations and detect blocking dependencies.
pub struct PrerequisiteGraph<'a> {
    conn: &'a Connection,
}

#[derive(Debug, Clone)]
pub struct PrerequisiteLink {
    pub from_topic_id: i64,
    pub to_topic_id: i64,
    pub strength: i64,
}

#[derive(Debug, Clone)]
pub struct ReentryProbeResult {
    pub fragile_topic_ids: Vec<i64>,
    pub at_risk_topic_ids: Vec<i64>,
    pub blocking_topic_ids: Vec<i64>,
    pub needs_reactivation: bool,
    pub probe_question_count: usize,
}

impl<'a> PrerequisiteGraph<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Get prerequisite links between topics in a subject.
    /// A topic A is prerequisite to topic B if any academic_node in A
    /// has a prerequisite edge to any academic_node in B.
    pub fn get_topic_prerequisites(&self, subject_id: i64) -> EcoachResult<Vec<PrerequisiteLink>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT DISTINCT an_from.topic_id AS from_topic, an_to.topic_id AS to_topic,
                        MAX(ne.strength_score) AS strength
                 FROM node_edges ne
                 INNER JOIN academic_nodes an_from ON an_from.id = ne.from_node_id
                 INNER JOIN academic_nodes an_to ON an_to.id = ne.to_node_id
                 INNER JOIN topics t_from ON t_from.id = an_from.topic_id AND t_from.subject_id = ?1
                 INNER JOIN topics t_to ON t_to.id = an_to.topic_id AND t_to.subject_id = ?1
                 WHERE ne.edge_type IN ('prerequisite', 'soft_prerequisite', 'depends_on')
                   AND an_from.topic_id <> an_to.topic_id
                 GROUP BY an_from.topic_id, an_to.topic_id
                 ORDER BY strength DESC",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map([subject_id], |row| {
                Ok(PrerequisiteLink {
                    from_topic_id: row.get(0)?,
                    to_topic_id: row.get(1)?,
                    strength: row.get(2)?,
                })
            })
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut links = Vec::new();
        for row in rows {
            links.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(links)
    }

    /// Order topic IDs respecting prerequisites (topological sort).
    /// Prerequisites come before dependents.
    pub fn topological_order(
        &self,
        topic_ids: &[i64],
        prerequisites: &[PrerequisiteLink],
    ) -> Vec<i64> {
        use std::collections::{BTreeMap, BTreeSet, VecDeque};

        let set: BTreeSet<i64> = topic_ids.iter().copied().collect();
        let mut in_degree: BTreeMap<i64, usize> = BTreeMap::new();
        let mut adj: BTreeMap<i64, Vec<i64>> = BTreeMap::new();

        for &tid in topic_ids {
            in_degree.entry(tid).or_insert(0);
        }

        for link in prerequisites {
            if set.contains(&link.from_topic_id) && set.contains(&link.to_topic_id) {
                adj.entry(link.from_topic_id)
                    .or_default()
                    .push(link.to_topic_id);
                *in_degree.entry(link.to_topic_id).or_insert(0) += 1;
            }
        }

        let mut queue: VecDeque<i64> = in_degree
            .iter()
            .filter(|(_, deg)| **deg == 0)
            .map(|(&tid, _)| tid)
            .collect();

        let mut result = Vec::new();
        while let Some(tid) = queue.pop_front() {
            result.push(tid);
            if let Some(neighbors) = adj.get(&tid) {
                for &next in neighbors {
                    if let Some(deg) = in_degree.get_mut(&next) {
                        *deg = deg.saturating_sub(1);
                        if *deg == 0 {
                            queue.push_back(next);
                        }
                    }
                }
            }
        }

        // Add any remaining topics not in the graph (no edges)
        for &tid in topic_ids {
            if !result.contains(&tid) {
                result.push(tid);
            }
        }

        result
    }

    /// Determine which prerequisites are fragile for a student before advancing.
    /// Used for re-entry calibration probes.
    pub fn compute_reentry_probe(
        &self,
        student_id: i64,
        subject_id: i64,
        current_topic_id: i64,
    ) -> EcoachResult<ReentryProbeResult> {
        let prerequisites = self.get_topic_prerequisites(subject_id)?;

        // Find topics that are prerequisites of the current topic
        let prereq_topic_ids: Vec<i64> = prerequisites
            .iter()
            .filter(|link| link.to_topic_id == current_topic_id)
            .map(|link| link.from_topic_id)
            .collect();

        if prereq_topic_ids.is_empty() {
            return Ok(ReentryProbeResult {
                fragile_topic_ids: Vec::new(),
                at_risk_topic_ids: Vec::new(),
                blocking_topic_ids: Vec::new(),
                needs_reactivation: false,
                probe_question_count: 0,
            });
        }

        let mut fragile = Vec::new();
        let mut at_risk = Vec::new();
        let mut blocking = Vec::new();

        for &prereq_tid in &prereq_topic_ids {
            let state: Option<(i64, i64, i64)> = self
                .conn
                .query_row(
                    "SELECT mastery_score, fragility_score, COALESCE(decay_risk, 0)
                     FROM student_topic_states
                     WHERE student_id = ?1 AND topic_id = ?2",
                    params![student_id, prereq_tid],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                )
                .ok();

            match state {
                Some((mastery, fragility, decay)) => {
                    if mastery < 3000 {
                        blocking.push(prereq_tid);
                    } else if fragility >= 6000 || decay >= 5000 {
                        at_risk.push(prereq_tid);
                    } else if fragility >= 4000 {
                        fragile.push(prereq_tid);
                    }
                }
                None => {
                    // No state = unseen prerequisite = blocking
                    blocking.push(prereq_tid);
                }
            }
        }

        let needs_reactivation = !at_risk.is_empty() || !blocking.is_empty();
        let probe_count = blocking.len() * 2 + at_risk.len() * 2 + fragile.len();

        Ok(ReentryProbeResult {
            fragile_topic_ids: fragile,
            at_risk_topic_ids: at_risk,
            blocking_topic_ids: blocking,
            needs_reactivation,
            probe_question_count: probe_count.min(7), // cap at 7
        })
    }

    /// Identify topics that were once conquered but are now at risk,
    /// suitable for inserting reactivation stations.
    pub fn find_reactivation_candidates(
        &self,
        student_id: i64,
        subject_id: i64,
    ) -> EcoachResult<Vec<i64>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT topic_id FROM student_topic_states
                 WHERE student_id = ?1
                   AND topic_id IN (SELECT id FROM topics WHERE subject_id = ?2)
                   AND mastery_score >= 5000
                   AND (decay_risk >= 5000 OR fragility_score >= 6000)
                 ORDER BY decay_risk DESC, fragility_score DESC
                 LIMIT 5",
            )
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map(params![student_id, subject_id], |row| row.get(0))
            .map_err(|e| EcoachError::Storage(e.to_string()))?;

        let mut candidates = Vec::new();
        for row in rows {
            candidates.push(row.map_err(|e| EcoachError::Storage(e.to_string()))?);
        }
        Ok(candidates)
    }
}
