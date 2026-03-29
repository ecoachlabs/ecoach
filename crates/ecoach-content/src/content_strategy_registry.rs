use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentTypeStrategy {
    pub node_type: String,
    pub strategy_families: Vec<String>,
    pub drill_families: Vec<String>,
    pub failure_modes: Vec<String>,
    pub mastery_evidence: Vec<String>,
    pub review_mode: String,
    pub time_sensitivity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContentStrategyRegistry {
    pub strategies: Vec<ContentTypeStrategy>,
}

impl ContentStrategyRegistry {
    pub fn core() -> Self {
        Self {
            strategies: vec![
                strategy(
                    "definition",
                    &[
                        "boundary",
                        "contrast",
                        "example_non_example",
                        "semantic_unpacking",
                    ],
                    &["definition_recall", "example_sort", "compare_terms"],
                    &[
                        "boundary_blur",
                        "near_neighbor_confusion",
                        "verbatim_illusion",
                    ],
                    &[
                        "own_word_definition",
                        "non_example_rejection",
                        "contrast_success",
                    ],
                    "retrieval_with_contrast",
                    "timed_recall_after_stability",
                ),
                strategy(
                    "concept",
                    &[
                        "prototype_then_variation",
                        "representation_switching",
                        "misconception_exposure",
                    ],
                    &["scenario_recognition", "concept_map", "teach_back"],
                    &[
                        "single_example_lock",
                        "surface_feature_confusion",
                        "definition_without_concept",
                    ],
                    &[
                        "transfer_success",
                        "invariant_extraction",
                        "flexible_explanation",
                    ],
                    "mixed_context_review",
                    "timed_transfer_after_understanding",
                ),
                strategy(
                    "formula",
                    &[
                        "semantic_unpacking",
                        "when_to_use",
                        "application_then_variation",
                    ],
                    &["formula_recall", "variable_meaning", "substitute_and_solve"],
                    &[
                        "meaning_blindness",
                        "wrong_formula_selection",
                        "unit_confusion",
                    ],
                    &[
                        "formula_recall",
                        "correct_formula_selection",
                        "applied_accuracy",
                    ],
                    "formula_plus_application_review",
                    "timed_application_after_selection_stability",
                ),
                strategy(
                    "procedure",
                    &["stage_map", "faded_example", "decision_gate"],
                    &["step_completion", "error_repair", "guided_then_independent"],
                    &["step_skip", "order_confusion", "false_shortcut"],
                    &[
                        "independent_execution",
                        "error_detection",
                        "step_explanation",
                    ],
                    "sequenced_reactivation",
                    "timed_execution_after_clean_steps",
                ),
                strategy(
                    "comparison",
                    &["contrast", "boundary", "side_by_side_separation"],
                    &["which_one_fits", "difference_grid", "reverse_contrast"],
                    &[
                        "concept_contamination",
                        "false_similarity",
                        "one_way_difference_only",
                    ],
                    &[
                        "two_way_separation",
                        "trap_rejection",
                        "mixed_pair_discrimination",
                    ],
                    "contrast_review",
                    "timed_discrimination_after_pair_stability",
                ),
                strategy(
                    "principle",
                    &["causal_chain", "concept_to_application", "justify_back"],
                    &["principle_selection", "explain_why", "scenario_application"],
                    &[
                        "memorized_statement_without_force",
                        "wrong_scope",
                        "misapplied_principle",
                    ],
                    &[
                        "principle_explanation",
                        "correct_trigger_selection",
                        "scenario_transfer",
                    ],
                    "principle_with_application_review",
                    "timed_selection_after_calm_explanation",
                ),
                strategy(
                    "rule",
                    &["boundary", "error_exposure", "decision_gate"],
                    &["rule_trigger", "correct_incorrect_judgment", "apply_rule"],
                    &["overgeneralization", "condition_omission", "rule_swap"],
                    &["trigger_accuracy", "condition_recall", "rule_application"],
                    "rule_trigger_review",
                    "timed_trigger_check_after_condition_clarity",
                ),
                strategy(
                    "theorem",
                    &[
                        "meaning_then_use",
                        "proof_chain",
                        "representation_switching",
                    ],
                    &[
                        "theorem_selection",
                        "statement_completion",
                        "proof_skeleton",
                    ],
                    &[
                        "statement_memory_only",
                        "wrong_precondition",
                        "misapplied_theorem",
                    ],
                    &[
                        "statement_accuracy",
                        "condition_recognition",
                        "proof_or_use_success",
                    ],
                    "theorem_plus_problem_review",
                    "timed_use_after_condition_mastery",
                ),
                strategy(
                    "worked_pattern",
                    &["faded_example", "step_map", "variation_after_model"],
                    &["complete_the_steps", "spot_the_move", "pattern_transfer"],
                    &[
                        "pattern_copy_without_reason",
                        "step_loss",
                        "surface_matching_only",
                    ],
                    &[
                        "pattern_reproduction",
                        "move_explanation",
                        "variation_success",
                    ],
                    "worked_to_independent_review",
                    "timed_pattern_rebuild_after_independence",
                ),
                strategy(
                    "application",
                    &[
                        "decision_gate",
                        "representation_switching",
                        "example_variation",
                    ],
                    &[
                        "scenario_selection",
                        "real_world_application",
                        "multi_step_transfer",
                    ],
                    &["theory_without_use", "wrong_method_choice", "context_drop"],
                    &[
                        "correct_method_choice",
                        "scenario_transfer",
                        "stable multi_step use",
                    ],
                    "application_mix_review",
                    "timed_transfer_after_method_selection",
                ),
                strategy(
                    "interpretation",
                    &[
                        "signal_highlighting",
                        "representation_switching",
                        "explain_back",
                    ],
                    &["graph_reading", "table_to_claim", "evidence_selection"],
                    &["signal_miss", "label_confusion", "misread_trend"],
                    &[
                        "accurate_reading",
                        "claim_with_evidence",
                        "cross-representation interpretation",
                    ],
                    "interpretation_recheck",
                    "timed_reading_after signal stability",
                ),
                strategy(
                    "diagram_spatial",
                    &[
                        "spatial_mapping",
                        "label_then_reason",
                        "representation_switching",
                    ],
                    &[
                        "diagram_label",
                        "locate_and_explain",
                        "spatial_relationship_check",
                    ],
                    &["label_swap", "orientation_confusion", "feature_misread"],
                    &[
                        "accurate_labelling",
                        "spatial_explanation",
                        "diagram_transfer",
                    ],
                    "diagram_reactivation",
                    "timed_spatial_identification_after calm mapping",
                ),
                strategy(
                    "proof_justification",
                    &["proof_chain", "why_required", "claim_evidence_linking"],
                    &[
                        "complete_the_justification",
                        "identify_missing_reason",
                        "proof_ordering",
                    ],
                    &["assertion_without_support", "step_jump", "wrong-why"],
                    &["reasoned_chain", "valid justification", "proof stability"],
                    "justification_chain_review",
                    "timed_proof after chain accuracy",
                ),
                strategy(
                    "essay_structured",
                    &[
                        "structure_and_expression",
                        "idea_selection",
                        "timed_expression",
                    ],
                    &["outline_building", "paragraph_order", "evidence_selection"],
                    &[
                        "response_architecture_collapse",
                        "evidence_thinness",
                        "time_misallocation",
                    ],
                    &["coherent_structure", "relevant support", "timed completion"],
                    "outline_then_timed_review",
                    "timed_expression is core",
                ),
                strategy(
                    "word_problem_translation",
                    &["translation", "decision_gate", "representation_switching"],
                    &[
                        "language_to_equation",
                        "identify_what_is_asked",
                        "select_method",
                    ],
                    &["translation_failure", "context_overload", "method_mismatch"],
                    &[
                        "structured_translation",
                        "correct method choice",
                        "solved scenario",
                    ],
                    "translation_reactivation",
                    "timed_translation after calm decoding",
                ),
                strategy(
                    "vocabulary",
                    &["boundary", "context_usage", "contrast"],
                    &["term_recall", "context_fit", "audio_to_term"],
                    &["word_swap", "shallow_recognition", "context_misuse"],
                    &[
                        "term_recall",
                        "correct_context_use",
                        "confusion_pair separation",
                    ],
                    "contextual recall review",
                    "timed recall after context stability",
                ),
                strategy(
                    "symbol_notation",
                    &[
                        "semantic_unpacking",
                        "representation_switching",
                        "meaning_then_use",
                    ],
                    &[
                        "symbol_to_meaning",
                        "meaning_to_symbol",
                        "notation_in_context",
                    ],
                    &["symbol_swap", "notation_blindness", "meaningless copying"],
                    &[
                        "symbol meaning",
                        "notation production",
                        "correct use in context",
                    ],
                    "notation_reactivation",
                    "timed recognition after meaning clarity",
                ),
            ],
        }
    }

    pub fn for_node_type(&self, node_type: &str) -> Option<&ContentTypeStrategy> {
        self.strategies
            .iter()
            .find(|strategy| strategy.node_type == node_type)
    }
}

fn strategy(
    node_type: &str,
    strategy_families: &[&str],
    drill_families: &[&str],
    failure_modes: &[&str],
    mastery_evidence: &[&str],
    review_mode: &str,
    time_sensitivity: &str,
) -> ContentTypeStrategy {
    ContentTypeStrategy {
        node_type: node_type.to_string(),
        strategy_families: strategy_families
            .iter()
            .map(|value| value.to_string())
            .collect(),
        drill_families: drill_families
            .iter()
            .map(|value| value.to_string())
            .collect(),
        failure_modes: failure_modes
            .iter()
            .map(|value| value.to_string())
            .collect(),
        mastery_evidence: mastery_evidence
            .iter()
            .map(|value| value.to_string())
            .collect(),
        review_mode: review_mode.to_string(),
        time_sensitivity: time_sensitivity.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_strategy_registry_covers_core_node_types() {
        let registry = ContentStrategyRegistry::core();

        assert!(registry.for_node_type("definition").is_some());
        assert!(registry.for_node_type("formula").is_some());
        assert!(registry.for_node_type("word_problem_translation").is_some());
        assert!(registry.for_node_type("essay_structured").is_some());
    }
}
