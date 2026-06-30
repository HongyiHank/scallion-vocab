use serde::{Deserialize, Serialize};

pub(crate) const OPT_COUNT: usize = 4;
pub(crate) const REVIEW_LO: usize = 8;
pub(crate) const REVIEW_HI: usize = 15;

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
}

#[derive(Clone, Debug)]
pub struct ReviewItem {
    pub word_idx: usize,
    pub due_after: usize,
}

#[derive(Clone)]
pub struct QuizState {
    pub words: Vec<Word>,
    unseen: Vec<usize>,
    pub history: Vec<HistoryItem>,
    pub review: Vec<ReviewItem>,
    pub current: usize,
    pub infinite: bool,
}

impl QuizState {
    pub fn new(words: Vec<Word>, infinite: bool) -> Self {
        let n = words.len();
        let mut unseen: Vec<usize> = (0..n).collect();
        shuffle(&mut unseen);
        Self { words, unseen, history: vec![], review: vec![], current: 0, infinite }
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

        let target = if self.infinite {
            match self
                .review
                .iter()
                .position(|r| self.history.len() >= r.due_after)
            {
                Some(idx) => self.review.remove(idx).word_idx,
                None => self.pick_new(),
            }
        } else {
            if self.unseen.is_empty() {
                return false;
            }
            self.unseen.pop().unwrap_or(0)
        };

        // 構建選項：target + 其他不重複且不同英文的單字
        // ponytail: 排除同 front（英文）的詞，避免同一英文多種中文出現在選項中混淆
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
        });

        self.current = self.history.len().saturating_sub(1);
        true
    }

    pub fn has_more(&self) -> bool {
        self.infinite || self.current + 1 < self.history.len() || !self.unseen.is_empty()
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
        let should_review = opt_idx != self.history[current].correct_opt;

        self.history[current].answered = true;
        self.history[current].selected_idx = Some(opt_idx);

        if should_review {
            self.schedule_review(target_idx);
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

        self.schedule_review(target_idx);
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
