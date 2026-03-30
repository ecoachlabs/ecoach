use ecoach_glossary::GlossaryService;
use ecoach_library::LibraryService;

use crate::{error::CommandError, state::AppState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryShelfDto {
    pub shelf_type: String,
    pub title: String,
    pub item_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryItemDto {
    pub id: i64,
    pub item_type: String,
    pub title: String,
    pub topic_id: Option<i64>,
    pub topic_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryEntryDto {
    pub id: i64,
    pub title: String,
    pub entry_type: String,
    pub short_text: Option<String>,
    pub topic_id: Option<i64>,
}

pub fn get_library_home(
    state: &AppState,
    student_id: i64,
) -> Result<Vec<LibraryShelfDto>, CommandError> {
    state.with_connection(|conn| {
        let service = LibraryService::new(conn);
        let snapshot = service.build_home_snapshot(student_id, 20)?;
        Ok(snapshot
            .generated_shelves
            .into_iter()
            .map(|shelf| LibraryShelfDto {
                shelf_type: shelf.shelf_type,
                title: shelf.title,
                item_count: shelf.items.len(),
            })
            .collect())
    })
}

pub fn save_library_item(
    state: &AppState,
    student_id: i64,
    item_type: String,
    reference_id: i64,
) -> Result<i64, CommandError> {
    state.with_connection(|conn| {
        let service = LibraryService::new(conn);
        let id = service.save_item(student_id, &item_type, reference_id)?;
        Ok(id)
    })
}

pub fn search_glossary(
    state: &AppState,
    query: String,
) -> Result<Vec<GlossaryEntryDto>, CommandError> {
    state.with_connection(|conn| {
        let service = GlossaryService::new(conn);
        let entries = service.search_entries(&query)?;
        Ok(entries
            .into_iter()
            .map(|e| GlossaryEntryDto {
                id: e.id,
                title: e.title,
                entry_type: e.entry_type,
                short_text: e.short_text,
                topic_id: e.topic_id,
            })
            .collect())
    })
}
