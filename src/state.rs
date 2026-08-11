//! Модель челленджа и вся чистая логика.
//!
//! Ничего производного не храним: серия, счётчик, процент и признак финиша — функции от
//! `Challenge` + `today`. Все они принимают `today` аргументом, а не зовут `Local::now()`,
//! поэтому проверяются обычным `cargo test` на нативном таргете, без браузера и wasm-раннера.

use chrono::{Duration, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const MIN_LENGTH: u32 = 1;
pub const MAX_LENGTH: u32 = 365;
pub const DEFAULT_NAME: &str = "My Challenge";

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Challenge {
    pub name: String,
    pub start: NaiveDate,
    pub length: u32,
    /// ТОЛЬКО выполненные дни: отсутствие даты = не выполнено. «Пропуск» получается бесплатно.
    pub done: BTreeSet<NaiveDate>,
}

impl Challenge {
    pub fn new(name: String, start: NaiveDate, length: u32, today: NaiveDate) -> Self {
        Self { name, start, length, done: BTreeSet::new() }.sanitize(today)
    }

    /// Единственное место, где чинятся невозможные значения. Зовётся при сохранении настроек
    /// и при импорте JSON — второе и есть настоящая граница доверия.
    ///
    /// `length = 0` дал бы деление на ноль в проценте кольца, большое N повесило бы браузер
    /// числом клеток, старт в будущем сделал бы `day_number` нулевым или отрицательным.
    pub fn sanitize(mut self, today: NaiveDate) -> Self {
        self.length = self.length.clamp(MIN_LENGTH, MAX_LENGTH);
        if self.start > today {
            self.start = today;
        }
        if self.name.trim().is_empty() {
            self.name = DEFAULT_NAME.to_string();
        }
        self
    }

    pub fn last_day(&self) -> NaiveDate {
        self.start + Duration::days(self.length as i64 - 1)
    }

    pub fn day_number(&self, today: NaiveDate) -> i64 {
        (today - self.start).num_days() + 1
    }

    pub fn is_finished(&self, today: NaiveDate) -> bool {
        self.day_number(today) > self.length as i64
    }

    /// Отвечает на вопрос «этот день вообще можно переключить». Не украшение UI, а логика:
    /// она одна защищает `done` от мусора вроде отмеченного завтра.
    pub fn is_editable(&self, d: NaiveDate, today: NaiveDate) -> bool {
        d >= self.start && d <= today.min(self.last_day())
    }

    pub fn is_done(&self, d: NaiveDate) -> bool {
        self.done.contains(&d)
    }

    /// Единственная точка записи в `done`, поэтому проверка диапазона живёт здесь, а не в UI.
    pub fn toggle(&mut self, d: NaiveDate, today: NaiveDate) {
        if !self.is_editable(d, today) {
            return;
        }
        if !self.done.remove(&d) {
            self.done.insert(d);
        }
    }

    pub fn done_count(&self) -> usize {
        self.done.range(self.start..=self.last_day()).count()
    }

    pub fn percent(&self) -> u32 {
        // length ≥ 1 гарантирован sanitize, деления на ноль быть не может
        (self.done_count() as u32 * 100) / self.length
    }

    /// Считаем назад от сегодня, если сегодня отмечено, иначе от вчера. Если считать
    /// «включая сегодня», каждое утро до отметки на экране был бы 0 и приложение выглядело
    /// бы сломанным. Ниже `start` не уходим: метрики ограничены окном челленджа.
    pub fn current_streak(&self, today: NaiveDate) -> u32 {
        let mut d = if self.done.contains(&today) { today } else { today - Duration::days(1) };
        let mut n = 0;
        while d >= self.start && self.done.contains(&d) {
            n += 1;
            d -= Duration::days(1);
        }
        n
    }

    /// Тоже окном `[start, last_day]`: иначе после `Start over` на экране оказался бы рекорд
    /// из прошлого цикла, который по сетке не воспроизводится.
    pub fn best_streak(&self) -> u32 {
        let mut best = 0;
        let mut run = 0;
        let mut prev = None;
        for &d in self.done.range(self.start..=self.last_day()) {
            run = if prev == Some(d - Duration::days(1)) { run + 1 } else { 1 };
            best = best.max(run);
            prev = Some(d);
        }
        best
    }

    /// Число колонок от N: клетка всегда квадратная и сетка всегда влезает в экран.
    /// Ниже 40px клетка перестаёт быть кнопкой — правка прошлого уходит в календарь.
    ///
    /// Границы ярусов взяты не на глаз: на 390px под сетку остаётся ~300px, поэтому у каждого
    /// яруса верхнее N — это последнее, при котором `ряды × клетка + зазоры` ещё влезает.
    /// 12 колонок держат до 120 дней (10 рядов по 26px = 298px), дальше нужны 22.
    pub fn cols(&self) -> u32 {
        match self.length {
            0..=35 => 7,
            36..=63 => 9,
            64..=120 => 12,
            _ => 22,
        }
    }

    /// Тап по клетке работает только на ярусе 44px. См. `cols`.
    pub fn grid_tappable(&self) -> bool {
        self.cols() == 7
    }
}

// ── Хранилище: один ключ, ~15 строк. Без gloo-storage и без leptos-use ────────────────

const KEY: &str = "challenge-tracker";

fn storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

pub fn load() -> Option<Challenge> {
    serde_json::from_str(&storage()?.get_item(KEY).ok()??).ok()
}

pub fn save(c: &Challenge) {
    if let (Some(s), Ok(json)) = (storage(), serde_json::to_string(c)) {
        let _ = s.set_item(KEY, &json);
    }
}

pub fn clear() {
    if let Some(s) = storage() {
        let _ = s.remove_item(KEY);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> NaiveDate {
        s.parse().unwrap()
    }

    const TODAY: &str = "2026-08-10";

    /// Та же фикстура, что на макете в Pencil: старт 25.07.2026, N = 30, сегодня — день 17,
    /// выполнено 13 дней, пропущены дни 4, 9 и 12, сегодня не отмечено.
    fn mock() -> Challenge {
        let start = d("2026-07-25");
        let mut c = Challenge::new("Perfect Posture".into(), start, 30, d(TODAY));
        for n in [1, 2, 3, 5, 6, 7, 8, 10, 11, 13, 14, 15, 16] {
            c.done.insert(start + Duration::days(n - 1));
        }
        c
    }

    #[test]
    fn mock_matches_the_design() {
        let c = mock();
        assert_eq!(c.done_count(), 13);
        assert_eq!(c.day_number(d(TODAY)), 17);
        assert_eq!(c.percent(), 43);
        // Ловушка серии: сегодня пусто, но на экране 4, потому что считаем от вчера
        assert_eq!(c.current_streak(d(TODAY)), 4);
        assert_eq!(c.best_streak(), 4);
        assert!(!c.is_finished(d(TODAY)));
    }

    #[test]
    fn streak_includes_today_when_marked() {
        let mut c = mock();
        c.toggle(d(TODAY), d(TODAY));
        assert_eq!(c.current_streak(d(TODAY)), 5);
        assert_eq!(c.done_count(), 14);
    }

    #[test]
    fn streak_is_zero_when_yesterday_is_empty_too() {
        let start = d("2026-08-01");
        let c = Challenge::new("x".into(), start, 30, d(TODAY));
        assert_eq!(c.current_streak(d(TODAY)), 0);
    }

    #[test]
    fn streak_stops_at_start_not_at_previous_cycle() {
        // День перед стартом отмечен (остался от прошлого цикла) — в серию он не входит
        let start = d("2026-08-05");
        let mut c = Challenge::new("x".into(), start, 30, d(TODAY));
        for day in ["2026-08-03", "2026-08-04", "2026-08-05", "2026-08-06"] {
            c.done.insert(d(day));
        }
        assert_eq!(c.current_streak(d("2026-08-06")), 2);
        assert_eq!(c.best_streak(), 2);
        assert_eq!(c.done_count(), 2);
    }

    #[test]
    fn day_number_and_finish_on_the_boundaries() {
        let start = d("2026-07-25");
        let c = Challenge::new("x".into(), start, 30, d(TODAY));
        assert_eq!(c.day_number(start), 1);
        assert_eq!(c.day_number(c.last_day()), 30);
        assert!(!c.is_finished(c.last_day()));
        assert!(c.is_finished(c.last_day() + Duration::days(1)));
    }

    #[test]
    fn editable_only_inside_the_window_up_to_today() {
        let start = d("2026-07-25");
        let today = d(TODAY);
        let c = Challenge::new("x".into(), start, 30, today);
        assert!(!c.is_editable(start - Duration::days(1), today));
        assert!(!c.is_editable(start + Duration::days(30), today));
        assert!(!c.is_editable(today + Duration::days(1), today));
        assert!(c.is_editable(today, today));
        assert!(c.is_editable(start, today));
    }

    #[test]
    fn toggle_refuses_dates_outside_the_window() {
        let today = d(TODAY);
        let mut c = mock();
        c.toggle(today + Duration::days(1), today);
        c.toggle(c.start - Duration::days(1), today);
        c.toggle(c.last_day() + Duration::days(1), today);
        assert_eq!(c.done.len(), 13, "в done не должно попасть ничего лишнего");
    }

    #[test]
    fn sanitize_clamps_length_and_pulls_start_back() {
        let today = d(TODAY);
        let zero = Challenge::new("x".into(), today, 0, today);
        assert_eq!(zero.length, MIN_LENGTH);
        let huge = Challenge::new("x".into(), today, 10_000, today);
        assert_eq!(huge.length, MAX_LENGTH);
        let future = Challenge::new("x".into(), today + Duration::days(5), 30, today);
        assert_eq!(future.start, today, "старт в будущем даёт Day 0, поэтому зажимаем");
        assert_eq!(future.day_number(today), 1);
    }

    #[test]
    fn empty_name_becomes_the_default() {
        let c = Challenge::new("   ".into(), d(TODAY), 30, d(TODAY));
        assert_eq!(c.name, DEFAULT_NAME);
    }

    #[test]
    fn columns_scale_with_length() {
        let today = d(TODAY);
        let cols = |n| Challenge::new("x".into(), today, n, today).cols();
        assert_eq!(cols(30), 7);
        assert_eq!(cols(35), 7);
        assert_eq!(cols(36), 9);
        assert_eq!(cols(60), 9);
        assert_eq!(cols(100), 12);
        assert_eq!(cols(120), 12);
        assert_eq!(cols(121), 22);
        assert_eq!(cols(365), 22);
        assert!(Challenge::new("x".into(), today, 30, today).grid_tappable());
        assert!(!Challenge::new("x".into(), today, 60, today).grid_tappable());
    }

    #[test]
    fn start_over_keeps_history_but_resets_the_numbers() {
        let today = d(TODAY);
        let mut c = mock();
        let old = c.done.len();
        c.start = today; // ровно то, что делает Start over
        assert_eq!(c.done.len(), old, "историю Start over не трёт");
        assert_eq!(c.done_count(), 0, "но в окно нового челленджа она не попадает");
        assert_eq!(c.best_streak(), 0);
        assert_eq!(c.day_number(today), 1);
    }

    #[test]
    fn json_roundtrip_stays_human_readable() {
        let c = mock();
        let json = serde_json::to_string(&c).unwrap();
        assert!(json.contains("\"2026-07-25\""), "даты должны читаться руками: {json}");
        assert_eq!(serde_json::from_str::<Challenge>(&json).unwrap(), c);
    }
}
