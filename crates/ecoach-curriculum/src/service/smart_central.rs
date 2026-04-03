use super::*;
use crate::models::{
    CurriculumAdminNodeDetail, CurriculumCohortPin, CurriculumCohortPinInput,
    CurriculumImpactAnalysis, CurriculumImpactItem, CurriculumIngestionWorkspace,
    CurriculumNodeCitation, CurriculumNodeCitationInput, CurriculumNodeComment,
    CurriculumNodeCommentInput, CurriculumNodeExemplar, CurriculumNodeExemplarInput,
    CurriculumNodeIntelligence, CurriculumNodeIntelligenceInput, CurriculumParentSummary,
    CurriculumRecommendation, CurriculumRegenerationJob, CurriculumRegistryEntry,
    CurriculumStudentHomeSnapshot, CurriculumStudentNodeState, CurriculumStudentSubjectCard,
    CurriculumStudentSubjectMap, StudentCurriculumAssignment, StudentCurriculumAssignmentInput,
};

impl<'a> CurriculumService<'a> {
    pub fn save_curriculum_node_citation(
        &self,
        input: CurriculumNodeCitationInput,
    ) -> EcoachResult<CurriculumNodeCitation> {
        let review_status = input
            .review_status
            .unwrap_or_else(|| "pending_review".to_string());
        if let Some(id) = input.id {
            self.conn
                .execute(
                    "UPDATE curriculum_node_citations
                     SET curriculum_node_id = ?1,
                         source_upload_id = ?2,
                         citation_kind = ?3,
                         reference_code = ?4,
                         source_file_label = ?5,
                         source_page = ?6,
                         source_section = ?7,
                         source_snippet = ?8,
                         ocr_confidence_score = ?9,
                         parsing_confidence_score = ?10,
                         review_status = ?11,
                         updated_at = datetime('now')
                     WHERE id = ?12",
                    params![
                        input.curriculum_node_id,
                        input.source_upload_id,
                        input.citation_kind,
                        input.reference_code,
                        input.source_file_label,
                        input.source_page,
                        input.source_section,
                        input.source_snippet,
                        input.ocr_confidence_score.clamp(0, 10_000),
                        input.parsing_confidence_score.clamp(0, 10_000),
                        review_status,
                        id,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.get_curriculum_node_citation(id)?.ok_or_else(|| {
                EcoachError::NotFound(format!("curriculum node citation {} not found", id))
            })
        } else {
            self.conn
                .execute(
                    "INSERT INTO curriculum_node_citations (
                        curriculum_node_id, source_upload_id, citation_kind, reference_code,
                        source_file_label, source_page, source_section, source_snippet,
                        ocr_confidence_score, parsing_confidence_score, review_status
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                    params![
                        input.curriculum_node_id,
                        input.source_upload_id,
                        input.citation_kind,
                        input.reference_code,
                        input.source_file_label,
                        input.source_page,
                        input.source_section,
                        input.source_snippet,
                        input.ocr_confidence_score.clamp(0, 10_000),
                        input.parsing_confidence_score.clamp(0, 10_000),
                        review_status,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.get_curriculum_node_citation(self.conn.last_insert_rowid())?
                .ok_or_else(|| EcoachError::Storage("citation insert did not persist".to_string()))
        }
    }

    pub fn save_curriculum_node_exemplar(
        &self,
        input: CurriculumNodeExemplarInput,
    ) -> EcoachResult<CurriculumNodeExemplar> {
        let metadata_json = serialize_json(&input.metadata)?;
        let review_status = input
            .review_status
            .unwrap_or_else(|| "pending_review".to_string());
        if let Some(id) = input.id {
            self.conn
                .execute(
                    "UPDATE curriculum_node_exemplars
                     SET curriculum_node_id = ?1,
                         citation_id = ?2,
                         exemplar_kind = ?3,
                         raw_text = ?4,
                         public_text = ?5,
                         metadata_json = ?6,
                         display_order = ?7,
                         review_status = ?8,
                         updated_at = datetime('now')
                     WHERE id = ?9",
                    params![
                        input.curriculum_node_id,
                        input.citation_id,
                        input.exemplar_kind,
                        input.raw_text,
                        input.public_text,
                        metadata_json,
                        input.display_order,
                        review_status,
                        id,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.get_curriculum_node_exemplar(id)?.ok_or_else(|| {
                EcoachError::NotFound(format!("curriculum exemplar {} not found", id))
            })
        } else {
            self.conn
                .execute(
                    "INSERT INTO curriculum_node_exemplars (
                        curriculum_node_id, citation_id, exemplar_kind, raw_text, public_text,
                        metadata_json, display_order, review_status
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        input.curriculum_node_id,
                        input.citation_id,
                        input.exemplar_kind,
                        input.raw_text,
                        input.public_text,
                        metadata_json,
                        input.display_order,
                        review_status,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.get_curriculum_node_exemplar(self.conn.last_insert_rowid())?
                .ok_or_else(|| EcoachError::Storage("exemplar insert did not persist".to_string()))
        }
    }

    pub fn save_curriculum_node_comment(
        &self,
        input: CurriculumNodeCommentInput,
    ) -> EcoachResult<CurriculumNodeComment> {
        let metadata_json = serialize_json(&input.metadata)?;
        let review_status = input
            .review_status
            .unwrap_or_else(|| "pending_review".to_string());
        if let Some(id) = input.id {
            self.conn
                .execute(
                    "UPDATE curriculum_node_comments
                     SET curriculum_node_id = ?1,
                         citation_id = ?2,
                         comment_type = ?3,
                         comment_text = ?4,
                         public_text = ?5,
                         metadata_json = ?6,
                         display_order = ?7,
                         review_status = ?8,
                         updated_at = datetime('now')
                     WHERE id = ?9",
                    params![
                        input.curriculum_node_id,
                        input.citation_id,
                        input.comment_type,
                        input.comment_text,
                        input.public_text,
                        metadata_json,
                        input.display_order,
                        review_status,
                        id,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.get_curriculum_node_comment(id)?.ok_or_else(|| {
                EcoachError::NotFound(format!("curriculum comment {} not found", id))
            })
        } else {
            self.conn
                .execute(
                    "INSERT INTO curriculum_node_comments (
                        curriculum_node_id, citation_id, comment_type, comment_text, public_text,
                        metadata_json, display_order, review_status
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        input.curriculum_node_id,
                        input.citation_id,
                        input.comment_type,
                        input.comment_text,
                        input.public_text,
                        metadata_json,
                        input.display_order,
                        review_status,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.get_curriculum_node_comment(self.conn.last_insert_rowid())?
                .ok_or_else(|| EcoachError::Storage("comment insert did not persist".to_string()))
        }
    }

    pub fn upsert_curriculum_node_intelligence(
        &self,
        input: CurriculumNodeIntelligenceInput,
    ) -> EcoachResult<CurriculumNodeIntelligence> {
        self.conn
            .execute(
                "INSERT INTO curriculum_node_intelligence (
                    curriculum_node_id, friendly_topic_name, internal_subtopic_atoms_json,
                    knowledge_points_json, skills_json, cognitive_verb,
                    expected_evidence_type, instructional_mode, assessment_mode,
                    misconception_tags_json, prerequisite_node_ids_json, dependent_node_ids_json,
                    difficulty_ladder_json, teaching_strategies_json, question_families_json,
                    worked_example_templates_json, memory_tags_json, local_context_examples_json,
                    exam_mapping_json, notes_json, approval_status
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21)
                 ON CONFLICT(curriculum_node_id) DO UPDATE SET
                    friendly_topic_name = excluded.friendly_topic_name,
                    internal_subtopic_atoms_json = excluded.internal_subtopic_atoms_json,
                    knowledge_points_json = excluded.knowledge_points_json,
                    skills_json = excluded.skills_json,
                    cognitive_verb = excluded.cognitive_verb,
                    expected_evidence_type = excluded.expected_evidence_type,
                    instructional_mode = excluded.instructional_mode,
                    assessment_mode = excluded.assessment_mode,
                    misconception_tags_json = excluded.misconception_tags_json,
                    prerequisite_node_ids_json = excluded.prerequisite_node_ids_json,
                    dependent_node_ids_json = excluded.dependent_node_ids_json,
                    difficulty_ladder_json = excluded.difficulty_ladder_json,
                    teaching_strategies_json = excluded.teaching_strategies_json,
                    question_families_json = excluded.question_families_json,
                    worked_example_templates_json = excluded.worked_example_templates_json,
                    memory_tags_json = excluded.memory_tags_json,
                    local_context_examples_json = excluded.local_context_examples_json,
                    exam_mapping_json = excluded.exam_mapping_json,
                    notes_json = excluded.notes_json,
                    approval_status = excluded.approval_status,
                    updated_at = datetime('now')",
                params![
                    input.curriculum_node_id,
                    input.friendly_topic_name,
                    serialize_json(&input.internal_subtopic_atoms)?,
                    serialize_json(&input.knowledge_points)?,
                    serialize_json(&input.skills)?,
                    input.cognitive_verb,
                    input.expected_evidence_type,
                    input.instructional_mode,
                    input.assessment_mode,
                    serialize_json(&input.misconception_tags)?,
                    serialize_json(&input.prerequisite_node_ids)?,
                    serialize_json(&input.dependent_node_ids)?,
                    serialize_json(&input.difficulty_ladder)?,
                    serialize_json(&input.teaching_strategies)?,
                    serialize_json(&input.question_families)?,
                    serialize_json(&input.worked_example_templates)?,
                    serialize_json(&input.memory_tags)?,
                    serialize_json(&input.local_context_examples)?,
                    serialize_json(&input.exam_mapping)?,
                    serialize_json(&input.notes)?,
                    input.approval_status.unwrap_or_else(|| "draft".to_string()),
                ],
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        self.get_curriculum_node_intelligence(input.curriculum_node_id)?
            .ok_or_else(|| EcoachError::Storage("intelligence upsert did not persist".to_string()))
    }

    pub fn get_curriculum_registry(&self) -> EcoachResult<Vec<CurriculumRegistryEntry>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT version.id, version.curriculum_family_id, version.name, version.country,
                        version.exam_board, version.education_stage, version.version_label,
                        version.status, version.effective_from, version.effective_to,
                        version.source_summary_json, version.published_at, version.replaced_by_version_id,
                        family.id, family.slug, family.name, family.country_code, family.exam_board,
                        family.education_stage, family.description, family.is_public
                 FROM curriculum_versions version
                 INNER JOIN curriculum_families family ON family.id = version.curriculum_family_id
                 ORDER BY CASE WHEN version.status = 'published' THEN 0 ELSE 1 END,
                          COALESCE(version.published_at, version.updated_at) DESC,
                          version.id DESC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    map_curriculum_version(row)?,
                    CurriculumFamily {
                        id: row.get(13)?,
                        slug: row.get(14)?,
                        name: row.get(15)?,
                        country_code: row.get(16)?,
                        exam_board: row.get(17)?,
                        education_stage: row.get(18)?,
                        description: row.get(19)?,
                        is_public: row.get(20)?,
                    },
                ))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut items = Vec::new();
        for row in rows {
            let (version, family) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            let subject_count = self.count_subject_tracks_for_version(version.id)?;
            let node_count = self.count_nodes_for_version(version.id)?;
            let pending_review_tasks = self.count_pending_review_tasks_for_version(&version)?;
            let low_confidence_nodes = self.count_low_confidence_nodes(version.id)?;
            let current_cohorts = self
                .list_curriculum_version_cohort_pins(version.id)?
                .into_iter()
                .filter(|item| item.rollout_status != "retired")
                .map(|item| item.cohort_label)
                .collect::<Vec<_>>();
            let latest_source = self.latest_source_for_version(&version)?;
            let workflow_state = derive_curriculum_workflow_state(
                &version.status,
                version.replaced_by_version_id.is_some(),
                subject_count,
                node_count,
                pending_review_tasks,
                low_confidence_nodes,
            )
            .to_string();
            items.push(CurriculumRegistryEntry {
                family,
                version,
                subject_count,
                node_count,
                pending_review_tasks,
                low_confidence_nodes,
                current_cohorts,
                latest_source_title: latest_source.as_ref().map(|item| item.title.clone()),
                latest_source_status: latest_source
                    .as_ref()
                    .map(|item| item.source_status.clone()),
                workflow_state,
                has_source_file: latest_source
                    .as_ref()
                    .and_then(|item| item.source_path.as_ref())
                    .is_some(),
            });
        }
        Ok(items)
    }

    pub fn get_curriculum_ingestion_workspace(
        &self,
        source_upload_id: i64,
    ) -> EcoachResult<Option<CurriculumIngestionWorkspace>> {
        let Some(report) = self.get_source_report(source_upload_id)? else {
            return Ok(None);
        };
        let low_confidence_count = report
            .candidates
            .iter()
            .filter(|item| item.confidence_score < 6_500)
            .count() as i64;
        let duplicate_warning_count = report
            .review_tasks
            .iter()
            .filter(|item| item.task_type == "duplicate_check")
            .count() as i64;
        let unresolved_count = report
            .review_tasks
            .iter()
            .filter(|item| item.status != "resolved")
            .count() as i64;
        let parsed_clean_count = report
            .candidates
            .iter()
            .filter(|item| item.review_status == "approved")
            .count() as i64;
        Ok(Some(CurriculumIngestionWorkspace {
            source_upload: report.source_upload.clone(),
            parse_candidates: report.candidates.clone(),
            review_tasks: report.review_tasks.clone(),
            low_confidence_count,
            duplicate_warning_count,
            unresolved_count,
            parsed_clean_count,
            extraction_summary: json!({
                "candidate_count": report.candidates.len(),
                "review_task_count": report.review_tasks.len(),
                "status": report.source_upload.source_status,
                "source_kind": report.source_upload.source_kind,
                "metadata": report.source_upload.metadata,
            }),
        }))
    }
    pub fn get_curriculum_admin_node_detail(
        &self,
        node_id: i64,
    ) -> EcoachResult<CurriculumAdminNodeDetail> {
        let bundle = self.get_curriculum_node_bundle(node_id)?.ok_or_else(|| {
            EcoachError::NotFound(format!("curriculum node {} not found", node_id))
        })?;
        Ok(CurriculumAdminNodeDetail {
            citations: self.list_curriculum_node_citations(node_id)?,
            exemplars: self.list_curriculum_node_exemplars(node_id)?,
            comments: self.list_curriculum_node_comments(node_id)?,
            intelligence: self.get_curriculum_node_intelligence(node_id)?,
            linked_resource_counts: self.count_linked_resource_buckets(node_id)?,
            learner_signal_summary: self.build_learner_signal_summary(&bundle.node)?,
            bundle,
        })
    }

    pub fn analyze_curriculum_version_impact(
        &self,
        base_version_id: i64,
        compare_version_id: i64,
    ) -> EcoachResult<CurriculumImpactAnalysis> {
        let diff = self.get_curriculum_version_diff(base_version_id, compare_version_id)?;
        let base_nodes = map_nodes_by_slug(&self.list_nodes_for_version(base_version_id)?);
        let compare_nodes = map_nodes_by_slug(&self.list_nodes_for_version(compare_version_id)?);
        let mut items = Vec::new();
        let mut affected_lessons = 0_i64;
        let mut affected_questions = 0_i64;
        let mut affected_drills = 0_i64;
        let mut affected_diagnostics = 0_i64;
        let mut affected_learners = 0_i64;
        let mut stale_content_count = 0_i64;
        let mut safe_to_update_count = 0_i64;
        let mut requires_review_count = 0_i64;
        let mut requires_regeneration_count = 0_i64;
        let mut requires_staging_count = 0_i64;

        for entry in diff.entries {
            let affected_node = compare_nodes
                .get(&entry.entity_key)
                .cloned()
                .or_else(|| base_nodes.get(&entry.entity_key).cloned());
            let (question_count, lesson_count, drill_count, diagnostic_count) =
                if let Some(node) = affected_node.as_ref() {
                    let counts = self.count_linked_resource_buckets(node.id)?;
                    (
                        counts.get("question").and_then(Value::as_i64).unwrap_or(0),
                        counts.get("lesson").and_then(Value::as_i64).unwrap_or(0),
                        counts.get("drill").and_then(Value::as_i64).unwrap_or(0),
                        counts
                            .get("diagnostic")
                            .and_then(Value::as_i64)
                            .unwrap_or(0),
                    )
                } else {
                    (0, 0, 0, 0)
                };
            let learner_count = if let Some(topic_id) =
                affected_node.as_ref().and_then(|item| item.legacy_topic_id)
            {
                self.conn
                    .query_row(
                        "SELECT COUNT(DISTINCT student_id)
                         FROM curriculum_coverage_ledger
                         WHERE topic_id = ?1",
                        [topic_id],
                        |row| row.get::<_, i64>(0),
                    )
                    .map_err(|err| EcoachError::Storage(err.to_string()))?
            } else {
                0
            };
            let severity = impact_severity(&entry.diff_type).to_string();
            let action_required = impact_action(&entry.diff_type).to_string();
            match action_required.as_str() {
                "refresh_metadata" => safe_to_update_count += 1,
                "review" => requires_review_count += 1,
                "regenerate" | "deprecate" => requires_regeneration_count += 1,
                _ => requires_staging_count += 1,
            }
            if action_required != "refresh_metadata"
                && (question_count + lesson_count + drill_count + diagnostic_count) > 0
            {
                stale_content_count += 1;
            }
            affected_lessons += lesson_count;
            affected_questions += question_count;
            affected_drills += drill_count;
            affected_diagnostics += diagnostic_count;
            affected_learners += learner_count;
            items.push(CurriculumImpactItem {
                entity_type: entry.entity_type,
                entity_key: entry.entity_key,
                diff_type: entry.diff_type,
                severity,
                action_required,
                affected_node_id: affected_node.as_ref().map(|node| node.id),
                affected_question_count: question_count,
                affected_lesson_count: lesson_count,
                affected_drill_count: drill_count,
                affected_diagnostic_count: diagnostic_count,
                affected_learner_count: learner_count,
                summary: entry.summary,
            });
        }

        let mut affected_cohorts = self
            .list_curriculum_version_cohort_pins(compare_version_id)?
            .into_iter()
            .map(|item| item.cohort_label)
            .collect::<Vec<_>>();
        for label in self
            .list_curriculum_version_cohort_pins(base_version_id)?
            .into_iter()
            .map(|item| item.cohort_label)
        {
            if !affected_cohorts.contains(&label) {
                affected_cohorts.push(label);
            }
        }

        Ok(CurriculumImpactAnalysis {
            base_version_id,
            compare_version_id,
            items,
            affected_lessons,
            affected_questions,
            affected_drills,
            affected_diagnostics,
            stale_content_count,
            affected_learners,
            affected_cohorts,
            safe_to_update_count,
            requires_review_count,
            requires_regeneration_count,
            requires_staging_count,
        })
    }

    pub fn stage_curriculum_regeneration_jobs(
        &self,
        base_version_id: i64,
        compare_version_id: i64,
        triggered_by_account_id: Option<i64>,
        max_jobs: i64,
    ) -> EcoachResult<Vec<CurriculumRegenerationJob>> {
        let analysis =
            self.analyze_curriculum_version_impact(base_version_id, compare_version_id)?;
        let limit = max_jobs.max(1) as usize;
        let mut created = Vec::new();
        for item in analysis
            .items
            .into_iter()
            .filter(|item| item.action_required != "refresh_metadata")
            .take(limit)
        {
            self.conn
                .execute(
                    "INSERT INTO curriculum_regeneration_jobs (
                        base_version_id, compare_version_id, affected_node_id, entity_type,
                        entity_key, severity, action_required, resource_type, resource_count,
                        impact_summary, payload_json, status, triggered_by_account_id
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                    params![
                        base_version_id,
                        compare_version_id,
                        item.affected_node_id,
                        item.entity_type,
                        item.entity_key,
                        item.severity,
                        item.action_required,
                        if item.affected_question_count > 0 {
                            "question"
                        } else if item.affected_lesson_count > 0 {
                            "lesson"
                        } else if item.affected_drill_count > 0 {
                            "drill"
                        } else if item.affected_diagnostic_count > 0 {
                            "diagnostic"
                        } else {
                            "mixed"
                        },
                        item.affected_question_count
                            + item.affected_lesson_count
                            + item.affected_drill_count
                            + item.affected_diagnostic_count,
                        item.summary,
                        serialize_json(&json!({
                            "affected_learner_count": item.affected_learner_count,
                            "question_count": item.affected_question_count,
                            "lesson_count": item.affected_lesson_count,
                            "drill_count": item.affected_drill_count,
                            "diagnostic_count": item.affected_diagnostic_count,
                            "action_required": item.action_required,
                        }))?,
                        if item.action_required == "regenerate" {
                            "ready"
                        } else {
                            "queued"
                        },
                        triggered_by_account_id,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            created.push(
                self.get_curriculum_regeneration_job(self.conn.last_insert_rowid())?
                    .ok_or_else(|| {
                        EcoachError::Storage("regeneration job did not persist".to_string())
                    })?,
            );
        }
        Ok(created)
    }

    pub fn list_curriculum_regeneration_jobs(
        &self,
        compare_version_id: i64,
        status: Option<&str>,
        limit: i64,
    ) -> EcoachResult<Vec<CurriculumRegenerationJob>> {
        let sql = if status.is_some() {
            "SELECT id, base_version_id, compare_version_id, affected_node_id, entity_type,
                    entity_key, severity, action_required, resource_type, resource_count,
                    impact_summary, payload_json, status, triggered_by_account_id
             FROM curriculum_regeneration_jobs
             WHERE compare_version_id = ?1 AND status = ?2
             ORDER BY created_at DESC, id DESC
             LIMIT ?3"
        } else {
            "SELECT id, base_version_id, compare_version_id, affected_node_id, entity_type,
                    entity_key, severity, action_required, resource_type, resource_count,
                    impact_summary, payload_json, status, triggered_by_account_id
             FROM curriculum_regeneration_jobs
             WHERE compare_version_id = ?1
             ORDER BY created_at DESC, id DESC
             LIMIT ?2"
        };
        let mut statement = self
            .conn
            .prepare(sql)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = if let Some(status) = status {
            statement.query_map(
                params![compare_version_id, status, limit.max(1)],
                map_curriculum_regeneration_job,
            )
        } else {
            statement.query_map(
                params![compare_version_id, limit.max(1)],
                map_curriculum_regeneration_job,
            )
        }
        .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    pub fn pin_curriculum_version_to_cohort(
        &self,
        input: CurriculumCohortPinInput,
    ) -> EcoachResult<CurriculumCohortPin> {
        let rollout_status = input.rollout_status.unwrap_or_else(|| "active".to_string());
        if let Some(id) = input.id {
            self.conn
                .execute(
                    "UPDATE curriculum_cohort_pins
                     SET curriculum_version_id = ?1,
                         cohort_key = ?2,
                         cohort_label = ?3,
                         level_code = ?4,
                         effective_from = ?5,
                         effective_to = ?6,
                         rollout_status = ?7,
                         pinned_by_account_id = ?8,
                         notes = ?9,
                         updated_at = datetime('now')
                     WHERE id = ?10",
                    params![
                        input.curriculum_version_id,
                        input.cohort_key,
                        input.cohort_label,
                        input.level_code,
                        input.effective_from,
                        input.effective_to,
                        rollout_status,
                        input.pinned_by_account_id,
                        input.notes,
                        id,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.get_curriculum_cohort_pin(id)?.ok_or_else(|| {
                EcoachError::NotFound(format!("curriculum cohort pin {} not found", id))
            })
        } else {
            self.conn
                .execute(
                    "INSERT INTO curriculum_cohort_pins (
                        curriculum_version_id, cohort_key, cohort_label, level_code,
                        effective_from, effective_to, rollout_status, pinned_by_account_id, notes
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                    params![
                        input.curriculum_version_id,
                        input.cohort_key,
                        input.cohort_label,
                        input.level_code,
                        input.effective_from,
                        input.effective_to,
                        rollout_status,
                        input.pinned_by_account_id,
                        input.notes,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.get_curriculum_cohort_pin(self.conn.last_insert_rowid())?
                .ok_or_else(|| {
                    EcoachError::Storage("cohort pin insert did not persist".to_string())
                })
        }
    }

    pub fn list_curriculum_version_cohort_pins(
        &self,
        curriculum_version_id: i64,
    ) -> EcoachResult<Vec<CurriculumCohortPin>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, curriculum_version_id, cohort_key, cohort_label, level_code,
                        effective_from, effective_to, rollout_status, pinned_by_account_id, notes
                 FROM curriculum_cohort_pins
                 WHERE curriculum_version_id = ?1
                 ORDER BY COALESCE(effective_from, created_at) DESC, id DESC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([curriculum_version_id], map_curriculum_cohort_pin)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    pub fn assign_student_curriculum_version(
        &self,
        input: StudentCurriculumAssignmentInput,
    ) -> EcoachResult<StudentCurriculumAssignment> {
        let assignment_source = input.assignment_source.unwrap_or_else(|| {
            if input.cohort_pin_id.is_some() {
                "cohort".to_string()
            } else {
                "manual".to_string()
            }
        });
        let status = input.status.unwrap_or_else(|| "active".to_string());
        if status == "active" {
            self.conn
                .execute(
                    "UPDATE student_curriculum_assignments
                     SET status = 'superseded', updated_at = datetime('now')
                     WHERE student_id = ?1 AND status = 'active' AND (?2 IS NULL OR id != ?2)",
                    params![input.student_id, input.id],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
        }
        if let Some(id) = input.id {
            self.conn
                .execute(
                    "UPDATE student_curriculum_assignments
                     SET student_id = ?1,
                         curriculum_version_id = ?2,
                         cohort_pin_id = ?3,
                         assignment_source = ?4,
                         status = ?5,
                         notes = ?6,
                         updated_at = datetime('now')
                     WHERE id = ?7",
                    params![
                        input.student_id,
                        input.curriculum_version_id,
                        input.cohort_pin_id,
                        assignment_source,
                        status,
                        input.notes,
                        id,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.get_student_curriculum_assignment(id)?.ok_or_else(|| {
                EcoachError::NotFound(format!("student curriculum assignment {} not found", id))
            })
        } else {
            self.conn
                .execute(
                    "INSERT INTO student_curriculum_assignments (
                        student_id, curriculum_version_id, cohort_pin_id,
                        assignment_source, status, notes
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        input.student_id,
                        input.curriculum_version_id,
                        input.cohort_pin_id,
                        assignment_source,
                        status,
                        input.notes,
                    ],
                )
                .map_err(|err| EcoachError::Storage(err.to_string()))?;
            self.get_student_curriculum_assignment(self.conn.last_insert_rowid())?
                .ok_or_else(|| {
                    EcoachError::Storage("student assignment insert did not persist".to_string())
                })
        }
    }

    pub fn get_active_student_curriculum_assignment(
        &self,
        student_id: i64,
    ) -> EcoachResult<Option<StudentCurriculumAssignment>> {
        self.conn
            .query_row(
                "SELECT id, student_id, curriculum_version_id, cohort_pin_id, assignment_source,
                        status, notes, assigned_at
                 FROM student_curriculum_assignments
                 WHERE student_id = ?1 AND status = 'active'
                 ORDER BY assigned_at DESC, id DESC
                 LIMIT 1",
                [student_id],
                map_student_curriculum_assignment,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    pub fn get_student_curriculum_home(
        &self,
        student_id: i64,
        curriculum_version_id: Option<i64>,
    ) -> EcoachResult<CurriculumStudentHomeSnapshot> {
        let version = self.resolve_active_curriculum_version(student_id, curriculum_version_id)?;
        let subject_tracks = self.list_subject_tracks_for_version(version.id)?;
        let coverage = self.load_coverage_map(student_id)?;
        let memory = self.load_memory_states(student_id)?;
        let recent_movements = self.list_recent_movements(student_id, version.id)?;
        let mut subject_cards = Vec::new();
        for track in subject_tracks {
            subject_cards
                .push(self.build_student_subject_card(student_id, &track, &coverage, &memory)?);
        }
        let subject_count = subject_cards.len().max(1) as i64;
        let entered_percent = subject_cards
            .iter()
            .map(|item| item.entered_percent)
            .sum::<i64>()
            / subject_count;
        let stable_percent = subject_cards
            .iter()
            .map(|item| item.stable_percent)
            .sum::<i64>()
            / subject_count;
        let exam_readiness_percent = subject_cards
            .iter()
            .map(|item| item.exam_ready_percent)
            .sum::<i64>()
            / subject_count;
        let weak_topics_count = subject_cards.iter().map(|item| item.weak_area_count).sum();
        let blocked_topics_count = subject_cards.iter().map(|item| item.blocked_count).sum();
        let review_due_count = subject_cards.iter().map(|item| item.review_due_count).sum();
        let strongest_subject = subject_cards
            .iter()
            .max_by_key(|item| item.stable_percent)
            .map(|item| item.public_title.clone())
            .unwrap_or_else(|| "your subjects".to_string());
        let highest_risk_subject = subject_cards
            .iter()
            .max_by_key(|item| item.weak_area_count + item.blocked_count)
            .map(|item| item.public_title.clone())
            .unwrap_or_else(|| "the curriculum".to_string());
        let position_statement = format!(
            "Your strongest current curriculum progress is in {}. The biggest present risk is in {}, where weak or blocked topics are holding back the next layer of work.",
            strongest_subject, highest_risk_subject
        );
        let mut recommended_topics = Vec::<CurriculumRecommendation>::new();
        for track in self.list_subject_tracks_for_version(version.id)? {
            if let Some(legacy_subject_id) = track.legacy_subject_id {
                for item in
                    self.get_curriculum_next_best_topics(student_id, legacy_subject_id, 2)?
                {
                    if !recommended_topics
                        .iter()
                        .any(|existing| existing.node_id == item.node_id)
                    {
                        recommended_topics.push(item);
                    }
                }
            }
        }
        recommended_topics.sort_by(|left, right| right.priority_score.cmp(&left.priority_score));
        recommended_topics.truncate(5);
        Ok(CurriculumStudentHomeSnapshot {
            student_id,
            curriculum_version: version,
            subject_cards,
            entered_percent,
            stable_percent,
            exam_readiness_percent,
            weak_topics_count,
            blocked_topics_count,
            review_due_count,
            position_statement,
            recent_movements,
            recommended_topics,
        })
    }

    pub fn get_student_subject_curriculum_map(
        &self,
        student_id: i64,
        subject_track_id: i64,
    ) -> EcoachResult<CurriculumStudentSubjectMap> {
        let subject = self
            .get_curriculum_subject_track(subject_track_id)?
            .ok_or_else(|| {
                EcoachError::NotFound(format!(
                    "curriculum subject track {} not found",
                    subject_track_id
                ))
            })?;
        let coverage = self.load_coverage_map(student_id)?;
        let memory = self.load_memory_states(student_id)?;
        let overview = self.build_student_subject_card(student_id, &subject, &coverage, &memory)?;
        let mut nodes = self
            .list_nodes_for_subject_track(subject_track_id)?
            .into_iter()
            .map(|node| self.build_student_node_state(student_id, &coverage, &memory, node))
            .collect::<EcoachResult<Vec<_>>>()?;
        nodes.sort_by(|left, right| {
            left.node
                .sequence_no
                .cmp(&right.node.sequence_no)
                .then(left.node.public_title.cmp(&right.node.public_title))
        });
        let recommended_topics = if let Some(legacy_subject_id) = subject.legacy_subject_id {
            self.get_curriculum_next_best_topics(student_id, legacy_subject_id, 5)?
        } else {
            Vec::new()
        };
        Ok(CurriculumStudentSubjectMap {
            student_id,
            subject,
            overview,
            nodes,
            recommended_topics,
        })
    }

    pub fn get_parent_curriculum_summary(
        &self,
        parent_id: i64,
        learner_id: i64,
        curriculum_version_id: Option<i64>,
    ) -> EcoachResult<CurriculumParentSummary> {
        let linked: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*)
                 FROM parent_student_links
                 WHERE parent_account_id = ?1 AND student_account_id = ?2",
                params![parent_id, learner_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        if linked == 0 {
            return Err(EcoachError::Validation(format!(
                "parent {} is not linked to learner {}",
                parent_id, learner_id
            )));
        }
        let home = self.get_student_curriculum_home(learner_id, curriculum_version_id)?;
        let version = home.curriculum_version.clone();
        let mut weak_topics = Vec::new();
        let mut overdue_topics = Vec::new();
        let mut exam_risk = serde_json::Map::new();
        for track in self.list_subject_tracks_for_version(version.id)? {
            let map = self.get_student_subject_curriculum_map(learner_id, track.id)?;
            for item in map
                .nodes
                .iter()
                .filter(|item| {
                    matches!(
                        item.status_label.as_str(),
                        "Fragile" | "Slipping" | "Blocked"
                    )
                })
                .take(3)
            {
                weak_topics.push(item.node.public_title.clone());
            }
            for item in map.nodes.iter().filter(|item| item.review_due).take(2) {
                overdue_topics.push(item.node.public_title.clone());
            }
            let risk_label = if map.overview.blocked_count > 0 || map.overview.weak_area_count >= 4
            {
                "high"
            } else if map.overview.weak_area_count >= 2 || map.overview.review_due_count >= 2 {
                "medium"
            } else {
                "low"
            };
            exam_risk.insert(track.public_title.clone(), Value::from(risk_label));
        }
        weak_topics.truncate(6);
        overdue_topics.truncate(6);
        let on_track = home.exam_readiness_percent >= 55 && home.blocked_topics_count == 0;
        let summary_text = if on_track {
            "The learner is broadly on track in the current curriculum, with the main focus now on keeping review-due and fragile topics from slipping back.".to_string()
        } else {
            "The learner is not fully on track yet. The biggest risks are concentrated in fragile, blocked, or overdue topics that need guided repair before the next exam push.".to_string()
        };
        Ok(CurriculumParentSummary {
            parent_id,
            learner_id,
            curriculum_version: version,
            subject_cards: home.subject_cards,
            on_track,
            weak_topics,
            overdue_topics,
            exam_risk_by_subject: Value::Object(exam_risk),
            summary_text,
        })
    }

    fn get_curriculum_node_citation(
        &self,
        id: i64,
    ) -> EcoachResult<Option<CurriculumNodeCitation>> {
        self.conn
            .query_row(
                "SELECT id, curriculum_node_id, source_upload_id, citation_kind, reference_code,
                        source_file_label, source_page, source_section, source_snippet,
                        ocr_confidence_score, parsing_confidence_score, review_status
                 FROM curriculum_node_citations
                 WHERE id = ?1",
                [id],
                map_curriculum_node_citation,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn list_curriculum_node_citations(
        &self,
        curriculum_node_id: i64,
    ) -> EcoachResult<Vec<CurriculumNodeCitation>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, curriculum_node_id, source_upload_id, citation_kind, reference_code,
                        source_file_label, source_page, source_section, source_snippet,
                        ocr_confidence_score, parsing_confidence_score, review_status
                 FROM curriculum_node_citations
                 WHERE curriculum_node_id = ?1
                 ORDER BY COALESCE(source_page, 0) ASC, id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([curriculum_node_id], map_curriculum_node_citation)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    fn get_curriculum_node_exemplar(
        &self,
        id: i64,
    ) -> EcoachResult<Option<CurriculumNodeExemplar>> {
        self.conn
            .query_row(
                "SELECT id, curriculum_node_id, citation_id, exemplar_kind, raw_text,
                        public_text, metadata_json, display_order, review_status
                 FROM curriculum_node_exemplars
                 WHERE id = ?1",
                [id],
                map_curriculum_node_exemplar,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn list_curriculum_node_exemplars(
        &self,
        curriculum_node_id: i64,
    ) -> EcoachResult<Vec<CurriculumNodeExemplar>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, curriculum_node_id, citation_id, exemplar_kind, raw_text,
                        public_text, metadata_json, display_order, review_status
                 FROM curriculum_node_exemplars
                 WHERE curriculum_node_id = ?1
                 ORDER BY display_order ASC, id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([curriculum_node_id], map_curriculum_node_exemplar)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    fn get_curriculum_node_comment(&self, id: i64) -> EcoachResult<Option<CurriculumNodeComment>> {
        self.conn
            .query_row(
                "SELECT id, curriculum_node_id, citation_id, comment_type, comment_text,
                        public_text, metadata_json, display_order, review_status
                 FROM curriculum_node_comments
                 WHERE id = ?1",
                [id],
                map_curriculum_node_comment,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn list_curriculum_node_comments(
        &self,
        curriculum_node_id: i64,
    ) -> EcoachResult<Vec<CurriculumNodeComment>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT id, curriculum_node_id, citation_id, comment_type, comment_text,
                        public_text, metadata_json, display_order, review_status
                 FROM curriculum_node_comments
                 WHERE curriculum_node_id = ?1
                 ORDER BY display_order ASC, id ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([curriculum_node_id], map_curriculum_node_comment)
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        collect_rows(rows)
    }

    fn get_curriculum_node_intelligence(
        &self,
        curriculum_node_id: i64,
    ) -> EcoachResult<Option<CurriculumNodeIntelligence>> {
        self.conn
            .query_row(
                "SELECT id, curriculum_node_id, friendly_topic_name, internal_subtopic_atoms_json,
                        knowledge_points_json, skills_json, cognitive_verb, expected_evidence_type,
                        instructional_mode, assessment_mode, misconception_tags_json,
                        prerequisite_node_ids_json, dependent_node_ids_json, difficulty_ladder_json,
                        teaching_strategies_json, question_families_json,
                        worked_example_templates_json, memory_tags_json,
                        local_context_examples_json, exam_mapping_json, notes_json, approval_status
                 FROM curriculum_node_intelligence
                 WHERE curriculum_node_id = ?1",
                [curriculum_node_id],
                map_curriculum_node_intelligence,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn latest_source_for_version(
        &self,
        version: &CurriculumVersion,
    ) -> EcoachResult<Option<CurriculumSourceUpload>> {
        self.conn
            .query_row(
                "SELECT id, uploader_account_id, source_kind, title, source_path, country_code,
                        exam_board, education_level, subject_code, academic_year,
                        language_code, version_label, source_status, confidence_score, metadata_json
                 FROM curriculum_source_uploads
                 WHERE version_label = ?1 OR academic_year = ?1
                 ORDER BY created_at DESC, id DESC
                 LIMIT 1",
                [version.version_label.as_str()],
                map_source_upload,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn count_subject_tracks_for_version(&self, version_id: i64) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM curriculum_subject_tracks WHERE curriculum_version_id = ?1",
                [version_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn count_nodes_for_version(&self, version_id: i64) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM curriculum_nodes WHERE curriculum_version_id = ?1",
                [version_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn count_pending_review_tasks_for_version(
        &self,
        version: &CurriculumVersion,
    ) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*)
                 FROM curriculum_review_tasks task
                 INNER JOIN curriculum_source_uploads source ON source.id = task.source_upload_id
                 WHERE task.status != 'resolved'
                   AND (source.version_label = ?1 OR source.academic_year = ?1)",
                [version.version_label.as_str()],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn count_low_confidence_nodes(&self, version_id: i64) -> EcoachResult<i64> {
        self.conn
            .query_row(
                "SELECT COUNT(*)
                 FROM curriculum_nodes
                 WHERE curriculum_version_id = ?1
                   AND (confidence_score < 6500 OR review_status = 'pending_review')",
                [version_id],
                |row| row.get(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn count_linked_resource_buckets(&self, node_id: i64) -> EcoachResult<Value> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT resource_type, COUNT(*)
                 FROM curriculum_resource_links
                 WHERE entity_type = 'node' AND entity_id = ?1
                 GROUP BY resource_type",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([node_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut payload = serde_json::Map::new();
        for row in rows {
            let (resource_type, count) =
                row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            payload.insert(resource_type, Value::from(count));
        }
        Ok(Value::Object(payload))
    }

    fn build_learner_signal_summary(&self, node: &CurriculumNode) -> EcoachResult<Value> {
        let Some(topic_id) = node.legacy_topic_id else {
            return Ok(json!({
                "coverage_status_counts": {},
                "recent_accuracy_bp": null,
                "linked_students": 0,
            }));
        };
        let mut coverage_statement = self
            .conn
            .prepare(
                "SELECT coverage_status, COUNT(*)
                 FROM curriculum_coverage_ledger
                 WHERE topic_id = ?1
                 GROUP BY coverage_status",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let coverage_rows = coverage_statement
            .query_map([topic_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut coverage_payload = serde_json::Map::new();
        for row in coverage_rows {
            let (status, count) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            coverage_payload.insert(status, Value::from(count));
        }
        let linked_students = self
            .conn
            .query_row(
                "SELECT COUNT(DISTINCT student_id) FROM curriculum_coverage_ledger WHERE topic_id = ?1",
                [topic_id],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let recent_accuracy_bp = self
            .conn
            .query_row(
                "SELECT CASE
                        WHEN COUNT(*) = 0 THEN NULL
                        ELSE CAST(ROUND(AVG(CASE WHEN is_correct = 1 THEN 10000.0 ELSE 0 END)) AS INTEGER)
                    END
                 FROM student_question_attempts attempt
                 INNER JOIN questions question ON question.id = attempt.question_id
                 WHERE question.topic_id = ?1",
                [topic_id],
                |row| row.get::<_, Option<i64>>(0),
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        Ok(json!({
            "coverage_status_counts": coverage_payload,
            "recent_accuracy_bp": recent_accuracy_bp,
            "linked_students": linked_students,
        }))
    }

    fn get_curriculum_regeneration_job(
        &self,
        id: i64,
    ) -> EcoachResult<Option<CurriculumRegenerationJob>> {
        self.conn
            .query_row(
                "SELECT id, base_version_id, compare_version_id, affected_node_id, entity_type,
                        entity_key, severity, action_required, resource_type, resource_count,
                        impact_summary, payload_json, status, triggered_by_account_id
                 FROM curriculum_regeneration_jobs
                 WHERE id = ?1",
                [id],
                map_curriculum_regeneration_job,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn get_curriculum_cohort_pin(&self, id: i64) -> EcoachResult<Option<CurriculumCohortPin>> {
        self.conn
            .query_row(
                "SELECT id, curriculum_version_id, cohort_key, cohort_label, level_code,
                        effective_from, effective_to, rollout_status, pinned_by_account_id, notes
                 FROM curriculum_cohort_pins
                 WHERE id = ?1",
                [id],
                map_curriculum_cohort_pin,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn get_student_curriculum_assignment(
        &self,
        id: i64,
    ) -> EcoachResult<Option<StudentCurriculumAssignment>> {
        self.conn
            .query_row(
                "SELECT id, student_id, curriculum_version_id, cohort_pin_id, assignment_source,
                        status, notes, assigned_at
                 FROM student_curriculum_assignments
                 WHERE id = ?1",
                [id],
                map_student_curriculum_assignment,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn resolve_active_curriculum_version(
        &self,
        student_id: i64,
        curriculum_version_id: Option<i64>,
    ) -> EcoachResult<CurriculumVersion> {
        if let Some(version_id) = curriculum_version_id {
            return self.get_curriculum_version(version_id)?.ok_or_else(|| {
                EcoachError::NotFound(format!("curriculum version {} not found", version_id))
            });
        }
        if let Some(assignment) = self.get_active_student_curriculum_assignment(student_id)? {
            return self
                .get_curriculum_version(assignment.curriculum_version_id)?
                .ok_or_else(|| {
                    EcoachError::NotFound(format!(
                        "assigned curriculum version {} not found",
                        assignment.curriculum_version_id
                    ))
                });
        }
        self.latest_public_curriculum_version()?.ok_or_else(|| {
            EcoachError::NotFound("no published curriculum version is available".to_string())
        })
    }

    fn latest_public_curriculum_version(&self) -> EcoachResult<Option<CurriculumVersion>> {
        self.conn
            .query_row(
                "SELECT version.id, version.curriculum_family_id, version.name, version.country,
                        version.exam_board, version.education_stage, version.version_label,
                        version.status, version.effective_from, version.effective_to,
                        version.source_summary_json, version.published_at, version.replaced_by_version_id
                 FROM curriculum_versions version
                 INNER JOIN curriculum_families family ON family.id = version.curriculum_family_id
                 WHERE version.status = 'published' AND family.is_public = 1
                 ORDER BY COALESCE(version.published_at, version.updated_at) DESC, version.id DESC
                 LIMIT 1",
                [],
                map_curriculum_version,
            )
            .optional()
            .map_err(|err| EcoachError::Storage(err.to_string()))
    }

    fn build_student_subject_card(
        &self,
        student_id: i64,
        subject: &CurriculumSubjectTrack,
        coverage: &HashMap<i64, String>,
        memory: &HashMap<i64, i64>,
    ) -> EcoachResult<CurriculumStudentSubjectCard> {
        let nodes = self.list_nodes_for_subject_track(subject.id)?;
        let total = nodes.len().max(1) as i64;
        let mut entered = 0_i64;
        let mut stable = 0_i64;
        let mut exam_ready = 0_i64;
        let mut weak = 0_i64;
        let mut blocked = 0_i64;
        let mut review_due = 0_i64;
        let mut strongest: Option<(i64, String)> = None;
        let mut weakest: Option<(i64, String)> = None;

        for node in nodes {
            let state =
                self.build_student_node_state(student_id, coverage, memory, node.clone())?;
            let strength_score = node
                .legacy_topic_id
                .and_then(|topic_id| coverage.get(&topic_id))
                .map(|status| 10_000 - coverage_priority(status))
                .unwrap_or(0);
            if state.status_label != "Not Started" {
                entered += 1;
            }
            if matches!(state.status_label.as_str(), "Strong" | "Exam Ready") {
                stable += 1;
            }
            if state.exam_ready {
                exam_ready += 1;
            }
            if matches!(
                state.status_label.as_str(),
                "Learning" | "Fragile" | "Slipping" | "Blocked"
            ) {
                weak += 1;
            }
            if state.blocked {
                blocked += 1;
            }
            if state.review_due {
                review_due += 1;
            }
            if strongest
                .as_ref()
                .map(|item| strength_score > item.0)
                .unwrap_or(true)
            {
                strongest = Some((strength_score, state.node.public_title.clone()));
            }
            if weakest
                .as_ref()
                .map(|item| strength_score < item.0)
                .unwrap_or(true)
            {
                weakest = Some((strength_score, state.node.public_title.clone()));
            }
        }

        let trend_label = if stable * 2 >= total && weak <= 2 {
            "improving"
        } else if weak > stable {
            "slipping"
        } else {
            "mixed"
        }
        .to_string();
        let next_action = if let Some(legacy_subject_id) = subject.legacy_subject_id {
            self.get_curriculum_next_best_topics(student_id, legacy_subject_id, 1)?
                .into_iter()
                .next()
                .map(|item| format!("Start {}", item.public_title))
        } else {
            None
        };

        Ok(CurriculumStudentSubjectCard {
            subject_track_id: subject.id,
            subject_slug: subject.subject_slug.clone(),
            public_title: subject.public_title.clone(),
            entered_percent: (entered * 100) / total,
            stable_percent: (stable * 100) / total,
            exam_ready_percent: (exam_ready * 100) / total,
            weak_area_count: weak,
            blocked_count: blocked,
            review_due_count: review_due,
            trend_label,
            strongest_topic_title: strongest.map(|item| item.1),
            weakest_topic_title: weakest.map(|item| item.1),
            next_action,
        })
    }

    fn build_student_node_state(
        &self,
        student_id: i64,
        coverage: &HashMap<i64, String>,
        memory: &HashMap<i64, i64>,
        node: CurriculumNode,
    ) -> EcoachResult<CurriculumStudentNodeState> {
        let blocked_by = self.unmet_prerequisite_titles(student_id, coverage, node.id)?;
        let legacy_status = node
            .legacy_topic_id
            .and_then(|topic_id| coverage.get(&topic_id).cloned())
            .unwrap_or_else(|| "not_introduced".to_string());
        let memory_strength = node
            .legacy_topic_id
            .and_then(|topic_id| memory.get(&topic_id).copied())
            .unwrap_or_default();
        let status_label = if !blocked_by.is_empty() {
            "Blocked".to_string()
        } else {
            student_facing_status(&legacy_status, memory_strength, node.exam_relevance_score)
                .to_string()
        };
        let review_due = matches!(legacy_status.as_str(), "decayed" | "re_opened")
            || (!matches!(status_label.as_str(), "Not Started" | "Blocked")
                && memory_strength < 4_500);
        let exam_ready = status_label == "Exam Ready";
        let reason = if !blocked_by.is_empty() {
            format!("Blocked by prerequisites: {}", blocked_by.join(", "))
        } else if review_due {
            format!(
                "{} needs review because it is slipping or memory strength is low.",
                node.public_title
            )
        } else {
            format!(
                "{} is currently marked as {}.",
                node.public_title,
                status_label.to_lowercase()
            )
        };
        Ok(CurriculumStudentNodeState {
            downstream_titles: self.list_downstream_titles(node.id)?,
            blocked: !blocked_by.is_empty(),
            review_due,
            exam_ready,
            reason,
            status_label,
            node,
        })
    }

    fn list_downstream_titles(&self, node_id: i64) -> EcoachResult<Vec<String>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT dependent.public_title
                 FROM curriculum_relationships relation
                 INNER JOIN curriculum_nodes dependent ON dependent.id = relation.from_entity_id
                 WHERE relation.from_entity_type = 'node'
                   AND relation.to_entity_type = 'node'
                   AND relation.to_entity_id = ?1
                   AND relation.relationship_type IN ('prerequisite', 'depends_on')
                 ORDER BY dependent.sequence_no ASC, dependent.public_title ASC",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map([node_id], |row| row.get::<_, String>(0))
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut titles = Vec::new();
        for row in rows {
            titles.push(row.map_err(|err| EcoachError::Storage(err.to_string()))?);
        }
        Ok(titles)
    }

    fn list_recent_movements(&self, student_id: i64, version_id: i64) -> EcoachResult<Vec<String>> {
        let mut statement = self
            .conn
            .prepare(
                "SELECT node.public_title, ledger.coverage_status
                 FROM curriculum_coverage_ledger ledger
                 INNER JOIN curriculum_nodes node ON node.legacy_topic_id = ledger.topic_id
                 WHERE ledger.student_id = ?1 AND node.curriculum_version_id = ?2
                 ORDER BY ledger.coverage_timestamp DESC
                 LIMIT 5",
            )
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let rows = statement
            .query_map(params![student_id, version_id], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|err| EcoachError::Storage(err.to_string()))?;
        let mut movements = Vec::new();
        for row in rows {
            let (title, status) = row.map_err(|err| EcoachError::Storage(err.to_string()))?;
            movements.push(format!(
                "{} moved into {}.",
                title,
                status.replace('_', " ")
            ));
        }
        Ok(movements)
    }
}

fn derive_curriculum_workflow_state(
    version_status: &str,
    superseded: bool,
    subject_count: i64,
    node_count: i64,
    pending_review_tasks: i64,
    low_confidence_nodes: i64,
) -> &'static str {
    if superseded {
        "superseded"
    } else if version_status == "published" {
        "published"
    } else if pending_review_tasks > 0 {
        "in_review"
    } else if subject_count > 0 && node_count > 0 && low_confidence_nodes == 0 {
        "ready_to_publish"
    } else {
        "draft"
    }
}

fn impact_severity(diff_type: &str) -> &'static str {
    match diff_type {
        "renamed_topic" => "cosmetic",
        "changed_ordering" | "changed_assessment_emphasis" => "interpretive",
        "added_topic" | "removed_topic" | "moved_topic" => "high_impact",
        _ => "structural",
    }
}

fn impact_action(diff_type: &str) -> &'static str {
    match diff_type {
        "renamed_topic" => "refresh_metadata",
        "changed_ordering" => "review",
        "changed_assessment_emphasis" => "regenerate",
        "added_topic" => "regenerate",
        "removed_topic" => "deprecate",
        "moved_topic" => "resequence",
        _ => "review",
    }
}

fn student_facing_status(
    legacy_status: &str,
    memory_strength: i64,
    exam_relevance_score: i64,
) -> &'static str {
    match legacy_status {
        "not_introduced" => "Not Started",
        "introduced" | "taught" => "Learning",
        "practiced" => {
            if memory_strength >= 7_000 {
                "Strong"
            } else {
                "Improving"
            }
        }
        "assessed" => {
            if memory_strength >= 7_000 && exam_relevance_score >= 7_500 {
                "Exam Ready"
            } else if memory_strength >= 6_000 {
                "Strong"
            } else {
                "Improving"
            }
        }
        "mastered" => {
            if memory_strength >= 6_500 && exam_relevance_score >= 7_500 {
                "Exam Ready"
            } else {
                "Strong"
            }
        }
        "unstable" => "Fragile",
        "decayed" | "re_opened" => "Slipping",
        _ => "Learning",
    }
}

fn serialize_json<T: serde::Serialize>(value: &T) -> EcoachResult<String> {
    serde_json::to_string(value).map_err(|err| EcoachError::Serialization(err.to_string()))
}

fn parse_string_list_column(index: usize, raw: &str) -> rusqlite::Result<Vec<String>> {
    serde_json::from_str::<Vec<String>>(raw).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(
            index,
            rusqlite::types::Type::Text,
            Box::new(EcoachError::Serialization(err.to_string())),
        )
    })
}

fn parse_i64_list_column(index: usize, raw: &str) -> rusqlite::Result<Vec<i64>> {
    serde_json::from_str::<Vec<i64>>(raw).map_err(|err| {
        rusqlite::Error::FromSqlConversionFailure(
            index,
            rusqlite::types::Type::Text,
            Box::new(EcoachError::Serialization(err.to_string())),
        )
    })
}

fn map_curriculum_node_citation(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<CurriculumNodeCitation> {
    Ok(CurriculumNodeCitation {
        id: row.get(0)?,
        curriculum_node_id: row.get(1)?,
        source_upload_id: row.get(2)?,
        citation_kind: row.get(3)?,
        reference_code: row.get(4)?,
        source_file_label: row.get(5)?,
        source_page: row.get(6)?,
        source_section: row.get(7)?,
        source_snippet: row.get(8)?,
        ocr_confidence_score: row.get(9)?,
        parsing_confidence_score: row.get(10)?,
        review_status: row.get(11)?,
    })
}

fn map_curriculum_node_exemplar(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<CurriculumNodeExemplar> {
    let metadata_json: String = row.get(6)?;
    Ok(CurriculumNodeExemplar {
        id: row.get(0)?,
        curriculum_node_id: row.get(1)?,
        citation_id: row.get(2)?,
        exemplar_kind: row.get(3)?,
        raw_text: row.get(4)?,
        public_text: row.get(5)?,
        metadata: parse_json_column(6, &metadata_json)?,
        display_order: row.get(7)?,
        review_status: row.get(8)?,
    })
}

fn map_curriculum_node_comment(row: &rusqlite::Row<'_>) -> rusqlite::Result<CurriculumNodeComment> {
    let metadata_json: String = row.get(6)?;
    Ok(CurriculumNodeComment {
        id: row.get(0)?,
        curriculum_node_id: row.get(1)?,
        citation_id: row.get(2)?,
        comment_type: row.get(3)?,
        comment_text: row.get(4)?,
        public_text: row.get(5)?,
        metadata: parse_json_column(6, &metadata_json)?,
        display_order: row.get(7)?,
        review_status: row.get(8)?,
    })
}

fn map_curriculum_node_intelligence(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<CurriculumNodeIntelligence> {
    let internal_subtopic_atoms_json: String = row.get(3)?;
    let knowledge_points_json: String = row.get(4)?;
    let skills_json: String = row.get(5)?;
    let misconception_tags_json: String = row.get(10)?;
    let prerequisite_node_ids_json: String = row.get(11)?;
    let dependent_node_ids_json: String = row.get(12)?;
    let difficulty_ladder_json: String = row.get(13)?;
    let teaching_strategies_json: String = row.get(14)?;
    let question_families_json: String = row.get(15)?;
    let worked_example_templates_json: String = row.get(16)?;
    let memory_tags_json: String = row.get(17)?;
    let local_context_examples_json: String = row.get(18)?;
    let exam_mapping_json: String = row.get(19)?;
    let notes_json: String = row.get(20)?;
    Ok(CurriculumNodeIntelligence {
        id: row.get(0)?,
        curriculum_node_id: row.get(1)?,
        friendly_topic_name: row.get(2)?,
        internal_subtopic_atoms: parse_string_list_column(3, &internal_subtopic_atoms_json)?,
        knowledge_points: parse_string_list_column(4, &knowledge_points_json)?,
        skills: parse_string_list_column(5, &skills_json)?,
        cognitive_verb: row.get(6)?,
        expected_evidence_type: row.get(7)?,
        instructional_mode: row.get(8)?,
        assessment_mode: row.get(9)?,
        misconception_tags: parse_string_list_column(10, &misconception_tags_json)?,
        prerequisite_node_ids: parse_i64_list_column(11, &prerequisite_node_ids_json)?,
        dependent_node_ids: parse_i64_list_column(12, &dependent_node_ids_json)?,
        difficulty_ladder: parse_string_list_column(13, &difficulty_ladder_json)?,
        teaching_strategies: parse_string_list_column(14, &teaching_strategies_json)?,
        question_families: parse_string_list_column(15, &question_families_json)?,
        worked_example_templates: parse_string_list_column(16, &worked_example_templates_json)?,
        memory_tags: parse_string_list_column(17, &memory_tags_json)?,
        local_context_examples: parse_string_list_column(18, &local_context_examples_json)?,
        exam_mapping: parse_json_column(19, &exam_mapping_json)?,
        notes: parse_json_column(20, &notes_json)?,
        approval_status: row.get(21)?,
    })
}

fn map_curriculum_cohort_pin(row: &rusqlite::Row<'_>) -> rusqlite::Result<CurriculumCohortPin> {
    Ok(CurriculumCohortPin {
        id: row.get(0)?,
        curriculum_version_id: row.get(1)?,
        cohort_key: row.get(2)?,
        cohort_label: row.get(3)?,
        level_code: row.get(4)?,
        effective_from: row.get(5)?,
        effective_to: row.get(6)?,
        rollout_status: row.get(7)?,
        pinned_by_account_id: row.get(8)?,
        notes: row.get(9)?,
    })
}

fn map_student_curriculum_assignment(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<StudentCurriculumAssignment> {
    Ok(StudentCurriculumAssignment {
        id: row.get(0)?,
        student_id: row.get(1)?,
        curriculum_version_id: row.get(2)?,
        cohort_pin_id: row.get(3)?,
        assignment_source: row.get(4)?,
        status: row.get(5)?,
        notes: row.get(6)?,
        assigned_at: row.get(7)?,
    })
}

fn map_curriculum_regeneration_job(
    row: &rusqlite::Row<'_>,
) -> rusqlite::Result<CurriculumRegenerationJob> {
    let payload_json: String = row.get(11)?;
    Ok(CurriculumRegenerationJob {
        id: row.get(0)?,
        base_version_id: row.get(1)?,
        compare_version_id: row.get(2)?,
        affected_node_id: row.get(3)?,
        entity_type: row.get(4)?,
        entity_key: row.get(5)?,
        severity: row.get(6)?,
        action_required: row.get(7)?,
        resource_type: row.get(8)?,
        resource_count: row.get(9)?,
        impact_summary: row.get(10)?,
        payload: parse_json_column(11, &payload_json)?,
        status: row.get(12)?,
        triggered_by_account_id: row.get(13)?,
    })
}
