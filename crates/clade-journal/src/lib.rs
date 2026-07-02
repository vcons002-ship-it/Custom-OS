//! clade-journal — the append-only action log behind Clade's reversibility
//! guarantee (docs/08-data-knowledge-model.md).
//!
//! M0 scope: the durable record and its invariants — append-only writes,
//! actor attribution, and the reversible/irreversible distinction with its
//! consent gate. The full undo *engine* (inverse replay against the
//! Substrate) lands at M6; the shape it needs is already here.
//!
//! Storage is JSONL in M0 (one fsync'd line per event). The move to a
//! WAL-mode SQLite store is internal to this crate.

use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

/// Who performed an action. The Journal never records an unattributed event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Actor {
    /// The human at the machine.
    User,
    /// The Cortex acting via Delegation or a Plan.
    Cortex,
}

/// Whether — and how — an event can be unwound.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub enum Reversibility {
    /// The inverse operation is recorded at write time; undo is a replay.
    Undoable { inverse: serde_json::Value },
    /// A one-way door. Cannot be journaled as executed without consent.
    Irreversible { consent: Option<Consent> },
}

/// The staged-and-confirmed record required before an irreversible event runs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Consent {
    /// What was shown to the user when they confirmed.
    pub staged_summary: String,
    /// Monotonic id of the confirmation itself.
    pub confirmed_seq: u64,
}

/// One journal event. `seq` is assigned by the journal, never the caller.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JournalEvent {
    pub seq: u64,
    pub actor: Actor,
    /// Capability id, e.g. `cap.image.adjust`, or a system verb for
    /// non-capability actions (`journal.consent`, `boot`).
    pub capability: String,
    /// Substrate item ids read by the action.
    pub inputs: Vec<String>,
    /// Substrate item ids created or modified.
    pub outputs: Vec<String>,
    pub reversibility: Reversibility,
    /// Which engine acted, when reasoning was involved (`local`, `ollama`,
    /// `gemini`); `None` for direct manipulation.
    pub engine: Option<String>,
}

/// Errors specific to the Journal's invariants.
#[derive(Debug)]
pub enum JournalError {
    /// An irreversible event reached the journal without a consent record.
    UnconsentedIrreversible(String),
}

impl std::fmt::Display for JournalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JournalError::UnconsentedIrreversible(cap) => write!(
                f,
                "irreversible event `{cap}` requires consent before it can be journaled as executed"
            ),
        }
    }
}
impl std::error::Error for JournalError {}

/// The append-only journal.
pub struct Journal {
    path: PathBuf,
    file: File,
    next_seq: u64,
}

impl Journal {
    /// Open (or create) the journal at `path`, recovering `next_seq` from the
    /// existing tail.
    pub fn open(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref().to_path_buf();
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let next_seq = match File::open(&path) {
            Ok(f) => BufReader::new(f)
                .lines()
                .map_while(Result::ok)
                .filter(|l| !l.trim().is_empty())
                .last()
                .map(|l| serde_json::from_str::<JournalEvent>(&l))
                .transpose()?
                .map(|e| e.seq + 1)
                .unwrap_or(0),
            Err(_) => 0,
        };
        let file = OpenOptions::new().create(true).append(true).open(&path)?;
        Ok(Self {
            path,
            file,
            next_seq,
        })
    }

    /// Append an event, enforcing the consent gate, and fsync it durable.
    /// Returns the assigned sequence number.
    pub fn append(
        &mut self,
        actor: Actor,
        capability: &str,
        inputs: Vec<String>,
        outputs: Vec<String>,
        reversibility: Reversibility,
        engine: Option<String>,
    ) -> anyhow::Result<u64> {
        if matches!(reversibility, Reversibility::Irreversible { consent: None }) {
            return Err(JournalError::UnconsentedIrreversible(capability.into()).into());
        }
        let event = JournalEvent {
            seq: self.next_seq,
            actor,
            capability: capability.into(),
            inputs,
            outputs,
            reversibility,
            engine,
        };
        let mut line = serde_json::to_vec(&event)?;
        line.push(b'\n');
        self.file.write_all(&line)?;
        self.file.sync_data()?;
        self.next_seq += 1;
        Ok(event.seq)
    }

    /// Read the full log, oldest first. (Iteration windows come with the
    /// SQLite store; Phase-1 logs are small.)
    pub fn events(&self) -> anyhow::Result<Vec<JournalEvent>> {
        let file = File::open(&self.path)?;
        BufReader::new(file)
            .lines()
            .map_while(Result::ok)
            .filter(|l| !l.trim().is_empty())
            .map(|l| Ok(serde_json::from_str(&l)?))
            .collect()
    }

    /// The inverse operation recorded for `seq`, if that event is undoable.
    /// M6's undo engine replays this against the Substrate.
    pub fn inverse_of(&self, seq: u64) -> anyhow::Result<Option<serde_json::Value>> {
        Ok(self
            .events()?
            .into_iter()
            .find(|e| e.seq == seq)
            .and_then(|e| match e.reversibility {
                Reversibility::Undoable { inverse } => Some(inverse),
                Reversibility::Irreversible { .. } => None,
            }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn scratch() -> (tempfile::TempDir, Journal) {
        let dir = tempfile::tempdir().unwrap();
        let journal = Journal::open(dir.path().join("journal.jsonl")).unwrap();
        (dir, journal)
    }

    #[test]
    fn appends_are_sequenced_and_durable() {
        let (dir, mut j) = scratch();
        let a = j
            .append(
                Actor::User,
                "cap.image.adjust",
                vec!["sub:item/ph_1".into()],
                vec!["sub:item/ph_1".into()],
                Reversibility::Undoable {
                    inverse: json!({"op": "restore", "hash": "blake3:x"}),
                },
                None,
            )
            .unwrap();
        let b = j
            .append(
                Actor::Cortex,
                "cap.image.collage",
                vec!["sub:item/ph_1".into()],
                vec!["sub:item/co_1".into()],
                Reversibility::Undoable {
                    inverse: json!({"op": "remove-derived", "target": "sub:item/co_1"}),
                },
                Some("local".into()),
            )
            .unwrap();
        assert_eq!((a, b), (0, 1));

        // Reopen: the tail recovers the sequence.
        drop(j);
        let mut j = Journal::open(dir.path().join("journal.jsonl")).unwrap();
        let c = j
            .append(
                Actor::User,
                "cap.image.annotate",
                vec![],
                vec![],
                Reversibility::Undoable {
                    inverse: json!({"op": "noop"}),
                },
                None,
            )
            .unwrap();
        assert_eq!(c, 2);
        assert_eq!(j.events().unwrap().len(), 3);
    }

    #[test]
    fn irreversible_without_consent_is_refused() {
        let (_dir, mut j) = scratch();
        let err = j
            .append(
                Actor::Cortex,
                "cap.share.send",
                vec![],
                vec![],
                Reversibility::Irreversible { consent: None },
                Some("local".into()),
            )
            .unwrap_err();
        assert!(err.to_string().contains("requires consent"));
        assert!(
            j.events().unwrap().is_empty(),
            "refused events must not be written"
        );
    }

    #[test]
    fn irreversible_with_consent_is_recorded() {
        let (_dir, mut j) = scratch();
        j.append(
            Actor::Cortex,
            "cap.share.send",
            vec!["sub:item/co_1".into()],
            vec![],
            Reversibility::Irreversible {
                consent: Some(Consent {
                    staged_summary: "send collage to Mom".into(),
                    confirmed_seq: 0,
                }),
            },
            Some("local".into()),
        )
        .unwrap();
        assert_eq!(j.events().unwrap().len(), 1);
        assert_eq!(
            j.inverse_of(0).unwrap(),
            None,
            "one-way doors have no inverse"
        );
    }

    #[test]
    fn undo_finds_the_recorded_inverse() {
        let (_dir, mut j) = scratch();
        let inverse = json!({"op": "restore", "hash": "blake3:before"});
        let seq = j
            .append(
                Actor::User,
                "cap.image.adjust",
                vec![],
                vec![],
                Reversibility::Undoable {
                    inverse: inverse.clone(),
                },
                None,
            )
            .unwrap();
        assert_eq!(j.inverse_of(seq).unwrap(), Some(inverse));
    }
}
