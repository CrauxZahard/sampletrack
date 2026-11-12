//! App state and the actions views call. The app owns the whole workspace in memory, writes
//! through [`Repository`] after each committed change, and keeps unsaved form text in
//! [`App`]'s `TextEdit`s.

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashSet;

use rune::{TextEdit, Tx};

use crate::model::{Data, Notice, Remark, Sample, Scope, Scopes, Stage, Status, now_secs};
#[cfg(test)]
use crate::store::Memory;
use crate::store::{LocalJson, Repository};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Samples,
    People,
    Notices,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Filter {
    All,
    Unanalyzed,
    Approved,
    Waiting,
    Resampling,
}

impl Filter {
    pub const ALL: [Filter; 5] = [
        Filter::All,
        Filter::Unanalyzed,
        Filter::Approved,
        Filter::Waiting,
        Filter::Resampling,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Filter::All => "Semua",
            Filter::Unanalyzed => "Belum analisa",
            Filter::Approved => "Approved",
            Filter::Waiting => "Menunggu HUDS",
            Filter::Resampling => "Resampling",
        }
    }

    fn matches(self, status: Status) -> bool {
        match self {
            Filter::All => true,
            Filter::Unanalyzed => status == Status::Unanalyzed,
            Filter::Approved => status == Status::Approved,
            Filter::Waiting => status == Status::WaitingHuds,
            Filter::Resampling => status == Status::Resampling,
        }
    }
}

/// Which notices the inbox shows. Status filters make the "resampling" queue one click for
/// QCI and the other readers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoticeFilter {
    All,
    Unread,
    Unanalyzed,
    Approved,
    Waiting,
    Resampling,
}

impl NoticeFilter {
    pub const ALL: [NoticeFilter; 6] = [
        NoticeFilter::All,
        NoticeFilter::Unread,
        NoticeFilter::Unanalyzed,
        NoticeFilter::Approved,
        NoticeFilter::Waiting,
        NoticeFilter::Resampling,
    ];

    pub fn label(self) -> &'static str {
        match self {
            NoticeFilter::All => "Semua",
            NoticeFilter::Unread => "Belum dibaca",
            NoticeFilter::Unanalyzed => "Belum analisa",
            NoticeFilter::Approved => "Approved",
            NoticeFilter::Waiting => "Menunggu HUDS",
            NoticeFilter::Resampling => "Resampling",
        }
    }

    fn matches(self, notice: &Notice, me: u64) -> bool {
        match self {
            NoticeFilter::All => true,
            NoticeFilter::Unread => !notice.read_by.contains(&me),
            NoticeFilter::Unanalyzed => notice.status == Some(Status::Unanalyzed),
            NoticeFilter::Approved => notice.status == Some(Status::Approved),
            NoticeFilter::Waiting => notice.status == Some(Status::WaitingHuds),
            NoticeFilter::Resampling => notice.status == Some(Status::Resampling),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Counts {
    pub total: usize,
    pub unanalyzed: usize,
    pub approved: usize,
    pub waiting: usize,
    pub resampling: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct QueryKey {
    search: String,
    filter: Filter,
    sort: SortKey,
    desc: bool,
}

struct ViewCache {
    data_version: u64,
    query: QueryKey,
    indices: Vec<usize>,
    generation: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortKey {
    Updated,
    Product,
    Batch,
    Status,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dialog {
    None,
    NewSample,
    Sample(u64),
    SwitchUser,
    NewUser,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    Success,
    Info,
    Warning,
}

#[derive(Clone, Debug)]
pub struct ToastNote {
    pub title: String,
    pub body: String,
    pub tone: Tone,
}

pub struct App {
    pub data: Data,
    pub repo: Box<dyn Repository>,

    pub page: Page,
    pub dialog: Dialog,
    pub open_select: Option<String>,

    pub search: TextEdit,
    pub filter: Filter,
    pub sort: SortKey,
    pub sort_desc: bool,

    /// New-sample form.
    pub new_product: TextEdit,
    pub new_batch: TextEdit,
    pub new_stage: Stage,
    pub new_remarks: TextEdit,
    /// The remark the update dialog is about to add.
    pub remark_draft: TextEdit,
    pub edit_status: Status,
    pub edit_id: Option<u64>,
    pub recipients: HashSet<u64>,

    /// New-member form.
    pub new_name: TextEdit,
    pub new_scopes: Scopes,

    pub notice_filter: NoticeFilter,
    pub toast: Option<ToastNote>,
    toast_at: u64,

    data_version: u64,
    counts_version: u64,
    view_cache: RefCell<ViewCache>,
    counts_cache: RefCell<(u64, Counts)>,
}

impl App {
    pub fn boot() -> App {
        let repo = LocalJson::open();
        let mut data = repo
            .load()
            .filter(|d| d.version >= 2 && !d.users.is_empty() && d.user(d.current_user).is_some())
            .unwrap_or_else(|| Data::seed(now_secs()));
        data.migrate();
        App::with_repo(data, Box::new(repo))
    }

    /// A workspace backed by memory, for tests.
    #[cfg(test)]
    pub fn preview(data: Data) -> App {
        App::with_repo(data, Box::new(Memory))
    }

    fn with_repo(data: Data, repo: Box<dyn Repository>) -> App {
        App {
            data,
            repo,
            page: Page::Samples,
            dialog: Dialog::None,
            open_select: None,
            search: TextEdit::new(""),
            filter: Filter::All,
            sort: SortKey::Updated,
            sort_desc: true,
            new_product: TextEdit::new(""),
            new_batch: TextEdit::new(""),
            new_stage: Stage::Granule,
            new_remarks: TextEdit::new(""),
            remark_draft: TextEdit::new(""),
            edit_status: Status::Approved,
            edit_id: None,
            recipients: HashSet::new(),
            new_name: TextEdit::new(""),
            new_scopes: Scopes::input(),
            notice_filter: NoticeFilter::All,
            toast: None,
            toast_at: 0,
            data_version: 1,
            counts_version: 1,
            view_cache: RefCell::new(ViewCache {
                data_version: 0,
                query: QueryKey {
                    search: String::new(),
                    filter: Filter::All,
                    sort: SortKey::Updated,
                    desc: true,
                },
                indices: Vec::new(),
                generation: 0,
            }),
            counts_cache: RefCell::new((0, Counts::default())),
        }
    }

    pub fn now(&self) -> u64 {
        now_secs()
    }

    pub fn me(&self) -> &crate::model::User {
        self.data
            .user(self.data.current_user)
            .unwrap_or(&self.data.users[0])
    }

    pub fn can_input(&self) -> bool {
        self.me().scopes.can_input()
    }

    pub fn can_update(&self) -> bool {
        self.me().scopes.can_update()
    }

    /// Remarks are a conversation, so every writable scope can join it.
    pub fn can_comment(&self) -> bool {
        !self.me().scopes.is_read_only()
    }

    pub fn is_admin(&self) -> bool {
        self.me().scopes.is_admin()
    }

    pub fn unread(&self) -> usize {
        self.data.unread_for(self.data.current_user)
    }

    pub fn persist(&mut self) {
        if let Err(e) = self.repo.save(&self.data) {
            self.toast("Penyimpanan gagal", e, Tone::Warning);
        }
    }

    pub fn announce(&mut self, title: impl Into<String>, body: impl Into<String>, tone: Tone) {
        self.toast = Some(ToastNote {
            title: title.into(),
            body: body.into(),
            tone,
        });
        self.toast_at = now_secs();
    }

    pub fn toast(&mut self, title: impl Into<String>, body: impl Into<String>, tone: Tone) {
        self.announce(title, body, tone);
    }

    pub fn dismiss_toast(&mut self) {
        self.toast = None;
    }

    /// Clears a toast after four seconds. `now` is the runtime's clock; the wall clock decides,
    /// so a paused or headless run still behaves.
    pub fn poll(&mut self, _now: f64, _tx: &mut Tx) -> bool {
        if self.toast.is_some() && now_secs().saturating_sub(self.toast_at) >= 4 {
            self.toast = None;
            return true;
        }
        false
    }

    pub fn page_title(&self) -> (&'static str, String) {
        match self.page {
            Page::Samples => {
                let c = self.counts();
                (
                    "Sample produk",
                    format!(
                        "{} sample \u{00b7} {} belum analisa \u{00b7} {} menunggu HUDS",
                        c.total, c.unanalyzed, c.waiting
                    ),
                )
            }
            Page::People => {
                let n = self.data.users.len();
                (
                    "Pengguna & akses",
                    format!("{n} anggota tim · scope izin per pengguna"),
                )
            }
            Page::Notices => {
                let unread = self.unread();
                (
                    "Notifikasi",
                    if unread == 0 {
                        "Semua update sudah dibaca".into()
                    } else {
                        format!("{unread} update belum dibaca")
                    },
                )
            }
        }
    }

    // ---- samples ----------------------------------------------------------

    pub fn counts(&self) -> Counts {
        let mut cache = self.counts_cache.borrow_mut();
        if cache.0 != self.counts_version {
            let mut counts = Counts {
                total: self.data.samples.len(),
                ..Counts::default()
            };
            for sample in &self.data.samples {
                match sample.status {
                    Status::Unanalyzed => counts.unanalyzed += 1,
                    Status::Approved => counts.approved += 1,
                    Status::WaitingHuds => counts.waiting += 1,
                    Status::Resampling => counts.resampling += 1,
                }
            }
            *cache = (self.counts_version, counts);
        }
        cache.1
    }

    /// Rebuilds the filtered/sorted index only when the query or the data changed. The table
    /// then mounts a window of that index instead of every row.
    fn refresh_visible(&self) {
        let query = QueryKey {
            search: self.search.text.trim().to_owned(),
            filter: self.filter,
            sort: self.sort,
            desc: self.sort_desc,
        };
        let mut cache = self.view_cache.borrow_mut();
        if cache.data_version == self.data_version && cache.query == query {
            return;
        }
        let samples = &self.data.samples;
        let needle = query.search.as_str();
        let mut indices = Vec::new();
        if needle.is_empty() && query.filter == Filter::All {
            indices.reserve(samples.len());
        }
        for (i, sample) in samples.iter().enumerate() {
            if !query.filter.matches(sample.status) {
                continue;
            }
            if !needle.is_empty()
                && !contains_ci(&sample.product, needle)
                && !contains_ci(&sample.batch, needle)
                && !contains_ci(sample.stage.label(), needle)
                && !sample.remarks.iter().any(|r| contains_ci(&r.text, needle))
            {
                continue;
            }
            indices.push(i);
        }
        indices.sort_unstable_by(|&a, &b| {
            let (left, right) = (&samples[a], &samples[b]);
            let order = match query.sort {
                SortKey::Updated => left.updated_at.cmp(&right.updated_at),
                SortKey::Product => cmp_ci(&left.product, &right.product),
                SortKey::Batch => cmp_ci(&left.batch, &right.batch),
                SortKey::Status => left.status.rank().cmp(&right.status.rank()),
            }
            .then_with(|| left.id.cmp(&right.id));
            if query.desc { order.reverse() } else { order }
        });
        cache.data_version = self.data_version;
        cache.query = query;
        cache.indices = indices;
        cache.generation = cache.generation.wrapping_add(1);
    }

    pub fn visible_len(&self) -> usize {
        self.refresh_visible();
        self.view_cache.borrow().indices.len()
    }

    /// Changes whenever the mounted row keys or their order may change, so `virtual_rows`
    /// can skip calling `key` for every row on an unchanged frame.
    pub fn visible_generation(&self) -> u64 {
        self.refresh_visible();
        self.view_cache.borrow().generation
    }

    pub fn visible_key(&self, row: usize) -> String {
        self.refresh_visible();
        let cache = self.view_cache.borrow();
        cache
            .indices
            .get(row)
            .and_then(|&index| self.data.samples.get(index))
            .map(|sample| format!("sample-{}", sample.id))
            .unwrap_or_default()
    }

    pub fn visible_sample(&self, row: usize) -> Option<&Sample> {
        self.refresh_visible();
        let cache = self.view_cache.borrow();
        let index = *cache.indices.get(row)?;
        self.data.samples.get(index)
    }

    pub fn sort_by(&mut self, key: SortKey) {
        if self.sort == key {
            self.sort_desc = !self.sort_desc;
        } else {
            self.sort = key;
            self.sort_desc = matches!(key, SortKey::Updated);
        }
    }

    pub fn reset_filters(&mut self) {
        self.search = TextEdit::new("");
        self.filter = Filter::All;
    }

    pub fn open_new_sample(&mut self) {
        self.new_product = TextEdit::new("");
        self.new_batch = TextEdit::new("");
        self.new_stage = Stage::Granule;
        self.new_remarks = TextEdit::new("");
        self.remark_draft = TextEdit::new("");
        self.dialog = Dialog::NewSample;
    }

    pub fn open_sample(&mut self, id: u64) {
        let Some(sample) = self.data.sample(id) else {
            return;
        };
        self.edit_id = Some(id);
        self.edit_status = sample.status;
        self.remark_draft = TextEdit::new("");
        self.recipients = self
            .data
            .users
            .iter()
            .filter(|u| u.id != self.data.current_user)
            .map(|u| u.id)
            .collect();
        self.dialog = Dialog::Sample(id);
    }

    pub fn close_dialog(&mut self) {
        self.dialog = Dialog::None;
        self.edit_id = None;
        self.open_select = None;
    }

    pub fn batch_taken(&self, batch: &str, except: Option<u64>) -> bool {
        let batch = batch.trim().to_lowercase();
        self.data
            .samples
            .iter()
            .any(|s| Some(s.id) != except && s.batch.to_lowercase() == batch)
    }

    pub fn new_sample_ready(&self) -> bool {
        !self.new_product.text.trim().is_empty()
            && !self.new_batch.text.trim().is_empty()
            && !self.batch_taken(&self.new_batch.text, None)
    }

    pub fn save_new_sample(&mut self) {
        if !self.can_input() {
            self.toast(
                "Akses ditolak",
                "Scope Anda tidak dapat menambah sample.",
                Tone::Warning,
            );
            return;
        }
        if !self.new_sample_ready() {
            return;
        }
        let now = now_secs();
        let me = self.data.current_user;
        let id = self.data.next_sample;
        self.data.next_sample += 1;
        let mut remarks = Vec::new();
        let initial = self.new_remarks.text.trim().to_owned();
        if !initial.is_empty() {
            let remark_id = self.data.next_remark;
            self.data.next_remark += 1;
            remarks.push(Remark {
                id: remark_id,
                by: me,
                text: initial,
                at: now,
            });
        }
        let sample = Sample {
            id,
            product: self.new_product.text.trim().to_owned(),
            batch: self.new_batch.text.trim().to_owned(),
            stage: self.new_stage,
            status: Status::Unanalyzed,
            input_by: me,
            input_at: now,
            updated_by: me,
            updated_at: now,
            remarks,
            legacy_note: String::new(),
        };
        let product = sample.product.clone();
        let batch = sample.batch.clone();
        let stage = sample.stage.label();
        self.data.samples.push(sample);
        self.data_version += 1;
        self.counts_version += 1;
        self.close_dialog();
        self.persist();
        self.page = Page::Samples;
        self.toast(
            "Sample ditambahkan",
            format!("{product} ({batch}, {stage}) masuk antrean analisa."),
            Tone::Success,
        );
    }

    pub fn save_update(&mut self) {
        let Some(id) = self.edit_id else {
            return;
        };
        if !self.can_update() {
            self.toast(
                "Akses ditolak",
                "Scope Anda tidak dapat mengubah status.",
                Tone::Warning,
            );
            return;
        }
        let Some(current) = self.data.sample(id) else {
            return;
        };
        if current.status.is_final() {
            self.toast(
                "Status sudah final",
                "Sample yang sudah di-update tidak dapat diubah lagi.",
                Tone::Warning,
            );
            return;
        }
        let status = self.edit_status;
        if status == Status::Unanalyzed {
            self.toast(
                "Pilih status",
                "Tentukan hasil analisa sebelum menyimpan.",
                Tone::Warning,
            );
            return;
        }
        let now = now_secs();
        let me = self.me().id;
        let actor = self.me().name.clone();
        let recipients: Vec<u64> = {
            let mut v: Vec<u64> = self
                .recipients
                .iter()
                .copied()
                .filter(|u| *u != me)
                .collect();
            v.sort_unstable();
            v
        };
        let Some(sample) = self.data.sample_mut(id) else {
            return;
        };
        sample.status = status;
        sample.updated_by = me;
        sample.updated_at = now;
        let product = sample.product.clone();
        let batch = sample.batch.clone();
        let stage = sample.stage;
        if !recipients.is_empty() {
            let body = format!(
                "{actor} memperbarui {product} ({batch}) → {}.",
                status.label()
            );
            let nid = self.data.next_notice;
            self.data.next_notice += 1;
            self.data.notices.push(Notice {
                id: nid,
                from: me,
                to: recipients.clone(),
                title: "Sample diperbarui".into(),
                body,
                at: now,
                read_by: Vec::new(),
                batch,
                status: Some(status),
                stage: Some(stage),
            });
        }
        self.data_version += 1;
        self.counts_version += 1;
        self.close_dialog();
        self.persist();
        self.page = Page::Samples;
        let sent = recipients.len();
        let body = if sent == 0 {
            format!("{product} sekarang {}.", status.label())
        } else {
            format!(
                "{product} sekarang {} · {sent} notifikasi terkirim.",
                status.label()
            )
        };
        self.toast("Update tersimpan", body, Tone::Success);
    }

    /// Appends one comment to a sample's thread. The dialog stays open so the thread grows in
    /// place, as a comment column does in a spreadsheet.
    pub fn add_remark(&mut self, sample_id: u64) {
        if !self.can_comment() {
            self.toast(
                "Akses ditolak",
                "Scope read-only tidak dapat menambah remarks.",
                Tone::Warning,
            );
            return;
        }
        let text = self.remark_draft.text.trim().to_owned();
        let Some(existing) = self.data.sample(sample_id) else {
            return;
        };
        if existing.status.is_final() {
            self.toast(
                "Sample sudah final",
                "Sample yang sudah di-update tidak dapat diubah lagi, termasuk remarks.",
                Tone::Warning,
            );
            return;
        }
        if text.is_empty() {
            return;
        }
        let id = self.data.next_remark;
        self.data.next_remark += 1;
        let by = self.data.current_user;
        let at = now_secs();
        if let Some(sample) = self.data.sample_mut(sample_id) {
            sample.remarks.push(Remark { id, by, text, at });
        }
        self.remark_draft = TextEdit::new("");
        self.data_version += 1;
        self.persist();
    }

    pub fn toggle_recipient(&mut self, id: u64) {
        if !self.recipients.remove(&id) {
            self.recipients.insert(id);
        }
    }

    pub fn all_others_selected(&self) -> bool {
        self.data
            .users
            .iter()
            .filter(|u| u.id != self.data.current_user)
            .all(|u| self.recipients.contains(&u.id))
    }

    pub fn set_all_recipients(&mut self, on: bool) {
        self.recipients.clear();
        if on {
            self.recipients.extend(
                self.data
                    .users
                    .iter()
                    .filter(|u| u.id != self.data.current_user)
                    .map(|u| u.id),
            );
        }
    }

    // ---- people -----------------------------------------------------------

    pub fn is_last_admin(&self, id: u64) -> bool {
        self.data.admin_count() == 1 && self.data.user(id).is_some_and(|u| u.scopes.is_admin())
    }

    /// Toggles one scope on a person. Admin covers every other scope, so those toggles wait
    /// until Admin is switched off; the last admin cannot be removed.
    pub fn toggle_scope(&mut self, id: u64, scope: Scope) {
        let Some(user) = self.data.user(id) else {
            return;
        };
        if user.scopes.is_admin() && !matches!(scope, Scope::Admin | Scope::ReadOnly) {
            self.toast(
                "Admin mencakup semua",
                "Matikan scope Admin dulu untuk mengatur izin lain.",
                Tone::Info,
            );
            return;
        }
        if self.is_last_admin(id) && matches!(scope, Scope::Admin | Scope::ReadOnly) {
            self.toast(
                "Admin terakhir",
                "Minimal satu admin harus tersisa di tim.",
                Tone::Warning,
            );
            return;
        }
        let Some(user) = self.data.user_mut(id) else {
            return;
        };
        user.scopes.toggle(scope);
        let name = user.name.clone();
        let summary = user.scopes.summary();
        if id == self.data.current_user && !self.is_admin() {
            self.page = Page::Samples;
        }
        self.persist();
        self.toast(
            "Scope diperbarui",
            format!("{name}: {summary}."),
            Tone::Success,
        );
    }

    pub fn open_new_user(&mut self) {
        self.new_name = TextEdit::new("");
        self.new_scopes = Scopes::input();
        self.dialog = Dialog::NewUser;
    }

    pub fn new_user_ready(&self) -> bool {
        let name = self.new_name.text.trim();
        !name.is_empty()
            && !self
                .data
                .users
                .iter()
                .any(|u| u.name.eq_ignore_ascii_case(name))
    }

    pub fn save_new_user(&mut self) {
        if !self.is_admin() {
            return;
        }
        if !self.new_user_ready() {
            return;
        }
        let id = self.data.next_user;
        self.data.next_user += 1;
        let name = self.new_name.text.trim().to_owned();
        let scopes = self.new_scopes;
        self.data.users.push(crate::model::User {
            id,
            name: name.clone(),
            scopes,
            legacy_role: String::new(),
        });
        self.close_dialog();
        self.persist();
        self.toast(
            "Anggota ditambahkan",
            format!("{name} masuk dengan scope {}.", scopes.summary()),
            Tone::Success,
        );
    }

    pub fn switch_user(&mut self, id: u64) {
        if self.data.user(id).is_none() {
            return;
        }
        self.data.current_user = id;
        if self.page == Page::People && !self.is_admin() {
            self.page = Page::Samples;
        }
        self.close_dialog();
        self.persist();
        let name = self.me().name.clone();
        self.toast("Masuk sebagai", name, Tone::Info);
    }

    // ---- notifications ----------------------------------------------------

    /// The current person's notices through the active inbox filter.
    pub fn notices(&self) -> Vec<&Notice> {
        let me = self.data.current_user;
        self.data
            .notices_for(me)
            .into_iter()
            .filter(|n| self.notice_filter.matches(n, me))
            .collect()
    }

    /// Counts per filter, in `NoticeFilter::ALL` order, for the tabs' badges.
    pub fn notice_counts(&self) -> [usize; 6] {
        let me = self.data.current_user;
        let rows = self.data.notices_for(me);
        let mut counts = [0usize; 6];
        for (i, filter) in NoticeFilter::ALL.into_iter().enumerate() {
            counts[i] = rows.iter().filter(|n| filter.matches(n, me)).count();
        }
        counts
    }

    pub fn mark_read(&mut self, notice_id: u64) {
        let me = self.data.current_user;
        if let Some(n) = self.data.notices.iter_mut().find(|n| n.id == notice_id) {
            if !n.read_by.contains(&me) {
                n.read_by.push(me);
                self.persist();
            }
        }
    }

    pub fn mark_all_read(&mut self) {
        let me = self.data.current_user;
        let mut changed = false;
        for n in &mut self.data.notices {
            if n.to.contains(&me) && !n.read_by.contains(&me) {
                n.read_by.push(me);
                changed = true;
            }
        }
        if changed {
            self.persist();
            self.toast(
                "Semua dibaca",
                "Notifikasi sudah ditandai terbaca.",
                Tone::Info,
            );
        }
    }

    pub fn storage_note(&self) -> String {
        self.repo.location()
    }

    // ---- snapshots --------------------------------------------------------

    pub fn reset_demo(&mut self) {
        self.data = Data::seed(now_secs());
        self.page = Page::Samples;
        self.dialog = Dialog::None;
        self.open_select = None;
        self.search = TextEdit::new("");
        self.filter = Filter::All;
        self.sort = SortKey::Updated;
        self.sort_desc = true;
        self.notice_filter = NoticeFilter::All;
        self.toast = None;
        self.edit_id = None;
        self.data_version = self.data_version.wrapping_add(1);
        self.counts_version = self.counts_version.wrapping_add(1);
    }
}

/// Case-insensitive substring test without allocating a lowercase copy per sample.
fn contains_ci(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }
    let haystack = haystack.as_bytes();
    let needle = needle.as_bytes();
    needle.len() <= haystack.len()
        && haystack
            .windows(needle.len())
            .any(|window| window.eq_ignore_ascii_case(needle))
}

/// Case-insensitive ordering by bytes, for sorting a million rows without per-comparison
/// allocations.
fn cmp_ci(a: &str, b: &str) -> Ordering {
    let mut left = a.bytes();
    let mut right = b.bytes();
    loop {
        match (left.next(), right.next()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(x), Some(y)) => {
                let (x, y) = (x.to_ascii_lowercase(), y.to_ascii_lowercase());
                if x != y {
                    return x.cmp(&y);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rune::{Env, Runtime};

    fn demo() -> App {
        App::preview(Data::seed(1_700_000_000))
    }

    #[test]
    fn a_new_sample_records_the_signed_in_user_for_both_audit_columns() {
        let mut a = demo();
        a.data.current_user = 2;
        a.open_new_sample();
        a.new_product = TextEdit::new("Uji Kadar Air");
        a.new_batch = TextEdit::new("MO-TEST-001");
        a.new_remarks = TextEdit::new("Sampel dari line 2.");
        a.save_new_sample();
        let row = a.data.samples.last().expect("the new sample");
        assert_eq!(row.input_by, 2);
        assert_eq!(row.updated_by, 2);
        assert_eq!(row.status, Status::Unanalyzed);
        assert_eq!(row.stage, Stage::Granule);
        assert_eq!(row.remarks.len(), 1);
        assert_eq!(row.remarks[0].by, 2);
        assert_eq!(row.remarks[0].text, "Sampel dari line 2.");
        assert_eq!(a.data.samples.len(), 13);
    }

    #[test]
    fn remarks_are_an_append_only_thread_until_the_sample_is_final() {
        let mut a = demo();
        let before = a.data.sample(8).unwrap().remarks.len();
        let status_editor = a.data.sample(8).unwrap().updated_by;

        // Input and update scopes can both comment, and each comment keeps its author.
        a.data.current_user = 2;
        a.open_sample(8);
        a.remark_draft = TextEdit::new("Cek ulang kemasan batch ini.");
        a.add_remark(8);
        a.data.current_user = 4;
        a.remark_draft = TextEdit::new("Foto kemasan sudah diunggah.");
        a.add_remark(8);

        let sample = a.data.sample(8).unwrap();
        assert_eq!(sample.remarks.len(), before + 2);
        assert_eq!(sample.remarks[before].by, 2);
        assert_eq!(sample.remarks[before + 1].by, 4);
        // Commenting does not rewrite the status audit columns.
        assert_eq!(sample.updated_by, status_editor);

        // Read-only can read the thread but cannot append.
        a.data.current_user = 3;
        a.remark_draft = TextEdit::new("Tidak boleh.");
        a.add_remark(8);
        assert_eq!(a.data.sample(8).unwrap().remarks.len(), before + 2);

        // A decision closes the sample: no more remarks from anyone.
        a.data.current_user = 4;
        a.open_sample(8);
        a.edit_status = Status::Resampling;
        a.save_update();
        a.data.current_user = 1;
        a.remark_draft = TextEdit::new("Terlambat.");
        a.add_remark(8);
        assert_eq!(a.data.sample(8).unwrap().remarks.len(), before + 2);
    }

    #[test]
    fn a_legacy_single_note_becomes_the_first_remark() {
        let mut data = Data::seed(1_700_000_000);
        let sample = &mut data.samples[0];
        sample.remarks.clear();
        sample.legacy_note = "Catatan lama dari versi sebelumnya.".into();
        data.next_remark = 1;
        data.migrate();
        let sample = &data.samples[0];
        assert_eq!(sample.remarks.len(), 1);
        assert_eq!(
            sample.remarks[0].text,
            "Catatan lama dari versi sebelumnya."
        );
        assert_eq!(sample.legacy_note, "");
        assert!(data.next_remark > sample.remarks[0].id);
    }

    #[test]
    fn scopes_are_enforced_by_the_actions_not_only_the_buttons() {
        let mut a = demo();
        // QA is read-only: neither create nor update.
        a.data.current_user = 3;
        a.open_sample(8);
        a.edit_status = Status::Resampling;
        a.save_update();
        assert_eq!(a.data.sample(8).unwrap().status, Status::Unanalyzed);
        a.open_new_sample();
        a.new_product = TextEdit::new("Tidak boleh");
        a.new_batch = TextEdit::new("MO-NOPE");
        a.save_new_sample();
        assert_eq!(a.data.samples.len(), 12);

        // Helper can input but not update.
        a.data.current_user = 2;
        a.open_sample(8);
        a.edit_status = Status::Resampling;
        a.save_update();
        assert_eq!(a.data.sample(8).unwrap().status, Status::Unanalyzed);

        // Leader QC holds both scopes, so both actions work for one person.
        a.data.current_user = 4;
        a.open_sample(8);
        a.edit_status = Status::Resampling;
        a.save_update();
        assert_eq!(a.data.sample(8).unwrap().status, Status::Resampling);
        a.open_new_sample();
        a.new_product = TextEdit::new("Boleh");
        a.new_batch = TextEdit::new("MO-LEADER");
        a.save_new_sample();
        assert_eq!(a.data.samples.len(), 13);
    }

    #[test]
    fn a_final_status_cannot_be_changed_again_by_anyone() {
        let mut a = demo();
        a.data.current_user = 4;
        a.open_sample(8);
        a.edit_status = Status::Approved;
        a.save_update();
        assert_eq!(a.data.sample(8).unwrap().status, Status::Approved);

        // Even an admin cannot move a decided sample.
        a.data.current_user = 1;
        a.open_sample(8);
        a.edit_status = Status::Resampling;
        a.save_update();
        assert_eq!(a.data.sample(8).unwrap().status, Status::Approved);
    }

    #[test]
    fn the_last_admin_cannot_be_demoted() {
        let mut a = demo();
        a.toggle_scope(1, Scope::ReadOnly);
        assert!(a.data.user(1).unwrap().scopes.is_admin());
        a.data.users.push(crate::model::User {
            id: 9,
            name: "Admin Dua".into(),
            scopes: Scopes::admin(),
            legacy_role: String::new(),
        });
        a.toggle_scope(1, Scope::ReadOnly);
        assert!(a.data.user(1).unwrap().scopes.is_read_only());
    }

    #[test]
    fn an_update_notifies_only_the_picked_people_and_stamps_the_editor() {
        let mut a = demo();
        a.data.current_user = 1;
        a.open_sample(8);
        a.edit_status = Status::WaitingHuds;
        a.recipients.clear();
        a.recipients.insert(2);
        a.recipients.insert(5);
        a.recipients.insert(1); // the sender is never a recipient
        a.save_update();
        let sample = a.data.sample(8).unwrap();
        assert_eq!(sample.status, Status::WaitingHuds);
        assert_eq!(sample.updated_by, 1);
        let notice = a.data.notices.last().unwrap();
        assert_eq!(notice.to, vec![2, 5]);
        assert!(notice.body.contains("Salbutamol Inhaler"));
        assert!(notice.read_by.is_empty());
        assert_eq!(notice.status, Some(Status::WaitingHuds));
        assert_eq!(notice.stage, Some(sample.stage));
    }

    #[test]
    fn the_inbox_filters_by_status_and_unread() {
        let mut a = demo();
        a.data.current_user = 1;
        assert_eq!(a.notice_counts(), [3, 2, 1, 0, 1, 1]);
        a.notice_filter = NoticeFilter::Resampling;
        let rows = a.notices();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].status, Some(Status::Resampling));
        a.notice_filter = NoticeFilter::Unread;
        assert_eq!(a.notices().len(), 2);
    }

    #[test]
    fn marking_a_notice_read_is_per_person() {
        let mut a = demo();
        a.data.current_user = 1;
        assert_eq!(a.unread(), 2);
        a.mark_read(1);
        assert_eq!(a.unread(), 1);
        a.mark_read(3);
        assert_eq!(a.unread(), 0);
        a.data.current_user = 2;
        assert_eq!(a.unread(), 1); // Helper read the first notice, not the resampling one
        a.data.current_user = 3;
        assert_eq!(a.unread(), 2);
    }

    #[test]
    fn a_scope_toggle_in_the_people_page_changes_the_person() {
        let mut app = demo();
        app.page = Page::People;
        let env = Env {
            system_dark: true,
            scale: 2.0,
        };
        let mut rt = Runtime::new(app, crate::ui::view, (1360.0, 860.0), env);
        rt.frame(0.0);
        assert!(rt.state.data.user(2).unwrap().scopes.input);
        let (x, y) = rt.find("scope-2-Input").expect("Helper's Input toggle");
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        for _ in 0..2 {
            rt.frame(1.0 / 60.0);
        }
        assert!(!rt.state.data.user(2).unwrap().scopes.can_input());
        assert!(rt.state.data.user(2).unwrap().scopes.is_read_only());
    }

    #[test]
    fn a_row_opens_the_update_dialog_and_saving_lands_the_change() {
        let mut app = demo();
        app.data.current_user = 1;
        let env = Env {
            system_dark: true,
            scale: 2.0,
        };
        let mut rt = Runtime::new(app, crate::ui::view, (1360.0, 860.0), env);
        rt.frame(0.0);
        let (x, y) = rt
            .find("filter-Belum analisa")
            .expect("the unanalyzed filter");
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        for _ in 0..2 {
            rt.frame(1.0 / 60.0);
        }

        let (x, y) = rt.find("sample-8").expect("the Salbutamol row");
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        for _ in 0..2 {
            rt.frame(1.0 / 60.0);
        }
        assert_eq!(rt.state.dialog, Dialog::Sample(8));

        let (x, y) = rt
            .find("choice-Resampling Ulang")
            .expect("the resampling choice");
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        for _ in 0..2 {
            rt.frame(1.0 / 60.0);
        }
        assert_eq!(rt.state.edit_status, Status::Resampling);

        let (x, y) = rt.find("save-update").expect("the save button");
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        for _ in 0..2 {
            rt.frame(1.0 / 60.0);
        }
        assert_eq!(rt.state.dialog, Dialog::None);
        assert_eq!(rt.state.data.sample(8).unwrap().status, Status::Resampling);
        assert!(
            rt.state
                .data
                .notices
                .iter()
                .any(|n| n.body.contains("Salbutamol Inhaler"))
        );
    }

    /// Run with:
    /// `cargo test -p sample-tracker stress_one_million -- --ignored --nocapture --release`
    #[test]
    #[ignore]
    fn stress_one_million_samples() {
        use std::time::Instant;

        let now = 1_700_000_000u64;
        let mut data = Data::seed(now);
        data.samples.clear();
        data.samples.reserve(1_000_000);
        let stages = Stage::ALL;
        let statuses = Status::ALL;
        for i in 0..1_000_000u64 {
            let id = i + 1;
            data.samples.push(Sample {
                id,
                product: format!(
                    "Produk {} {}",
                    i % 5_000,
                    if i % 2 == 0 { "500 mg" } else { "250 mg" }
                ),
                batch: format!("MO-{:07}", i),
                stage: stages[(i % 3) as usize],
                status: statuses[(i % 4) as usize],
                input_by: 2,
                input_at: now.saturating_sub(i % 86_400),
                updated_by: 4,
                updated_at: now.saturating_sub(i % 86_400),
                remarks: Vec::new(),
                legacy_note: String::new(),
            });
        }
        let mut app = App::preview(data);
        app.data_version += 1;
        app.counts_version += 1;

        let t = Instant::now();
        let total = app.counts().total;
        println!("counts: {total} rows in {:?}", t.elapsed());

        let t = Instant::now();
        let len = app.visible_len();
        println!(
            "first filtered/sorted index: {len} rows in {:?}",
            t.elapsed()
        );

        let t = Instant::now();
        for _ in 0..1_000 {
            assert_eq!(app.visible_len(), len);
        }
        println!("cached visible_len x1000: {:?}", t.elapsed());

        let t = Instant::now();
        app.sort = SortKey::Product;
        app.sort_desc = false;
        let sorted = app.visible_len();
        println!("sort by product: {sorted} rows in {:?}", t.elapsed());

        let t = Instant::now();
        app.search = TextEdit::new("MO-0500");
        let searched = app.visible_len();
        println!("search MO-0500: {searched} rows in {:?}", t.elapsed());

        let t = Instant::now();
        for i in 0..120 {
            assert!(app.visible_sample(i).is_some());
            assert!(!app.visible_key(i).is_empty());
        }
        println!("mounted-window lookups x120: {:?}", t.elapsed());
    }
}

pub fn scenes() -> &'static [(&'static str, fn(&mut App))] {
    &[
        ("samples", |s| s.reset_demo()),
        ("people", |s| {
            s.reset_demo();
            s.page = Page::People;
        }),
        ("notices", |s| {
            s.reset_demo();
            s.page = Page::Notices;
        }),
        ("new-sample", |s| {
            s.reset_demo();
            s.open_new_sample();
        }),
        ("edit-sample", |s| {
            s.reset_demo();
            s.open_sample(2);
        }),
        ("switcher", |s| {
            s.reset_demo();
            s.dialog = Dialog::SwitchUser;
        }),
        ("new-user", |s| {
            s.reset_demo();
            s.open_new_user();
        }),
        ("readonly", |s| {
            s.reset_demo();
            s.data.current_user = 3;
        }),
        ("readonly-edit", |s| {
            s.reset_demo();
            s.data.current_user = 3;
            s.open_sample(2);
        }),
        ("input-edit", |s| {
            s.reset_demo();
            s.data.current_user = 2;
            s.open_sample(1);
        }),
        ("leader-edit", |s| {
            s.reset_demo();
            s.data.current_user = 4;
            s.open_sample(2);
        }),
        ("toast", |s| {
            s.reset_demo();
            s.announce(
                "Update tersimpan",
                "Amoxicillin 500 mg sekarang Resampling Ulang · 5 notifikasi terkirim.",
                Tone::Success,
            );
        }),
    ]
}
