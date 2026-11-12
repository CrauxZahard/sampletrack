//! The app's data: people, product samples, notifications and the defaults a first launch
//! starts from. Everything here is plain data, so a server-backed [`Repository`](crate::store)
//! can hand the same shapes over HTTP later without the views changing.

#[cfg(not(target_arch = "wasm32"))]
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// A permission a person can hold. Input and update can be combined (Leader QC), Resample is
/// the QCI step that closes a resampling request, admin covers every other scope, and read-only
/// is the empty set.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    Input,
    Update,
    Resample,
    ReadOnly,
    Admin,
}

impl Scope {
    pub const ALL: [Scope; 5] = [
        Scope::Input,
        Scope::Update,
        Scope::Resample,
        Scope::ReadOnly,
        Scope::Admin,
    ];

    pub fn short(self) -> &'static str {
        match self {
            Scope::Input => "Input",
            Scope::Update => "Update",
            Scope::Resample => "Resample",
            Scope::ReadOnly => "Read-only",
            Scope::Admin => "Admin",
        }
    }
}

/// The scopes one person holds. Stored as flags, so input + update is a normal person (Leader
/// QC), not a special role.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scopes {
    #[serde(default)]
    pub input: bool,
    #[serde(default)]
    pub update: bool,
    #[serde(default)]
    pub resample: bool,
    #[serde(default)]
    pub admin: bool,
}

impl Scopes {
    pub const READ_ONLY: Scopes = Scopes {
        input: false,
        update: false,
        resample: false,
        admin: false,
    };

    pub const fn input() -> Scopes {
        Scopes {
            input: true,
            update: false,
            resample: false,
            admin: false,
        }
    }

    pub const fn update() -> Scopes {
        Scopes {
            input: false,
            update: true,
            resample: false,
            admin: false,
        }
    }

    pub const fn resample() -> Scopes {
        Scopes {
            input: false,
            update: false,
            resample: true,
            admin: false,
        }
    }

    pub const fn input_and_update() -> Scopes {
        Scopes {
            input: true,
            update: true,
            resample: false,
            admin: false,
        }
    }

    pub const fn admin() -> Scopes {
        Scopes {
            input: false,
            update: false,
            resample: false,
            admin: true,
        }
    }

    pub fn can_input(self) -> bool {
        self.input || self.admin
    }

    pub fn can_update(self) -> bool {
        self.update || self.admin
    }

    pub fn can_resample(self) -> bool {
        self.resample || self.admin
    }

    pub fn is_admin(self) -> bool {
        self.admin
    }

    pub fn is_read_only(self) -> bool {
        !self.input && !self.update && !self.resample && !self.admin
    }

    /// The scope a toggle shows as held: admin covers input, update and resample.
    pub fn holds(self, scope: Scope) -> bool {
        match scope {
            Scope::Input => self.can_input(),
            Scope::Update => self.can_update(),
            Scope::Resample => self.can_resample(),
            Scope::ReadOnly => self.is_read_only(),
            Scope::Admin => self.is_admin(),
        }
    }

    pub fn toggle(&mut self, scope: Scope) {
        match scope {
            Scope::Input => self.input = !self.input,
            Scope::Update => self.update = !self.update,
            Scope::Resample => self.resample = !self.resample,
            Scope::ReadOnly => *self = Scopes::READ_ONLY,
            Scope::Admin => self.admin = !self.admin,
        }
    }

    /// "Input sample + Update sample", "Read-only" or "Admin".
    pub fn summary(self) -> String {
        if self.admin {
            return "Akses penuh".into();
        }
        let mut parts = Vec::new();
        if self.input {
            parts.push("Input sample");
        }
        if self.update {
            parts.push("Update sample");
        }
        if self.resample {
            parts.push("Resample sample");
        }
        if parts.is_empty() {
            "Read-only".into()
        } else {
            parts.join(" + ")
        }
    }
}

/// Where a sample sits in the production stage.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    #[default]
    Granule,
    Core,
    Coated,
}

impl Stage {
    pub const ALL: [Stage; 3] = [Stage::Granule, Stage::Core, Stage::Coated];

    pub fn label(self) -> &'static str {
        match self {
            Stage::Granule => "Granule",
            Stage::Core => "Core",
            Stage::Coated => "Coated",
        }
    }
}

/// Where a sample sits in the review. A sample starts [`Status::Unanalyzed`]. A first decision
/// can land on an intermediate state ([`Status::WaitingHuds`], [`Status::Resampling`]) or a
/// terminal one; intermediate states have one more step (HUDS approval for the Leader QC, or a
/// resample-complete mark for QCI). Terminal states cannot move again.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    #[default]
    #[serde(rename = "unanalyzed")]
    Unanalyzed,
    Approved,
    #[serde(rename = "approved_huds")]
    ApprovedHuds,
    #[serde(rename = "waiting_huds")]
    WaitingHuds,
    Resampling,
    Resampled,
    #[serde(rename = "not_qualified")]
    NotQualified,
}

impl Status {
    #[cfg(test)]
    pub const ALL: [Status; 7] = [
        Status::Unanalyzed,
        Status::Approved,
        Status::ApprovedHuds,
        Status::WaitingHuds,
        Status::Resampling,
        Status::Resampled,
        Status::NotQualified,
    ];
    /// The outcomes a first analysis can land on. Unanalyzed is only the starting state.
    pub const DECISIONS: [Status; 4] = [
        Status::Approved,
        Status::WaitingHuds,
        Status::Resampling,
        Status::NotQualified,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Status::Unanalyzed => "Belum Analisa",
            Status::Approved => "Approved",
            Status::ApprovedHuds => "Approved dengan HUDS",
            Status::WaitingHuds => "Menunggu Approval HUDS",
            Status::Resampling => "Perlu Resample",
            Status::Resampled => "Sudah Resample",
            Status::NotQualified => "Tidak Memenuhi Syarat",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Status::Unanalyzed => "Sample baru masuk, belum dianalisa.",
            Status::Approved => "Sample lolos approval dan siap dipakai.",
            Status::ApprovedHuds => "Sample disetujui setelah review HUDS.",
            Status::WaitingHuds => "Dokumen sudah dikirim, menunggu review HUDS.",
            Status::Resampling => "Sample perlu diambil dan diuji ulang.",
            Status::Resampled => "Sample sudah diambil ulang dan selesai diresample.",
            Status::NotQualified => "Sample tidak lolos syarat dan dihentikan.",
        }
    }

    pub fn rank(self) -> u8 {
        match self {
            Status::Unanalyzed => 0,
            Status::WaitingHuds => 1,
            Status::Resampling => 2,
            Status::Resampled => 3,
            Status::ApprovedHuds => 4,
            Status::Approved => 5,
            Status::NotQualified => 6,
        }
    }

    /// The steps this status still allows. Empty for terminal statuses.
    pub fn transitions(self) -> &'static [Status] {
        match self {
            Status::Unanalyzed => &Self::DECISIONS,
            Status::WaitingHuds => &[Status::ApprovedHuds, Status::NotQualified],
            Status::Resampling => &[Status::Resampled],
            _ => &[],
        }
    }

    /// A terminal state: no further status, notification or remark.
    pub fn is_final(self) -> bool {
        matches!(
            self,
            Status::Approved | Status::ApprovedHuds | Status::Resampled | Status::NotQualified
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub name: String,
    #[serde(default)]
    pub scopes: Scopes,
    /// The single role older builds stored; [`Data::migrate`] turns it into scopes.
    #[serde(default, rename = "role", skip_serializing)]
    pub legacy_role: String,
}

impl User {
    /// The monogram in a person's avatar. An uppercase acronym is kept whole, so Leader QC
    /// shows QC, QA shows QA, QCI shows QCI and Helper shows H.
    pub fn initials(&self) -> String {
        let words: Vec<&str> = self.name.split_whitespace().collect();
        let acronym = |word: &str| word.len() <= 4 && word.chars().all(|c| c.is_ascii_uppercase());
        if words.len() >= 2 {
            if let Some(last) = words.last().filter(|w| acronym(w)) {
                return (*last).to_owned();
            }
        }
        if words.len() == 1 {
            let word = words[0];
            if acronym(word) {
                return word.to_owned();
            }
            return word
                .chars()
                .next()
                .map(|c| c.to_uppercase().collect())
                .unwrap_or_else(|| "?".into());
        }
        let mut out = String::new();
        for word in words.iter().take(2) {
            if let Some(c) = word.chars().next() {
                out.extend(c.to_uppercase());
            }
        }
        if out.is_empty() { "?".to_owned() } else { out }
    }
}

/// One comment in a sample's remarks thread. Anyone with a writable scope can add one, and a
/// sample keeps them all, like a cell's comments in a spreadsheet.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Remark {
    pub id: u64,
    pub by: u64,
    pub text: String,
    pub at: u64,
}

/// One product sample and the two audit facts the list shows: who entered it and who last
/// changed its status. `remarks` is the append-only comment thread.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sample {
    pub id: u64,
    pub product: String,
    pub batch: String,
    #[serde(default)]
    pub stage: Stage,
    pub status: Status,
    pub input_by: u64,
    pub input_at: u64,
    pub updated_by: u64,
    pub updated_at: u64,
    #[serde(default)]
    pub remarks: Vec<Remark>,
    /// The single note older builds stored. [`Data::migrate`] turns it into the first remark.
    #[serde(default, rename = "note", skip_serializing)]
    pub legacy_note: String,
}

impl Sample {
    pub fn latest_remark(&self) -> Option<&Remark> {
        self.remarks.last()
    }

    pub fn remark_count(&self) -> usize {
        self.remarks.len()
    }
}

/// A bulk update delivered to the people the sender picked.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Notice {
    pub id: u64,
    pub from: u64,
    pub to: Vec<u64>,
    pub title: String,
    pub body: String,
    pub at: u64,
    #[serde(default)]
    pub read_by: Vec<u64>,
    #[serde(default)]
    pub batch: String,
    /// The status the update landed on, for inbox filters. Older notices have none.
    #[serde(default)]
    pub status: Option<Status>,
    #[serde(default)]
    pub stage: Option<Stage>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Data {
    #[serde(default)]
    pub version: u32,
    pub users: Vec<User>,
    pub samples: Vec<Sample>,
    #[serde(default)]
    pub notices: Vec<Notice>,
    pub current_user: u64,
    pub next_user: u64,
    pub next_sample: u64,
    pub next_notice: u64,
    #[serde(default)]
    pub next_remark: u64,
    #[serde(default)]
    pub dark: Option<bool>,
}

#[cfg(not(target_arch = "wasm32"))]
pub fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(target_arch = "wasm32")]
pub fn now_secs() -> u64 {
    (js_sys::Date::now() / 1000.0) as u64
}

impl Data {
    /// The demo workspace: five people with different scopes and twelve samples spread across
    /// the three approval states.
    pub fn seed(now: u64) -> Data {
        let users = vec![
            User {
                id: 1,
                name: "Admin".into(),
                scopes: Scopes::admin(),
                legacy_role: String::new(),
            },
            User {
                id: 2,
                name: "Helper".into(),
                scopes: Scopes::input(),
                legacy_role: String::new(),
            },
            User {
                id: 3,
                name: "QA".into(),
                scopes: Scopes::READ_ONLY,
                legacy_role: String::new(),
            },
            User {
                id: 4,
                name: "Leader QC".into(),
                scopes: Scopes::input_and_update(),
                legacy_role: String::new(),
            },
            User {
                id: 5,
                name: "QCI".into(),
                scopes: Scopes::resample(),
                legacy_role: String::new(),
            },
        ];
        let hour = 3600;
        let day = 24 * hour;
        let rows = [
            (
                "Paracetamol 500 mg",
                "MO-2611-018",
                Stage::Coated,
                Status::Approved,
                2,
                4,
                2 * hour,
                "Hasil uji disolusi sesuai spesifikasi.",
            ),
            (
                "Amoxicillin 500 mg",
                "MO-2611-022",
                Stage::Granule,
                Status::WaitingHuds,
                2,
                4,
                5 * hour,
                "Dokumen sudah dikirim ke HUDS.",
            ),
            (
                "Vitamin C 1000 mg",
                "MO-2612-004",
                Stage::Granule,
                Status::Resampling,
                2,
                1,
                8 * hour,
                "Kadar aktif di bawah batas; ambil ulang.",
            ),
            (
                "Ibuprofen 400 mg",
                "MO-2612-009",
                Stage::Coated,
                Status::ApprovedHuds,
                2,
                4,
                day,
                "Lolos uji stabilitas; disetujui setelah review HUDS.",
            ),
            (
                "Cetirizine 10 mg",
                "MO-2612-011",
                Stage::Core,
                Status::WaitingHuds,
                2,
                4,
                day + 3 * hour,
                "Menunggu tanda tangan supervisor.",
            ),
            (
                "Omeprazole 20 mg",
                "MO-2612-015",
                Stage::Granule,
                Status::Approved,
                4,
                4,
                2 * day,
                "Dokumen lengkap.",
            ),
            (
                "Metformin 500 mg",
                "MO-2612-017",
                Stage::Coated,
                Status::Resampled,
                2,
                5,
                2 * day + 4 * hour,
                "Sudah diresample ulang dan diserahkan ke QCI.",
            ),
            (
                "Salbutamol Inhaler",
                "MO-2613-001",
                Stage::Core,
                Status::Unanalyzed,
                4,
                4,
                3 * day,
                "Sample baru diterima dari line 2; menunggu analisa.",
            ),
            (
                "Simvastatin 20 mg",
                "MO-2613-006",
                Stage::Coated,
                Status::NotQualified,
                2,
                4,
                3 * day + 2 * hour,
                "Tidak memenuhi syarat; kadar aktif di luar spesifikasi.",
            ),
            (
                "Loratadine 10 mg",
                "MO-2613-008",
                Stage::Core,
                Status::Approved,
                4,
                4,
                4 * day,
                "Hasil uji kandungan seragam.",
            ),
            (
                "Ranitidine 150 mg",
                "MO-2613-012",
                Stage::Granule,
                Status::Resampling,
                2,
                1,
                5 * day,
                "Perlu pengambilan ulang dari line 3.",
            ),
            (
                "Dexamethasone 0.5 mg",
                "MO-2613-014",
                Stage::Core,
                Status::Unanalyzed,
                2,
                2,
                6 * day,
                "Sample baru diterima; belum dianalisa.",
            ),
        ];
        let mut samples = Vec::with_capacity(rows.len());
        let mut next_remark = 1;
        for (i, (product, batch, stage, status, input_by, updated_by, ago, text)) in
            rows.into_iter().enumerate()
        {
            let at = now.saturating_sub(ago);
            let input_at = if status != Status::Unanalyzed {
                at.saturating_sub(6 * hour)
            } else {
                at
            };
            samples.push(Sample {
                id: (i + 1) as u64,
                product: product.into(),
                batch: batch.into(),
                stage,
                status,
                input_by,
                input_at,
                updated_by,
                updated_at: at,
                remarks: vec![Remark {
                    id: next_remark,
                    by: updated_by,
                    text: text.into(),
                    at,
                }],
                legacy_note: String::new(),
            });
            next_remark += 1;
        }
        // A few threads carry more than one voice, so the remarks column shows its count and
        // the dialog shows a real conversation out of the box.
        let extra = [
            (
                0,
                2,
                now.saturating_sub(hour),
                "Retest disolusi selesai; hasilnya tetap sesuai.",
            ),
            (
                1,
                1,
                now.saturating_sub(2 * hour),
                "Dokumen tambahan sudah diunggah ke HUDS.",
            ),
            (
                2,
                2,
                now.saturating_sub(6 * hour),
                "Pengambilan ulang dijadwalkan besok pagi.",
            ),
        ];
        for (sample, by, at, text) in extra {
            samples[sample].remarks.push(Remark {
                id: next_remark,
                by,
                text: text.into(),
                at,
            });
            next_remark += 1;
        }
        let notices = vec![
            Notice {
                id: 1,
                from: 4,
                to: vec![1, 2, 3, 5],
                title: "Sample diperbarui".into(),
                body: "Leader QC memperbarui Amoxicillin 500 mg (MO-2611-022) → Menunggu Approval HUDS."
                    .into(),
                at: now.saturating_sub(3 * hour),
                read_by: vec![2],
                batch: "MO-2611-022".into(),
                status: Some(Status::WaitingHuds),
                stage: Some(Stage::Granule),
            },
            Notice {
                id: 2,
                from: 2,
                to: vec![1],
                title: "Sample baru ditambahkan".into(),
                body: "Helper menambahkan Paracetamol 500 mg (MO-2611-018).".into(),
                at: now.saturating_sub(12 * hour),
                read_by: vec![1],
                batch: "MO-2611-018".into(),
                status: Some(Status::Unanalyzed),
                stage: Some(Stage::Coated),
            },
            Notice {
                id: 3,
                from: 4,
                to: vec![1, 2, 3, 5],
                title: "Sample perlu resample".into(),
                body: "Leader QC menandai Vitamin C 1000 mg (MO-2612-004) → Perlu Resample."
                    .into(),
                at: now.saturating_sub(hour),
                read_by: Vec::new(),
                batch: "MO-2612-004".into(),
                status: Some(Status::Resampling),
                stage: Some(Stage::Granule),
            },
            Notice {
                id: 4,
                from: 4,
                to: vec![1, 2, 3, 5],
                title: "Approved dengan HUDS".into(),
                body: "Leader QC menyetujui Ibuprofen 400 mg (MO-2612-009) setelah review HUDS."
                    .into(),
                at: now.saturating_sub(20 * hour),
                read_by: vec![1, 2],
                batch: "MO-2612-009".into(),
                status: Some(Status::ApprovedHuds),
                stage: Some(Stage::Coated),
            },
            Notice {
                id: 5,
                from: 5,
                to: vec![1, 2, 3, 4],
                title: "Resample selesai".into(),
                body: "QCI menandai Metformin 500 mg (MO-2612-017) → Sudah Resample.".into(),
                at: now.saturating_sub(20 * 60),
                read_by: Vec::new(),
                batch: "MO-2612-017".into(),
                status: Some(Status::Resampled),
                stage: Some(Stage::Coated),
            },
            Notice {
                id: 6,
                from: 4,
                to: vec![1, 2, 3, 5],
                title: "Tidak memenuhi syarat".into(),
                body: "Leader QC menandai Simvastatin 20 mg (MO-2613-006) → Tidak Memenuhi Syarat."
                    .into(),
                at: now.saturating_sub(2 * hour),
                read_by: Vec::new(),
                batch: "MO-2613-006".into(),
                status: Some(Status::NotQualified),
                stage: Some(Stage::Coated),
            },
            Notice {
                id: 7,
                from: 4,
                to: vec![1, 2, 3, 5],
                title: "Sample disetujui".into(),
                body: "Leader QC menyetujui Paracetamol 500 mg (MO-2611-018) → Approved.".into(),
                at: now.saturating_sub(8 * hour),
                read_by: vec![1, 2],
                batch: "MO-2611-018".into(),
                status: Some(Status::Approved),
                stage: Some(Stage::Coated),
            },
        ];
        Data {
            version: 3,
            users,
            samples,
            notices,
            current_user: 1,
            next_user: 6,
            next_sample: 13,
            next_notice: 8,
            next_remark,
            dark: None,
        }
    }

    /// Turns workspaces saved before remarks were a thread into the current shape: the single
    /// `note` becomes the first remark, and the id counter continues past every existing one.
    pub fn migrate(&mut self) {
        for user in &mut self.users {
            if !user.legacy_role.is_empty() {
                user.scopes = match user.legacy_role.as_str() {
                    "input" => Scopes::input(),
                    "update" => Scopes::update(),
                    "admin" => Scopes::admin(),
                    _ => Scopes::READ_ONLY,
                };
                user.legacy_role.clear();
            }
        }

        let highest = self
            .samples
            .iter()
            .flat_map(|s| s.remarks.iter().map(|r| r.id))
            .max()
            .unwrap_or(0);
        let mut next = self.next_remark.max(highest + 1).max(1);
        for sample in &mut self.samples {
            if sample.remarks.is_empty() && !sample.legacy_note.trim().is_empty() {
                let text = std::mem::take(&mut sample.legacy_note);
                let by = sample.updated_by;
                let at = sample.updated_at;
                sample.remarks.push(Remark {
                    id: next,
                    by,
                    text,
                    at,
                });
                next += 1;
            }
            sample.legacy_note.clear();
        }
        self.next_remark = next;
    }

    pub fn user(&self, id: u64) -> Option<&User> {
        self.users.iter().find(|u| u.id == id)
    }

    pub fn user_mut(&mut self, id: u64) -> Option<&mut User> {
        self.users.iter_mut().find(|u| u.id == id)
    }

    pub fn sample(&self, id: u64) -> Option<&Sample> {
        self.samples.iter().find(|s| s.id == id)
    }

    pub fn sample_mut(&mut self, id: u64) -> Option<&mut Sample> {
        self.samples.iter_mut().find(|s| s.id == id)
    }

    pub fn admin_count(&self) -> usize {
        self.users.iter().filter(|u| u.scopes.is_admin()).count()
    }

    pub fn unread_for(&self, user: u64) -> usize {
        self.notices
            .iter()
            .filter(|n| n.to.contains(&user) && !n.read_by.contains(&user))
            .count()
    }

    /// Newest first, as the notification page shows them.
    pub fn notices_for(&self, user: u64) -> Vec<&Notice> {
        let mut rows: Vec<&Notice> = self
            .notices
            .iter()
            .filter(|n| n.to.contains(&user))
            .collect();
        rows.sort_by(|a, b| b.at.cmp(&a.at));
        rows
    }
}

/// A short, human reading of an age: what a notification list needs.
pub fn ago(now: u64, at: u64) -> String {
    let d = now.saturating_sub(at);
    if d < 60 {
        "baru saja".into()
    } else if d < 3_600 {
        format!("{} mnt lalu", d / 60)
    } else if d < 86_400 {
        format!("{} jam lalu", d / 3_600)
    } else if d < 7 * 86_400 {
        format!("{} hari lalu", d / 86_400)
    } else {
        format!("{} minggu lalu", d / (7 * 86_400))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_covers_every_status_and_the_five_people() {
        let d = Data::seed(1_000_000);
        assert_eq!(d.users.len(), 5);
        assert_eq!(d.samples.len(), 12);
        for status in Status::ALL {
            assert!(
                d.samples.iter().any(|s| s.status == status),
                "seed has no {status:?} sample"
            );
        }
        assert_eq!(d.users.iter().filter(|u| u.scopes.can_input()).count(), 3);
        assert_eq!(d.users.iter().filter(|u| u.scopes.can_update()).count(), 2);
        assert_eq!(
            d.users.iter().filter(|u| u.scopes.can_resample()).count(),
            2
        );
        assert_eq!(
            d.users.iter().filter(|u| u.scopes.is_read_only()).count(),
            1
        );
        let leader = d.users.iter().find(|u| u.name == "Leader QC").unwrap();
        assert!(leader.scopes.input && leader.scopes.update && !leader.scopes.admin);
        let qci = d.users.iter().find(|u| u.name == "QCI").unwrap();
        assert!(qci.scopes.resample && !qci.scopes.is_read_only());
        assert!(d.samples.iter().all(|s| !s.remarks.is_empty()));
        assert!(d.samples.iter().any(|s| s.remarks.len() > 1));
    }

    #[test]
    fn statuses_only_move_along_the_review_graph() {
        assert!(Status::Unanalyzed.transitions().contains(&Status::Approved));
        assert_eq!(
            Status::WaitingHuds.transitions(),
            &[Status::ApprovedHuds, Status::NotQualified]
        );
        assert_eq!(Status::Resampling.transitions(), &[Status::Resampled]);
        assert!(
            Status::WaitingHuds
                .transitions()
                .contains(&Status::NotQualified)
        );
        assert!(!Status::Unanalyzed.is_final());
        assert!(!Status::WaitingHuds.is_final());
        assert!(!Status::Resampling.is_final());
        assert!(Status::Approved.is_final());
        assert!(Status::ApprovedHuds.is_final());
        assert!(Status::Resampled.is_final());
        assert!(Status::NotQualified.is_final());
        assert!(Status::Approved.transitions().is_empty());
        assert!(Status::Resampled.transitions().is_empty());
    }

    #[test]
    fn scopes_can_combine_and_read_only_is_the_empty_set() {
        assert!(!Scopes::READ_ONLY.can_input());
        assert!(!Scopes::READ_ONLY.can_update());
        assert!(!Scopes::READ_ONLY.can_resample());
        assert!(Scopes::admin().can_input() && Scopes::admin().can_update());
        assert!(Scopes::admin().can_resample());
        assert!(Scopes::input().can_input() && !Scopes::input().can_update());
        assert!(!Scopes::update().can_input() && Scopes::update().can_update());
        assert!(Scopes::resample().can_resample() && !Scopes::resample().can_update());
        assert!(!Scopes::resample().is_read_only());
        assert_eq!(Scopes::resample().summary(), "Resample sample");
        let leader = Scopes::input_and_update();
        assert!(leader.can_input() && leader.can_update());
        assert_eq!(leader.summary(), "Input sample + Update sample");
    }

    #[test]
    fn avatars_keep_role_acronyms_whole() {
        let user = |name: &str| User {
            id: 1,
            name: name.into(),
            scopes: Scopes::READ_ONLY,
            legacy_role: String::new(),
        };
        assert_eq!(user("Leader QC").initials(), "QC");
        assert_eq!(user("QA").initials(), "QA");
        assert_eq!(user("QCI").initials(), "QCI");
        assert_eq!(user("Helper").initials(), "H");
        assert_eq!(user("Admin").initials(), "A");
        assert_eq!(user("Nabila Putri").initials(), "NP");
    }

    #[test]
    fn ages_read_as_short_intervals() {
        assert_eq!(ago(100, 100), "baru saja");
        assert_eq!(ago(3_700, 100), "1 jam lalu");
        assert_eq!(ago(2 * 86_400, 0), "2 hari lalu");
    }
}
