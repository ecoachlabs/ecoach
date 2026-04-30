use std::{collections::HashMap, path::PathBuf};

use ecoach_commands::{
    assessment_commands,
    assessment_commands::{
        AdminPastPaperOptionInput, AdminPastPaperQuestionInput, AdminPastPaperSaveInput,
    },
    content_commands, AppState, CommandError,
};
use rusqlite::{params, OptionalExtension};

fn main() {
    if let Err(err) = run() {
        eprintln!(
            "seed-bece-math-past-papers failed: [{}] {}",
            err.code, err.message
        );
        std::process::exit(1);
    }
}

fn run() -> Result<(), CommandError> {
    let db_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(default_runtime_db_path);

    let state = AppState::open_runtime(&db_path)?;
    let subject_id = ensure_math_subject(&state)?;
    let topics = load_topic_ids(&state, subject_id)?;

    let paper_2023 = upsert_seed_paper(
        &state,
        subject_id,
        SeedPaperSpec {
            exam_year: 2023,
            paper_code: "BECE-MATH-SEED-2023",
            title: "BECE Mathematics Practice Seed 2023",
            questions: build_2023_questions(&topics),
        },
    )?;

    let paper_2022 = upsert_seed_paper(
        &state,
        subject_id,
        SeedPaperSpec {
            exam_year: 2022,
            paper_code: "BECE-MATH-SEED-2022",
            title: "BECE Mathematics Practice Seed 2022",
            questions: build_2022_questions(&topics),
        },
    )?;

    let years = assessment_commands::list_past_papers_for_subject(&state, subject_id)?;
    println!("runtime_db => {}", db_path.display());
    println!(
        "seeded papers => {} (id {}), {} (id {})",
        paper_2023.title, paper_2023.paper_id, paper_2022.title, paper_2022.paper_id
    );
    for year in years.into_iter().filter(|year| {
        year.paper_code
            .as_deref()
            .map(|code| code.starts_with("BECE-MATH-SEED-"))
            .unwrap_or(false)
    }) {
        let sections = year
            .sections
            .into_iter()
            .map(|section| {
                format!(
                    "{}:{}:{}",
                    section.section_label,
                    format!("{:?}", section.section_kind).to_lowercase(),
                    section.question_count
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        let keyword_preview = year
            .keywords
            .iter()
            .take(6)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        println!(
            "paper {} {} => {} | topics:{} | keywords:{}",
            year.exam_year,
            year.paper_code.unwrap_or_else(|| year.title.clone()),
            sections,
            year.topic_ids.len(),
            keyword_preview
        );
    }

    Ok(())
}

fn default_runtime_db_path() -> PathBuf {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.ecoach.app")
        .join("ecoach.db")
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate directory should have parent")
        .parent()
        .expect("workspace root should exist")
        .to_path_buf()
}

fn find_math_subject_id(state: &AppState) -> Result<Option<i64>, CommandError> {
    state.with_connection(|conn| {
        conn.query_row(
            "SELECT id
             FROM subjects
             WHERE lower(code) = 'math' OR lower(name) LIKE '%math%'
             ORDER BY display_order ASC, id ASC
             LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(storage_error)
    })
}

fn ensure_math_subject(state: &AppState) -> Result<i64, CommandError> {
    if let Some(subject_id) = find_math_subject_id(state)? {
        return Ok(subject_id);
    }

    let pack_path = workspace_root().join("packs").join("math-bece-sample");
    content_commands::install_pack(state, pack_path.to_string_lossy().to_string())?;

    find_math_subject_id(state)?.ok_or(CommandError {
        code: "not_found".to_string(),
        message: "could not find or install a mathematics subject".to_string(),
    })
}

fn load_topic_ids(state: &AppState, subject_id: i64) -> Result<SeedTopicIds, CommandError> {
    let available_topic_ids = state.with_connection(|conn| {
        let mut stmt = conn
            .prepare(
                "SELECT id
                 FROM topics
                 WHERE subject_id = ?1
                 ORDER BY display_order ASC, id ASC",
            )
            .map_err(storage_error)?;
        let rows = stmt
            .query_map([subject_id], |row| row.get::<_, i64>(0))
            .map_err(storage_error)?;
        let mut ids = Vec::new();
        for row in rows {
            ids.push(row.map_err(storage_error)?);
        }
        Ok(ids)
    })?;

    let fallback = available_topic_ids.first().copied().ok_or(CommandError {
        code: "not_found".to_string(),
        message: format!("subject {} has no topics available", subject_id),
    })?;

    Ok(SeedTopicIds {
        number_ops: find_topic_id(
            state,
            subject_id,
            &["Whole Number and Decimal Operations", "Mental Mathematics"],
        )?
        .unwrap_or(fallback),
        fractions: find_topic_id(state, subject_id, &["Fractions, Decimals and Percentages"])?
            .unwrap_or(fallback),
        ratios: find_topic_id(
            state,
            subject_id,
            &["Number: Ratios and Proportion", "Ratios and Proportion"],
        )?
        .unwrap_or(fallback),
        equations: find_topic_id(state, subject_id, &["Variables and Equations"])?
            .unwrap_or(fallback),
        algebra: find_topic_id(state, subject_id, &["Algebraic Expressions"])?.unwrap_or(fallback),
        powers: find_topic_id(state, subject_id, &["Powers and Indices"])?.unwrap_or(fallback),
        measurement: find_topic_id(state, subject_id, &["Measurement"])?.unwrap_or(fallback),
        data: find_topic_id(state, subject_id, &["Data"])?.unwrap_or(fallback),
        probability: find_topic_id(state, subject_id, &["Chance or Probability"])?
            .unwrap_or(fallback),
    })
}

fn find_topic_id(
    state: &AppState,
    subject_id: i64,
    candidates: &[&str],
) -> Result<Option<i64>, CommandError> {
    for candidate in candidates {
        let exact = state.with_connection(|conn| {
            conn.query_row(
                "SELECT id
                 FROM topics
                 WHERE subject_id = ?1 AND lower(name) = lower(?2)
                 ORDER BY display_order ASC, id ASC
                 LIMIT 1",
                params![subject_id, candidate],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(storage_error)
        })?;
        if let Some(id) = exact {
            return Ok(Some(id));
        }
    }

    for candidate in candidates {
        let pattern = format!("%{}%", candidate);
        let fuzzy = state.with_connection(|conn| {
            conn.query_row(
                "SELECT id
                 FROM topics
                 WHERE subject_id = ?1 AND lower(name) LIKE lower(?2)
                 ORDER BY display_order ASC, id ASC
                 LIMIT 1",
                params![subject_id, pattern],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(storage_error)
        })?;
        if let Some(id) = fuzzy {
            return Ok(Some(id));
        }
    }

    Ok(None)
}

fn upsert_seed_paper(
    state: &AppState,
    subject_id: i64,
    spec: SeedPaperSpec,
) -> Result<SeedPaperResult, CommandError> {
    let existing = load_existing_paper(state, spec.paper_code)?;
    let mut question_ids = existing
        .as_ref()
        .map(|paper| paper.question_ids.clone())
        .unwrap_or_default();
    let mut paper_id = existing.as_ref().map(|paper| paper.paper_id);
    let mut staged_questions: Vec<AdminPastPaperQuestionInput> = Vec::new();

    for question in spec.questions.into_iter() {
        let key = question_lookup_key(&question.section_label, question.question_number.as_deref());
        let section_label = question.section_label.clone();
        let question_number = question.question_number.clone().unwrap_or_default();
        staged_questions.push(AdminPastPaperQuestionInput {
            question_id: question_ids.get(&key).copied(),
            ..question
        });

        let result = assessment_commands::admin_save_past_paper(
            state,
            AdminPastPaperSaveInput {
                paper_id,
                subject_id,
                exam_year: spec.exam_year,
                paper_code: Some(spec.paper_code.to_string()),
                title: spec.title.to_string(),
                questions: staged_questions.clone(),
            },
        )
        .map_err(|err| CommandError {
            code: err.code,
            message: format!(
                "{} failed at section {} question {}: {}",
                spec.paper_code, section_label, question_number, err.message
            ),
        })?;

        paper_id = Some(result.paper_id);
        question_ids = load_existing_paper(state, spec.paper_code)?
            .map(|paper| paper.question_ids)
            .unwrap_or_default();
    }

    let result = assessment_commands::admin_save_past_paper(
        state,
        AdminPastPaperSaveInput {
            paper_id,
            subject_id,
            exam_year: spec.exam_year,
            paper_code: Some(spec.paper_code.to_string()),
            title: spec.title.to_string(),
            questions: staged_questions,
        },
    )?;
    Ok(SeedPaperResult {
        paper_id: result.paper_id,
        title: spec.title.to_string(),
    })
}

fn load_existing_paper(
    state: &AppState,
    paper_code: &str,
) -> Result<Option<ExistingPaper>, CommandError> {
    let paper_id = state.with_connection(|conn| {
        conn.query_row(
            "SELECT id FROM past_paper_sets WHERE paper_code = ?1 LIMIT 1",
            [paper_code],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map_err(storage_error)
    })?;

    let Some(paper_id) = paper_id else {
        return Ok(None);
    };

    let paper = assessment_commands::admin_get_past_paper(state, paper_id)?;
    let question_ids = paper
        .questions
        .into_iter()
        .filter_map(|question| {
            question.question_number.map(|number| {
                (
                    question_lookup_key(&question.section_label, Some(number.as_str())),
                    question.question_id,
                )
            })
        })
        .collect::<HashMap<_, _>>();

    Ok(Some(ExistingPaper {
        paper_id,
        question_ids,
    }))
}

fn question_lookup_key(section_label: &str, question_number: Option<&str>) -> String {
    format!(
        "{}::{}",
        section_label.trim().to_ascii_uppercase(),
        question_number.unwrap_or_default().trim()
    )
}

fn build_2023_questions(topics: &SeedTopicIds) -> Vec<AdminPastPaperQuestionInput> {
    vec![
        mcq(
            "A",
            "1",
            topics.fractions,
            "Simplify 15/20 to its lowest terms.",
            "Divide the numerator and denominator by 5 to get 3/4.",
            3600,
            vec![
                option("A", "3/4", true),
                option("B", "5/4", false),
                option("C", "2/5", false),
                option("D", "4/3", false),
            ],
        ),
        mcq(
            "A",
            "2",
            topics.fractions,
            "Find 25% of 160.",
            "25% is one quarter, and one quarter of 160 is 40.",
            3400,
            vec![
                option("A", "20", false),
                option("B", "40", true),
                option("C", "60", false),
                option("D", "80", false),
            ],
        ),
        mcq(
            "A",
            "3",
            topics.equations,
            "Solve 3x + 5 = 20.",
            "Subtract 5 from both sides to get 3x = 15, then divide by 3.",
            4300,
            vec![
                option("A", "3", false),
                option("B", "4", false),
                option("C", "5", true),
                option("D", "8", false),
            ],
        ),
        mcq(
            "A",
            "4",
            topics.ratios,
            "A class has boys and girls in the ratio 3:5. If there are 40 students altogether, how many are boys?",
            "The total number of parts is 8, so each part is 40/8 = 5. Boys are 3 parts: 3 x 5 = 15.",
            4700,
            vec![
                option("A", "12", false),
                option("B", "15", true),
                option("C", "20", false),
                option("D", "25", false),
            ],
        ),
        mcq(
            "A",
            "5",
            topics.measurement,
            "Find the area of a circle of radius 7 cm. Take pi = 22/7.",
            "Area = pi r^2 = (22/7) x 7 x 7 = 154 cm^2.",
            5200,
            vec![
                option("A", "44 cm^2", false),
                option("B", "77 cm^2", false),
                option("C", "154 cm^2", true),
                option("D", "308 cm^2", false),
            ],
        ),
        mcq(
            "A",
            "6",
            topics.data,
            "Find the mean of 4, 6, 8 and 12.",
            "Add the values to get 30 and divide by 4 to get 7.5.",
            3900,
            vec![
                option("A", "6.5", false),
                option("B", "7.0", false),
                option("C", "7.5", true),
                option("D", "8.0", false),
            ],
        ),
        mcq(
            "A",
            "7",
            topics.ratios,
            "Find the simple interest on GHC600 for 2 years at 5% per annum.",
            "Simple interest = PRT/100 = 600 x 5 x 2 / 100 = 60.",
            4800,
            vec![
                option("A", "30", false),
                option("B", "60", true),
                option("C", "120", false),
                option("D", "660", false),
            ],
        ),
        mcq(
            "A",
            "8",
            topics.probability,
            "A bag contains 3 red balls and 2 blue balls. What is the probability of picking a red ball at random?",
            "There are 5 balls altogether, and 3 of them are red, so the probability is 3/5.",
            4100,
            vec![
                option("A", "2/5", false),
                option("B", "3/5", true),
                option("C", "1/2", false),
                option("D", "5/3", false),
            ],
        ),
        mcq(
            "A",
            "9",
            topics.algebra,
            "Expand 2(3x - 4).",
            "Multiply 2 by each term inside the bracket: 2 x 3x = 6x and 2 x -4 = -8.",
            4500,
            vec![
                option("A", "6x - 8", true),
                option("B", "6x - 4", false),
                option("C", "5x - 8", false),
                option("D", "6x + 8", false),
            ],
        ),
        mcq(
            "A",
            "10",
            topics.measurement,
            "The exterior angle of a triangle is equal to the sum of the two opposite interior angles. If the two opposite interior angles are 35 degrees and 65 degrees, find the exterior angle.",
            "Add the two opposite interior angles: 35 + 65 = 100 degrees.",
            4200,
            vec![
                option("A", "90 degrees", false),
                option("B", "95 degrees", false),
                option("C", "100 degrees", true),
                option("D", "110 degrees", false),
            ],
        ),
        mcq(
            "A",
            "11",
            topics.number_ops,
            "Find the least common multiple (LCM) of 12 and 18.",
            "The common multiple with the smallest value is 36.",
            4200,
            vec![
                option("A", "6", false),
                option("B", "24", false),
                option("C", "36", true),
                option("D", "54", false),
            ],
        ),
        mcq(
            "A",
            "12",
            topics.fractions,
            "Which of the following is the greatest?",
            "Convert each to decimals: 2/5 = 0.4, 38% = 0.38, 0.45 = 0.45, and 0.405 = 0.405. So 0.45 is greatest.",
            4100,
            vec![
                option("A", "2/5", false),
                option("B", "38%", false),
                option("C", "0.45", true),
                option("D", "0.405", false),
            ],
        ),
        mcq(
            "A",
            "13",
            topics.powers,
            "Evaluate 3^2 + 4^2.",
            "3 squared is 9 and 4 squared is 16. Their sum is 25.",
            3600,
            vec![
                option("A", "7", false),
                option("B", "13", false),
                option("C", "25", true),
                option("D", "49", false),
            ],
        ),
        mcq(
            "A",
            "14",
            topics.algebra,
            "Factorise x^2 + 5x + 6.",
            "The factors of 6 that add up to 5 are 2 and 3, so the expression becomes (x + 2)(x + 3).",
            5200,
            vec![
                option("A", "(x + 1)(x + 6)", false),
                option("B", "(x + 2)(x + 3)", true),
                option("C", "(x - 2)(x - 3)", false),
                option("D", "(x + 5)(x + 1)", false),
            ],
        ),
        mcq(
            "A",
            "15",
            topics.equations,
            "Solve 5x = 3x + 14.",
            "Subtract 3x from both sides to get 2x = 14, so x = 7.",
            4300,
            vec![
                option("A", "5", false),
                option("B", "7", true),
                option("C", "8", false),
                option("D", "14", false),
            ],
        ),
        mcq(
            "A",
            "16",
            topics.measurement,
            "Find the circumference of a circle of diameter 14 cm. Take pi = 22/7.",
            "Circumference = pi x d = (22/7) x 14 = 44 cm.",
            4300,
            vec![
                option("A", "22 cm", false),
                option("B", "44 cm", true),
                option("C", "88 cm", false),
                option("D", "154 cm", false),
            ],
        ),
        mcq(
            "A",
            "17",
            topics.data,
            "Find the median of 3, 5, 7, 8 and 12.",
            "The numbers are already arranged, and the middle value is 7.",
            3200,
            vec![
                option("A", "5", false),
                option("B", "7", true),
                option("C", "8", false),
                option("D", "12", false),
            ],
        ),
        mcq(
            "A",
            "18",
            topics.probability,
            "What is the probability of not getting a head when a fair coin is tossed once?",
            "The only outcome that is not a head is a tail, so the probability is 1 out of 2.",
            3300,
            vec![
                option("A", "0", false),
                option("B", "1/4", false),
                option("C", "1/2", true),
                option("D", "1", false),
            ],
        ),
        mcq(
            "A",
            "19",
            topics.ratios,
            "On a map drawn to a scale of 1 : 50,000, the distance between two towns is 4 cm. Find the actual distance.",
            "Actual distance = 4 x 50,000 cm = 200,000 cm = 2 km.",
            5200,
            vec![
                option("A", "0.2 km", false),
                option("B", "2 km", true),
                option("C", "20 km", false),
                option("D", "200 km", false),
            ],
        ),
        mcq(
            "A",
            "20",
            topics.measurement,
            "Find the area of a triangle with base 12 cm and height 9 cm.",
            "Area = 1/2 x base x height = 1/2 x 12 x 9 = 54 cm^2.",
            4000,
            vec![
                option("A", "21 cm^2", false),
                option("B", "54 cm^2", true),
                option("C", "108 cm^2", false),
                option("D", "216 cm^2", false),
            ],
        ),
        mcq(
            "A",
            "21",
            topics.fractions,
            "Evaluate 3/4 + 2/3.",
            "Use a common denominator of 12: 9/12 + 8/12 = 17/12.",
            5000,
            vec![
                option("A", "5/7", false),
                option("B", "17/12", true),
                option("C", "11/12", false),
                option("D", "19/12", false),
            ],
        ),
        mcq(
            "A",
            "22",
            topics.algebra,
            "Simplify 5a - 2a + 7.",
            "Combine like terms: 5a - 2a = 3a, so the result is 3a + 7.",
            3600,
            vec![
                option("A", "7a", false),
                option("B", "3a + 7", true),
                option("C", "3a - 7", false),
                option("D", "5a + 5", false),
            ],
        ),
        mcq(
            "A",
            "23",
            topics.number_ops,
            "What is the least prime factor of 91?",
            "91 = 7 x 13, so its least prime factor is 7.",
            4700,
            vec![
                option("A", "3", false),
                option("B", "5", false),
                option("C", "7", true),
                option("D", "13", false),
            ],
        ),
        mcq(
            "A",
            "24",
            topics.ratios,
            "A shirt marked GHC200 is sold at a discount of 15%. Find the discount.",
            "Discount = 15/100 x 200 = GHC30.",
            4200,
            vec![
                option("A", "GHC15", false),
                option("B", "GHC20", false),
                option("C", "GHC30", true),
                option("D", "GHC170", false),
            ],
        ),
        mcq(
            "A",
            "25",
            topics.measurement,
            "Find the perimeter of a rectangle of length 11 cm and width 7 cm.",
            "Perimeter = 2(l + w) = 2(11 + 7) = 36 cm.",
            3500,
            vec![
                option("A", "18 cm", false),
                option("B", "22 cm", false),
                option("C", "36 cm", true),
                option("D", "77 cm", false),
            ],
        ),
        mcq(
            "A",
            "26",
            topics.data,
            "Find the range of 6, 9, 13, 15 and 8.",
            "Range = highest value - lowest value = 15 - 6 = 9.",
            3300,
            vec![
                option("A", "7", false),
                option("B", "8", false),
                option("C", "9", true),
                option("D", "21", false),
            ],
        ),
        mcq(
            "A",
            "27",
            topics.probability,
            "A card is picked at random from a standard pack of 52 cards. What is the probability of picking an ace?",
            "There are 4 aces in 52 cards, so the probability is 4/52 = 1/13.",
            4500,
            vec![
                option("A", "1/4", false),
                option("B", "1/13", true),
                option("C", "4/13", false),
                option("D", "13/52", false),
            ],
        ),
        mcq(
            "A",
            "28",
            topics.powers,
            "Find the square root of 144.",
            "12 x 12 = 144, so the square root is 12.",
            3100,
            vec![
                option("A", "10", false),
                option("B", "11", false),
                option("C", "12", true),
                option("D", "14", false),
            ],
        ),
        mcq(
            "A",
            "29",
            topics.equations,
            "If y/3 = 7, find y.",
            "Multiply both sides by 3 to get y = 21.",
            3200,
            vec![
                option("A", "10", false),
                option("B", "18", false),
                option("C", "21", true),
                option("D", "28", false),
            ],
        ),
        mcq(
            "A",
            "30",
            topics.measurement,
            "Find the area of a semicircle of radius 14 cm. Take pi = 22/7.",
            "Area of full circle = (22/7) x 14 x 14 = 616 cm^2. Half of this is 308 cm^2.",
            5500,
            vec![
                option("A", "154 cm^2", false),
                option("B", "308 cm^2", true),
                option("C", "616 cm^2", false),
                option("D", "44 cm^2", false),
            ],
        ),
        mcq(
            "A",
            "31",
            topics.fractions,
            "Write 1.25 as a mixed number.",
            "1.25 = 1 + 0.25 = 1 1/4.",
            3400,
            vec![
                option("A", "1 1/4", true),
                option("B", "1 2/5", false),
                option("C", "1 1/5", false),
                option("D", "5/4", false),
            ],
        ),
        mcq(
            "A",
            "32",
            topics.algebra,
            "Evaluate 2x - 3 when x = 5.",
            "Substitute x = 5: 2(5) - 3 = 10 - 3 = 7.",
            3000,
            vec![
                option("A", "5", false),
                option("B", "7", true),
                option("C", "10", false),
                option("D", "13", false),
            ],
        ),
        mcq(
            "A",
            "33",
            topics.number_ops,
            "Multiply 2.75 by 10.",
            "Multiplying by 10 moves the decimal point one place to the right, giving 27.5.",
            2800,
            vec![
                option("A", "2.75", false),
                option("B", "7.25", false),
                option("C", "27.5", true),
                option("D", "275", false),
            ],
        ),
        mcq(
            "A",
            "34",
            topics.ratios,
            "Divide 72 in the ratio 5 : 7. What is the larger share?",
            "Total parts = 12, so each part is 72/12 = 6. The larger share is 7 x 6 = 42.",
            4700,
            vec![
                option("A", "30", false),
                option("B", "35", false),
                option("C", "42", true),
                option("D", "49", false),
            ],
        ),
        mcq(
            "A",
            "35",
            topics.measurement,
            "Convert 2.5 metres to centimetres.",
            "1 metre = 100 cm, so 2.5 m = 250 cm.",
            3000,
            vec![
                option("A", "25 cm", false),
                option("B", "250 cm", true),
                option("C", "2500 cm", false),
                option("D", "0.25 cm", false),
            ],
        ),
        mcq(
            "A",
            "36",
            topics.data,
            "Find the mean of 9, 10, 11, 12 and 13.",
            "The sum is 55 and 55/5 = 11.",
            3100,
            vec![
                option("A", "10", false),
                option("B", "11", true),
                option("C", "12", false),
                option("D", "13", false),
            ],
        ),
        mcq(
            "A",
            "37",
            topics.probability,
            "A spinner is numbered 1 to 8. What is the probability of landing on a number greater than 5?",
            "The numbers greater than 5 are 6, 7 and 8. So the probability is 3 out of 8.",
            3600,
            vec![
                option("A", "1/4", false),
                option("B", "3/8", true),
                option("C", "1/2", false),
                option("D", "5/8", false),
            ],
        ),
        mcq(
            "A",
            "38",
            topics.equations,
            "Solve 4(x - 2) = 20.",
            "Divide by 4 to get x - 2 = 5, then add 2 to get x = 7.",
            4300,
            vec![
                option("A", "3", false),
                option("B", "5", false),
                option("C", "7", true),
                option("D", "8", false),
            ],
        ),
        mcq(
            "A",
            "39",
            topics.measurement,
            "Find the supplement of 125 degrees.",
            "Angles on a straight line add up to 180 degrees, so the supplement is 180 - 125 = 55 degrees.",
            3200,
            vec![
                option("A", "45 degrees", false),
                option("B", "55 degrees", true),
                option("C", "65 degrees", false),
                option("D", "125 degrees", false),
            ],
        ),
        mcq(
            "A",
            "40",
            topics.fractions,
            "If 3/5 of a number is 24, find the number.",
            "Let the number be n. Then 3/5 of n = 24, so n = 24 x 5/3 = 40.",
            5000,
            vec![
                option("A", "30", false),
                option("B", "35", false),
                option("C", "40", true),
                option("D", "45", false),
            ],
        ),
        theory(
            "B",
            "1",
            topics.ratios,
            "A trader buys 36 exercise books at GHC8 each and sells each one at GHC10.50. Find the total profit and the percentage profit.",
            "short_answer",
            6,
            5600,
            "Cost price = 36 x 8 = GHC288. Selling price = 36 x 10.50 = GHC378. Profit = 378 - 288 = GHC90. Percentage profit = (90/288) x 100 = 31.25%.",
        ),
        theory(
            "B",
            "2",
            topics.equations,
            "Solve the simultaneous equations x + y = 17 and x - y = 5.",
            "short_answer",
            6,
            5900,
            "Add the equations to get 2x = 22, so x = 11. Substitute into x + y = 17 to get y = 6.",
        ),
        theory(
            "B",
            "3",
            topics.measurement,
            "A rectangular field measures 18 m by 12 m. A path 1 m wide is made all around the outside of the field. Find the area of the path.",
            "short_answer",
            5,
            6100,
            "Outer dimensions = 20 m by 14 m, so outer area = 280 m^2. Inner area = 18 x 12 = 216 m^2. Area of path = 280 - 216 = 64 m^2.",
        ),
        theory(
            "B",
            "4",
            topics.data,
            "The marks scored by 8 students are 4, 7, 9, 6, 4, 8, 7 and 5. Find the mean, median and mode.",
            "short_answer",
            6,
            5400,
            "Sorted data: 4, 4, 5, 6, 7, 7, 8, 9. Mean = 50/8 = 6.25. Median = (6 + 7)/2 = 6.5. Mode = 4 and 7.",
        ),
    ]
}

fn build_2022_questions(topics: &SeedTopicIds) -> Vec<AdminPastPaperQuestionInput> {
    vec![
        mcq(
            "A",
            "1",
            topics.fractions,
            "Write 0.375 as a fraction in its simplest form.",
            "0.375 = 375/1000. Divide the numerator and denominator by 125 to get 3/8.",
            3800,
            vec![
                option("A", "3/5", false),
                option("B", "3/8", true),
                option("C", "5/8", false),
                option("D", "8/3", false),
            ],
        ),
        mcq(
            "A",
            "2",
            topics.equations,
            "Solve 2y - 7 = 11.",
            "Add 7 to both sides to get 2y = 18, then divide by 2.",
            4100,
            vec![
                option("A", "7", false),
                option("B", "8", false),
                option("C", "9", true),
                option("D", "11", false),
            ],
        ),
        mcq(
            "A",
            "3",
            topics.number_ops,
            "Find the highest common factor (HCF) of 18 and 24.",
            "The common factors are 1, 2, 3 and 6. The highest is 6.",
            4300,
            vec![
                option("A", "3", false),
                option("B", "6", true),
                option("C", "9", false),
                option("D", "12", false),
            ],
        ),
        mcq(
            "A",
            "4",
            topics.fractions,
            "Find the percentage increase when a quantity rises from 80 to 100.",
            "Increase = 20. Percentage increase = (20/80) x 100 = 25%.",
            4500,
            vec![
                option("A", "20%", false),
                option("B", "25%", true),
                option("C", "40%", false),
                option("D", "80%", false),
            ],
        ),
        mcq(
            "A",
            "5",
            topics.powers,
            "Evaluate (2^4 x 2) / 2^2.",
            "2^4 x 2 = 16 x 2 = 32, and 32 / 4 = 8.",
            4700,
            vec![
                option("A", "4", false),
                option("B", "6", false),
                option("C", "8", true),
                option("D", "16", false),
            ],
        ),
        mcq(
            "A",
            "6",
            topics.measurement,
            "Find the volume of a cube of side 4 cm.",
            "Volume of a cube = side x side x side = 4 x 4 x 4 = 64 cm^3.",
            3900,
            vec![
                option("A", "16 cm^3", false),
                option("B", "32 cm^3", false),
                option("C", "64 cm^3", true),
                option("D", "128 cm^3", false),
            ],
        ),
        mcq(
            "A",
            "7",
            topics.ratios,
            "If 5 pens cost GHC15, how much will 8 pens cost at the same rate?",
            "One pen costs GHC3, so 8 pens cost 8 x 3 = GHC24.",
            4300,
            vec![
                option("A", "GHC18", false),
                option("B", "GHC20", false),
                option("C", "GHC24", true),
                option("D", "GHC30", false),
            ],
        ),
        mcq(
            "A",
            "8",
            topics.probability,
            "What is the probability of getting an even number when a fair die is thrown once?",
            "The even outcomes are 2, 4 and 6, so there are 3 favourable outcomes out of 6. That simplifies to 1/2.",
            3600,
            vec![
                option("A", "1/3", false),
                option("B", "1/2", true),
                option("C", "2/3", false),
                option("D", "3/2", false),
            ],
        ),
        mcq(
            "A",
            "9",
            topics.measurement,
            "Find the sum of the interior angles of a quadrilateral.",
            "A quadrilateral can be split into two triangles, so the total is 2 x 180 = 360 degrees.",
            3400,
            vec![
                option("A", "180 degrees", false),
                option("B", "270 degrees", false),
                option("C", "360 degrees", true),
                option("D", "540 degrees", false),
            ],
        ),
        mcq(
            "A",
            "10",
            topics.data,
            "Find the mode of 2, 3, 3, 4, 5, 5, 5 and 6.",
            "The value 5 appears most often, so the mode is 5.",
            3200,
            vec![
                option("A", "3", false),
                option("B", "4", false),
                option("C", "5", true),
                option("D", "6", false),
            ],
        ),
        mcq(
            "A",
            "11",
            topics.number_ops,
            "Find the least common multiple (LCM) of 8 and 12.",
            "The least common multiple of 8 and 12 is 24.",
            3900,
            vec![
                option("A", "4", false),
                option("B", "12", false),
                option("C", "24", true),
                option("D", "48", false),
            ],
        ),
        mcq(
            "A",
            "12",
            topics.fractions,
            "Express 0.625 as a percentage.",
            "Multiply by 100 to get 62.5%.",
            3600,
            vec![
                option("A", "6.25%", false),
                option("B", "62.5%", true),
                option("C", "625%", false),
                option("D", "0.625%", false),
            ],
        ),
        mcq(
            "A",
            "13",
            topics.equations,
            "Solve 7k + 4 = 25.",
            "Subtract 4 to get 7k = 21, then divide by 7.",
            3900,
            vec![
                option("A", "2", false),
                option("B", "3", true),
                option("C", "4", false),
                option("D", "7", false),
            ],
        ),
        mcq(
            "A",
            "14",
            topics.ratios,
            "Divide 54 in the ratio 4 : 5. What is the larger part?",
            "Total parts = 9, so one part is 54/9 = 6. The larger part is 5 x 6 = 30.",
            4500,
            vec![
                option("A", "24", false),
                option("B", "27", false),
                option("C", "30", true),
                option("D", "36", false),
            ],
        ),
        mcq(
            "A",
            "15",
            topics.measurement,
            "Find the circumference of a circle of radius 3.5 cm. Take pi = 22/7.",
            "Circumference = 2pi r = 2 x (22/7) x 3.5 = 22 cm.",
            4300,
            vec![
                option("A", "11 cm", false),
                option("B", "22 cm", true),
                option("C", "38.5 cm", false),
                option("D", "44 cm", false),
            ],
        ),
        mcq(
            "A",
            "16",
            topics.data,
            "Find the mean of 5, 7, 10 and 13.",
            "The sum is 35, and 35/4 = 8.75.",
            3600,
            vec![
                option("A", "8", false),
                option("B", "8.5", false),
                option("C", "8.75", true),
                option("D", "9", false),
            ],
        ),
        mcq(
            "A",
            "17",
            topics.probability,
            "What is the probability of getting an odd number when a fair die is thrown once?",
            "The odd numbers are 1, 3 and 5, so the probability is 3/6 = 1/2.",
            3200,
            vec![
                option("A", "1/3", false),
                option("B", "1/2", true),
                option("C", "2/3", false),
                option("D", "5/6", false),
            ],
        ),
        mcq(
            "A",
            "18",
            topics.algebra,
            "Factorise 2x + 6.",
            "Take out the common factor 2 to get 2(x + 3).",
            3500,
            vec![
                option("A", "2(x + 3)", true),
                option("B", "3(x + 2)", false),
                option("C", "2(x + 6)", false),
                option("D", "x(2 + 6)", false),
            ],
        ),
        mcq(
            "A",
            "19",
            topics.powers,
            "Evaluate 5^0.",
            "Any non-zero number raised to the power 0 is 1.",
            2800,
            vec![
                option("A", "0", false),
                option("B", "1", true),
                option("C", "5", false),
                option("D", "25", false),
            ],
        ),
        mcq(
            "A",
            "20",
            topics.measurement,
            "Find the area of a rectangle of length 15 cm and width 9 cm.",
            "Area = length x width = 15 x 9 = 135 cm^2.",
            3200,
            vec![
                option("A", "24 cm^2", false),
                option("B", "90 cm^2", false),
                option("C", "135 cm^2", true),
                option("D", "270 cm^2", false),
            ],
        ),
        mcq(
            "A",
            "21",
            topics.fractions,
            "Evaluate 5/6 - 1/4.",
            "Use a common denominator of 12: 10/12 - 3/12 = 7/12.",
            4800,
            vec![
                option("A", "1/2", false),
                option("B", "7/12", true),
                option("C", "4/12", false),
                option("D", "9/12", false),
            ],
        ),
        mcq(
            "A",
            "22",
            topics.number_ops,
            "Evaluate 0.48 / 0.06.",
            "Multiply both numbers by 100 to get 48/6 = 8.",
            4200,
            vec![
                option("A", "0.8", false),
                option("B", "6", false),
                option("C", "8", true),
                option("D", "80", false),
            ],
        ),
        mcq(
            "A",
            "23",
            topics.ratios,
            "Find the simple interest on GHC400 for 3 years at 4% per annum.",
            "Simple interest = 400 x 4 x 3 / 100 = 48.",
            4500,
            vec![
                option("A", "GHC12", false),
                option("B", "GHC16", false),
                option("C", "GHC48", true),
                option("D", "GHC448", false),
            ],
        ),
        mcq(
            "A",
            "24",
            topics.measurement,
            "Find the volume of a cuboid measuring 5 cm by 4 cm by 3 cm.",
            "Volume = length x width x height = 5 x 4 x 3 = 60 cm^3.",
            3300,
            vec![
                option("A", "12 cm^3", false),
                option("B", "20 cm^3", false),
                option("C", "60 cm^3", true),
                option("D", "120 cm^3", false),
            ],
        ),
        mcq(
            "A",
            "25",
            topics.data,
            "Find the median of 2, 4, 5, 7, 9 and 10.",
            "The middle two values are 5 and 7, so the median is (5 + 7)/2 = 6.",
            4300,
            vec![
                option("A", "5", false),
                option("B", "6", true),
                option("C", "7", false),
                option("D", "8", false),
            ],
        ),
        mcq(
            "A",
            "26",
            topics.probability,
            "A bag contains 4 green balls and 1 yellow ball. What is the probability of picking the yellow ball?",
            "There is 1 yellow ball out of 5 balls altogether, so the probability is 1/5.",
            3100,
            vec![
                option("A", "1/5", true),
                option("B", "1/4", false),
                option("C", "4/5", false),
                option("D", "5", false),
            ],
        ),
        mcq(
            "A",
            "27",
            topics.algebra,
            "Expand (x + 3)(x + 2).",
            "Multiply term by term to get x^2 + 2x + 3x + 6 = x^2 + 5x + 6.",
            5200,
            vec![
                option("A", "x^2 + 6x + 5", false),
                option("B", "x^2 + 5x + 6", true),
                option("C", "x^2 + x + 6", false),
                option("D", "2x^2 + 5x + 6", false),
            ],
        ),
        mcq(
            "A",
            "28",
            topics.powers,
            "Find the cube root of 64.",
            "4 x 4 x 4 = 64, so the cube root is 4.",
            3000,
            vec![
                option("A", "3", false),
                option("B", "4", true),
                option("C", "6", false),
                option("D", "8", false),
            ],
        ),
        mcq(
            "A",
            "29",
            topics.equations,
            "Solve 9 - m = 4.",
            "Subtract 4 from 9 to get m = 5.",
            3000,
            vec![
                option("A", "4", false),
                option("B", "5", true),
                option("C", "9", false),
                option("D", "13", false),
            ],
        ),
        mcq(
            "A",
            "30",
            topics.measurement,
            "Two angles of a triangle are 35 degrees and 65 degrees. Find the third angle.",
            "The angles in a triangle add up to 180 degrees, so the third angle is 180 - (35 + 65) = 80 degrees.",
            3500,
            vec![
                option("A", "70 degrees", false),
                option("B", "75 degrees", false),
                option("C", "80 degrees", true),
                option("D", "90 degrees", false),
            ],
        ),
        mcq(
            "A",
            "31",
            topics.fractions,
            "Find 30% of 250.",
            "30% of 250 = 30/100 x 250 = 75.",
            3400,
            vec![
                option("A", "25", false),
                option("B", "50", false),
                option("C", "75", true),
                option("D", "125", false),
            ],
        ),
        mcq(
            "A",
            "32",
            topics.number_ops,
            "Find the highest common factor (HCF) of 42 and 56.",
            "The common factors of 42 and 56 include 1, 2, 7 and 14. The highest is 14.",
            4300,
            vec![
                option("A", "7", false),
                option("B", "12", false),
                option("C", "14", true),
                option("D", "28", false),
            ],
        ),
        mcq(
            "A",
            "33",
            topics.ratios,
            "On a map drawn to a scale of 1 : 100,000, the distance between two villages is 3 cm. Find the actual distance.",
            "Actual distance = 3 x 100,000 cm = 300,000 cm = 3 km.",
            5000,
            vec![
                option("A", "0.3 km", false),
                option("B", "3 km", true),
                option("C", "30 km", false),
                option("D", "300 km", false),
            ],
        ),
        mcq(
            "A",
            "34",
            topics.data,
            "Find the mode of 7, 8, 8, 9, 10, 10, 10 and 11.",
            "10 appears three times, more than any other number.",
            3000,
            vec![
                option("A", "8", false),
                option("B", "9", false),
                option("C", "10", true),
                option("D", "11", false),
            ],
        ),
        mcq(
            "A",
            "35",
            topics.probability,
            "What is the probability of obtaining a prime number when a fair die is thrown once?",
            "The prime numbers on a die are 2, 3 and 5. So the probability is 3/6 = 1/2.",
            3500,
            vec![
                option("A", "1/3", false),
                option("B", "1/2", true),
                option("C", "2/3", false),
                option("D", "5/6", false),
            ],
        ),
        mcq(
            "A",
            "36",
            topics.algebra,
            "Evaluate 3p + 2 when p = 4.",
            "Substitute p = 4 to get 3(4) + 2 = 14.",
            3000,
            vec![
                option("A", "10", false),
                option("B", "12", false),
                option("C", "14", true),
                option("D", "18", false),
            ],
        ),
        mcq(
            "A",
            "37",
            topics.measurement,
            "Find the area of a trapezium with parallel sides 8 cm and 12 cm and height 5 cm.",
            "Area = 1/2 x (8 + 12) x 5 = 50 cm^2.",
            4800,
            vec![
                option("A", "20 cm^2", false),
                option("B", "40 cm^2", false),
                option("C", "50 cm^2", true),
                option("D", "100 cm^2", false),
            ],
        ),
        mcq(
            "A",
            "38",
            topics.fractions,
            "Find the reciprocal of 2/7.",
            "The reciprocal is obtained by inverting the fraction, giving 7/2.",
            3100,
            vec![
                option("A", "2/7", false),
                option("B", "7/2", true),
                option("C", "5/7", false),
                option("D", "14/7", false),
            ],
        ),
        mcq(
            "A",
            "39",
            topics.equations,
            "If 2a + 3 = 17, find a.",
            "Subtract 3 to get 2a = 14, then divide by 2 to get a = 7.",
            3500,
            vec![
                option("A", "5", false),
                option("B", "6", false),
                option("C", "7", true),
                option("D", "8", false),
            ],
        ),
        mcq(
            "A",
            "40",
            topics.measurement,
            "Convert 1.8 litres to millilitres.",
            "1 litre = 1000 mL, so 1.8 litres = 1800 mL.",
            2800,
            vec![
                option("A", "180 mL", false),
                option("B", "1800 mL", true),
                option("C", "18,000 mL", false),
                option("D", "0.18 mL", false),
            ],
        ),
        theory(
            "B",
            "1",
            topics.ratios,
            "Divide GHC840 among Kofi, Ama and Yaw in the ratio 3:4:5.",
            "short_answer",
            6,
            5200,
            "Total parts = 3 + 4 + 5 = 12. One part = 840/12 = 70. Shares: Kofi = GHC210, Ama = GHC280, Yaw = GHC350.",
        ),
        theory(
            "B",
            "2",
            topics.equations,
            "A taxi charges a fixed amount of GHC6 plus GHC2 for each kilometre travelled. If Ama paid GHC26, how many kilometres did she travel?",
            "short_answer",
            5,
            5000,
            "Let the distance be x km. Then 6 + 2x = 26, so 2x = 20 and x = 10 km.",
        ),
        theory(
            "B",
            "3",
            topics.measurement,
            "A trapezium has parallel sides 10 cm and 16 cm and height 7 cm. Find its area.",
            "short_answer",
            5,
            5600,
            "Area of a trapezium = 1/2 x (sum of parallel sides) x height = 1/2 x (10 + 16) x 7 = 91 cm^2.",
        ),
        theory(
            "B",
            "4",
            topics.data,
            "The table below shows the number of books read by a group of students.\n1 book: 3 students\n2 books: 5 students\n3 books: 4 students\n4 books: 2 students\nFind the total number of students and the mean number of books read.",
            "short_answer",
            6,
            5700,
            "Total students = 3 + 5 + 4 + 2 = 14. Total books = (1 x 3) + (2 x 5) + (3 x 4) + (4 x 2) = 33. Mean = 33/14 = 2.36 books, correct to 2 decimal places.",
        ),
    ]
}

fn mcq(
    section_label: &str,
    question_number: &str,
    topic_id: i64,
    stem: &str,
    explanation_text: &str,
    difficulty_level: i64,
    options: Vec<AdminPastPaperOptionInput>,
) -> AdminPastPaperQuestionInput {
    AdminPastPaperQuestionInput {
        question_id: None,
        section_label: section_label.to_string(),
        question_number: Some(question_number.to_string()),
        topic_id,
        subtopic_id: None,
        stem: stem.to_string(),
        question_format: "mcq".to_string(),
        primary_pedagogic_function: None,
        explanation_text: Some(explanation_text.to_string()),
        difficulty_level: Some(difficulty_level),
        marks: Some(1),
        options,
    }
}

fn theory(
    section_label: &str,
    question_number: &str,
    topic_id: i64,
    stem: &str,
    question_format: &str,
    marks: i64,
    difficulty_level: i64,
    answer_text: &str,
) -> AdminPastPaperQuestionInput {
    AdminPastPaperQuestionInput {
        question_id: None,
        section_label: section_label.to_string(),
        question_number: Some(question_number.to_string()),
        topic_id,
        subtopic_id: None,
        stem: stem.to_string(),
        question_format: question_format.to_string(),
        primary_pedagogic_function: None,
        explanation_text: Some(answer_text.to_string()),
        difficulty_level: Some(difficulty_level),
        marks: Some(marks),
        options: Vec::new(),
    }
}

fn option(label: &str, text: &str, is_correct: bool) -> AdminPastPaperOptionInput {
    AdminPastPaperOptionInput {
        option_id: None,
        option_label: label.to_string(),
        option_text: text.to_string(),
        is_correct,
    }
}

fn storage_error(err: rusqlite::Error) -> CommandError {
    CommandError {
        code: "storage_error".to_string(),
        message: err.to_string(),
    }
}

struct ExistingPaper {
    paper_id: i64,
    question_ids: HashMap<String, i64>,
}

struct SeedPaperSpec {
    exam_year: i64,
    paper_code: &'static str,
    title: &'static str,
    questions: Vec<AdminPastPaperQuestionInput>,
}

struct SeedPaperResult {
    paper_id: i64,
    title: String,
}

struct SeedTopicIds {
    number_ops: i64,
    fractions: i64,
    ratios: i64,
    equations: i64,
    algebra: i64,
    powers: i64,
    measurement: i64,
    data: i64,
    probability: i64,
}
