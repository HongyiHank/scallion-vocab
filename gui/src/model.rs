use serde::{Deserialize, Serialize};
use std::time::Instant;

pub(crate) const OPT_COUNT: usize = 4;
pub(crate) const REVIEW_LO: usize = 8;
pub(crate) const REVIEW_HI: usize = 15;

// ── FSRS types ──

// FsrsRating 未實際經 serde 序列化，故不用 serde_repr crate
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[repr(i32)]
pub enum FsrsRating {
    Again = 1,
    Hard = 2,
    Good = 3,
    Easy = 4,
}

impl FsrsRating {
    pub fn label(&self) -> &'static str {
        match self {
            FsrsRating::Again => "重來",
            FsrsRating::Hard => "困難",
            FsrsRating::Good => "良好",
            FsrsRating::Easy => "簡單",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum CardState {
    New,
    Learning,
    Review,
    Relearning,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FsrsCard {
    pub stability: f64,
    pub difficulty: f64,
    pub state: CardState,
    pub lapses: u32,
}

impl Default for FsrsCard {
    fn default() -> Self {
        Self { stability: 0.0, difficulty: 5.0, state: CardState::New, lapses: 0 }
    }
}

// ── FSRS-6 algorithm (via fsrs-rs crate) ──

fn fsrs_instance() -> fsrs::FSRS {
    fsrs::FSRS::default()
}

/// Convert FSRS stability (days) to a question-count offset.
fn fsrs_stability_to_offset(stability_days: f32) -> usize {
    let offset = (stability_days * 10.0).round() as usize;
    (REVIEW_LO + offset).max(REVIEW_LO)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FsrsConfig {
    pub enabled: bool,
    pub review_wrong: bool,
    pub hard_threshold_ms: u64,
    pub good_threshold_ms: u64,
    pub easy_threshold_ms: u64,
}

impl Default for FsrsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            review_wrong: true,
            hard_threshold_ms: 10000,
            good_threshold_ms: 6000,
            easy_threshold_ms: 3000,
        }
    }
}

impl FsrsConfig {
    pub fn auto_rating(&self, correct: bool, answer_time_ms: u64) -> FsrsRating {
        if !correct {
            return FsrsRating::Again;
        }
        if answer_time_ms <= self.easy_threshold_ms {
            FsrsRating::Easy
        } else if answer_time_ms <= self.good_threshold_ms {
            FsrsRating::Good
        } else {
            FsrsRating::Hard
        }
    }
}



#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Word {
    pub front: String,
    pub back: String,
}

#[derive(Clone, PartialEq)]
pub enum Screen {
    Upload,
    Quiz,
}

#[derive(Clone, Debug)]
pub struct HistoryItem {
    pub target_idx: usize,
    pub options: Vec<usize>,
    pub ask_front: bool,
    pub answered: bool,
    pub skipped: bool,
    pub selected_idx: Option<usize>,
    pub correct_opt: usize,
    pub question_time: Instant,
    pub answer_time_ms: Option<u64>,
    pub auto_rating: Option<FsrsRating>,
    pub manual_rating: Option<FsrsRating>,
}

impl HistoryItem {
    pub fn rating(&self) -> Option<FsrsRating> {
        self.manual_rating.or(self.auto_rating)
    }
}

#[derive(Clone, Debug)]
pub struct ReviewItem {
    pub word_idx: usize,
    pub due_after: usize,
}

#[derive(Clone)]
pub struct QuizState {
    pub words: Vec<Word>,
    pub fsrs_cards: Vec<FsrsCard>,
    unseen: Vec<usize>,
    pub history: Vec<HistoryItem>,
    pub review: Vec<ReviewItem>,
    pub current: usize,
    pub infinite: bool,
    pub fsrs_config: FsrsConfig,
}

impl QuizState {
    pub fn new(words: Vec<Word>, infinite: bool, fsrs_config: FsrsConfig) -> Self {
        let n = words.len();
        let mut unseen: Vec<usize> = (0..n).collect();
        shuffle(&mut unseen);
        Self {
            fsrs_cards: vec![FsrsCard::default(); n],
            words,
            unseen,
            history: vec![],
            review: vec![],
            current: 0,
            infinite,
            fsrs_config,
        }
    }

    pub fn current_question(&self) -> Option<&HistoryItem> {
        self.history.get(self.current)
    }

    fn refill_unseen(&mut self) {
        let n = self.words.len();
        let mut idxs: Vec<usize> = (0..n).collect();
        shuffle(&mut idxs);
        self.unseen = idxs;
    }

    // pull from unseen; if same as last question, swap with next to avoid immediate repeat
    fn pick_new(&mut self) -> usize {
        if self.unseen.is_empty() {
            self.refill_unseen();
        }
        let picked = self.unseen.pop().unwrap_or(0);

        if self.words.len() > 1 {
            if let Some(last_q) = self.history.last() {
                if last_q.target_idx == picked {
                    if let Some(next) = self.unseen.pop() {
                        self.unseen.push(picked);
                        return next;
                    }
                }
            }
        }
        picked
    }

    pub fn gen_question(&mut self) -> bool {
        if self.words.is_empty() {
            return false;
        }

        let target = {
            // review queue 優先
            if let Some(idx) = self
                .review
                .iter()
                .position(|r| self.history.len() >= r.due_after)
            {
                self.review.remove(idx).word_idx
            } else if self.unseen.is_empty() {
                if self.infinite { self.pick_new() } else { return false }
            } else {
                self.unseen.pop().unwrap_or(0)
            }
        };

        // 構建選項：target + 其他不重複且不同英文的單字
        // 排除同 front（英文）的詞，避免同一英文多種中文出現在選項中混淆
        let target_front = &self.words[target].front;
        let mut opts = vec![target];
        let mut pool: Vec<usize> = (0..self.words.len())
            .filter(|i| *i != target && &self.words[*i].front != target_front)
            .collect();
        shuffle(&mut pool);
        for idx in pool {
            if opts.len() >= OPT_COUNT {
                break;
            }
            if !opts.contains(&idx) {
                opts.push(idx);
            }
        }
        shuffle(&mut opts);

        let ask_front = rand_bool();
        let correct_opt = opts.iter().position(|i| *i == target).unwrap();

        self.history.push(HistoryItem {
            target_idx: target,
            options: opts,
            ask_front,
            answered: false,
            skipped: false,
            selected_idx: None,
            correct_opt,
            question_time: Instant::now(),
            answer_time_ms: None,
            auto_rating: None,
            manual_rating: None,
        });

        self.current = self.history.len().saturating_sub(1);
        true
    }

    pub fn has_more(&self) -> bool {
        self.infinite
            || self.current + 1 < self.history.len()
            || !self.unseen.is_empty()
            || self.review.iter().any(|r| self.history.len() >= r.due_after)
    }

    // FSRS-based review scheduling
    fn schedule_review_fsrs(&mut self, word_idx: usize, rating: FsrsRating) {
        let card = &self.fsrs_cards[word_idx];
        let is_new = card.state == CardState::New;

        let prev_memory = if is_new {
            None
        } else {
            // all reviews same-session, elapsed_days = 0
            Some(fsrs::MemoryState { stability: card.stability as f32, difficulty: card.difficulty as f32 })
        };

        let fsrs = fsrs_instance();
        let next = fsrs.next_states(prev_memory, 0.9, 0).expect("FSRS next_states");
        let item = match rating {
            FsrsRating::Again => next.again,
            FsrsRating::Hard => next.hard,
            FsrsRating::Good => next.good,
            FsrsRating::Easy => next.easy,
        };

        let new_lapses = card.lapses + if rating == FsrsRating::Again { 1 } else { 0 };
        let offset = fsrs_stability_to_offset(item.interval);

        self.fsrs_cards[word_idx] = FsrsCard {
            stability: item.memory.stability as f64,
            difficulty: item.memory.difficulty as f64,
            state: match rating {
                FsrsRating::Again => CardState::Relearning,
                _ => CardState::Review,
            },
            lapses: new_lapses,
        };

        let due_after = self.history.len().saturating_sub(1) + offset;
        if let Some(existing) = self.review.iter_mut().find(|r| r.word_idx == word_idx) {
            existing.due_after = due_after;
        } else {
            self.review.push(ReviewItem { word_idx, due_after });
        }
    }

    // queue a word for review; if already queued, keep the earlier due time
    fn schedule_review(&mut self, word_idx: usize) {
        let offset = REVIEW_LO + rand_usize(REVIEW_HI - REVIEW_LO + 1);
        let due_after = self.history.len().saturating_sub(1) + offset;

        if let Some(existing) = self.review.iter_mut().find(|r| r.word_idx == word_idx) {
            existing.due_after = existing.due_after.min(due_after);
        } else {
            self.review.push(ReviewItem { word_idx, due_after });
        }
    }

    pub fn answer(&mut self, opt_idx: usize) {
        let current = self.current;
        if current >= self.history.len() {
            return;
        }
        if self.history[current].answered || opt_idx >= self.history[current].options.len() {
            return;
        }

        let target_idx = self.history[current].target_idx;
        let is_correct = opt_idx == self.history[current].correct_opt;

        self.history[current].answered = true;
        self.history[current].selected_idx = Some(opt_idx);

        // Record answer time and auto-select rating
        let elapsed = self.history[current].question_time.elapsed();
        let answer_time_ms = elapsed.as_millis() as u64;
        self.history[current].answer_time_ms = Some(answer_time_ms);
        let rating = self.fsrs_config.auto_rating(is_correct, answer_time_ms);
        self.history[current].auto_rating = Some(rating);

        if !is_correct {
            // Wrong answer: schedule review with FSRS or fallback
            if self.fsrs_config.review_wrong {
                if self.fsrs_config.enabled {
                    self.schedule_review_fsrs(target_idx, rating);
                } else {
                    self.schedule_review(target_idx);
                }
            }
        } else {
            // Correct answer: schedule review with FSRS (or not at all for fallback)
            if self.fsrs_config.enabled {
                self.schedule_review_fsrs(target_idx, rating);
            }
        }
    }

    pub fn set_rating(&mut self, rating: FsrsRating) {
        let current = self.current;
        if current >= self.history.len() {
            return;
        }
        if !self.history[current].answered {
            return;
        }
        let old_rating = self.history[current].rating();
        self.history[current].manual_rating = Some(rating);

        // Re-schedule if rating changed (only when FSRS is active)
        if self.fsrs_config.enabled && old_rating != Some(rating) {
            let target_idx = self.history[current].target_idx;
            self.schedule_review_fsrs(target_idx, rating);
        }
    }

    pub fn skip(&mut self) {
        let current = self.current;
        if current >= self.history.len() {
            return;
        }
        if self.history[current].answered {
            return;
        }

        let target_idx = self.history[current].target_idx;

        self.history[current].answered = true;
        self.history[current].skipped = true;
        self.history[current].selected_idx = None;

        // Record time and set Again rating
        let elapsed = self.history[current].question_time.elapsed();
        let answer_time_ms = elapsed.as_millis() as u64;
        self.history[current].answer_time_ms = Some(answer_time_ms);
        self.history[current].auto_rating = Some(FsrsRating::Again);

        if self.fsrs_config.enabled {
            self.schedule_review_fsrs(target_idx, FsrsRating::Again);
        } else {
            self.schedule_review(target_idx);
        }
    }

    pub fn next(&mut self) {
        if self.history.is_empty() {
            self.gen_question();
            return;
        }

        if self.current + 1 < self.history.len() {
            self.current += 1;
        } else {
            self.gen_question();
        }
    }

    pub fn prev(&mut self) {
        self.current = self.current.saturating_sub(1);
    }
}

fn shuffle<T>(items: &mut [T]) {
    if items.len() > 1 {
        fastrand::shuffle(items);
    }
}

fn rand_bool() -> bool {
    fastrand::bool()
}

fn rand_usize(max: usize) -> usize {
    if max == 0 {
        0
    } else {
        fastrand::usize(..max)
    }
}

pub async fn sleep_ms(ms: u64) {
    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
}
