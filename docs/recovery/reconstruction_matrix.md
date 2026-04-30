# Reconstruction Matrix

Use this matrix to convert conversations, screenshots, and recovered files into backend contracts.

| Evidence source | Feature or slice | Role | Workflow described | Terms or labels used | Backend modules touched | Open questions | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `ideas/idea35.txt` | Rebuild discipline and recovery safety | Super Admin, Developer | Recover, document, rebuild in slices | rebuild master, truth pack, recovery snapshot | `storage`, `commands`, docs | none | active |
| Conversation thread | Dashboard, Coach activation, real data recording | Student, Coach, Developer | Rebuild homepage as learner cockpit; Activate Coach from top bar; auto-advance questions; wrong-answer weakness popup; record all interactions | Activate Coach, CoachHub mode, solidify it, real questions, heatmaps, misconception engine | `student`, `practice`, `diagnostic`, `memory`, `misconception`, `mastery` | Exact final top-bar utilities and popup flow | mapped in `docs/recovery/forensic_product_map.md` |
| Screenshot | Older dashboard visual structure | Student | Left rail, top utility bar, readiness metrics, action cards, subject health, right trail/feed | Readiness, streak, Your Trail, Start Quiz, Mock Centre, Topic Clinic | `frontend/src/layouts`, `frontend/src/pages/student` | Final version differed with nested outline rings and right-side live question | mapped in `docs/recovery/forensic_product_map.md` |
| Recovered file | Vue/Rust recovered product DNA | Admin, Student, Coach, Developer | Question studio, question families, key concept quiz, misconception graph, Topic Clinic, prep planning, memory, mock review, elite metrics | family, variant, remediation, misconception, proof contract, decay, readiness, elite | `questions`, `content`, `practice`, `memory`, `misconception`, `mastery`, `coach` | Which medium-confidence recovered fragments should be rebuilt first | mapped in `docs/recovery/forensic_product_map.md` |

## Usage Rule

Do not start implementation from vague memory when this matrix can capture the contract first.
