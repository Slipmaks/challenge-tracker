//! Компоненты. Тактильный неоморфизм, скруглённое SVG-кольцо, липкие шиты и клавиатурная навигация.

use crate::state::{self, Challenge, MAX_LENGTH, MIN_LENGTH};
use chrono::{Datelike, Duration, Months, NaiveDate};
use leptos::{html, prelude::*, wasm_bindgen::JsCast};

const RING_CIRCUMFERENCE: f64 = 2.0 * std::f64::consts::PI * 84.0;

#[derive(Clone, Copy, PartialEq)]
enum Confirm {
    None,
    Reset,
    Overwrite,
    Apply,
}

fn focus_later(btn: NodeRef<html::Button>) {
    request_animation_frame(move || {
        if let Some(b) = btn.get_untracked() {
            let _ = b.focus();
        }
    });
}

fn focus_grid_date(container: &NodeRef<html::Div>, attr: &str, d: NaiveDate) {
    if let Some(el) = container.get_untracked()
        && let Ok(Some(target)) = el.query_selector(&format!("[{attr}='{d}']"))
        && let Ok(html_el) = target.dyn_into::<web_sys::HtmlElement>()
    {
        let _ = html_el.focus();
    }
}

/// Общая обёртка bottom sheet (на ≥640px центрированная карточка).
#[component]
fn Sheet(
    #[prop(into)] title: String,
    #[prop(default = true)] closable: bool,
    closing: ReadSignal<bool>,
    on_close: Callback<()>,
    children: Children,
) -> impl IntoView {
    let panel = NodeRef::<html::Div>::new();
    Effect::new(move |_| {
        if let Some(p) = panel.get() {
            let _ = p.focus();
        }
    });
    view! {
        <div
            class=move || { if closing.get() { "backdrop closing" } else { "backdrop" } }
            on:click=move |_| if closable { on_close.run(()) }
            on:keydown=move |ev| if closable && ev.key() == "Escape" { on_close.run(()) }
        >
            <div
                class=move || { if closing.get() { "sheet raised closing" } else { "sheet raised" } }
                node_ref=panel
                role="dialog"
                aria-modal="true"
                aria-labelledby="sheet-title"
                tabindex="-1"
                on:click=|ev| ev.stop_propagation()
            >
                <div class="sheet-handle" aria-hidden="true" />
                <div class="sheet-head">
                    <h2 class="sheet-title" id="sheet-title">{title}</h2>
                    {closable
                        .then(|| {
                            view! {
                                <button
                                    class="icon-btn raised-sm"
                                    aria-label="Close"
                                    title="Close"
                                    on:click=move |_| on_close.run(())
                                >
                                    <svg
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="2"
                                        stroke-linecap="round"
                                        aria-hidden="true"
                                    >
                                        <path d="M6 6l12 12M18 6L6 18" />
                                    </svg>
                                </button>
                            }
                        })}
                </div>
                {children()}
            </div>
        </div>
    }
}

/// Инлайн-подтверждение: заменяет опасную/главную кнопку плашкой «вопрос · да · нет».
#[component]
fn ConfirmRow(
    #[prop(into)] text: TextProp,
    #[prop(default = "Yes")] yes: &'static str,
    #[prop(default = "No")] no: &'static str,
    #[prop(optional)] big: bool,
    on_yes: Callback<()>,
    on_no: Callback<()>,
) -> impl IntoView {
    let safe = NodeRef::<html::Button>::new();
    Effect::new(move |_| {
        if let Some(b) = safe.get() {
            let _ = b.focus();
        }
    });
    view! {
        <div class=if big { "confirm big raised" } else { "confirm raised-sm" }>
            <span class="confirm-text" role="alert">{move || text.get()}</span>
            <div class="confirm-actions">
                <button class="btn-slim raised-sm" on:click=move |_| on_yes.run(())>{yes}</button>
                <button class="btn-slim safe raised-sm" node_ref=safe on:click=move |_| on_no.run(())>{no}</button>
            </div>
        </div>
    }
}

/// Ошибка или подсказка под полем.
#[component]
fn Alert(#[prop(into)] text: TextProp) -> impl IntoView {
    view! {
        <span class="alert" role="alert">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
                <circle cx="12" cy="12" r="10" />
                <path d="M12 8v4M12 16h.01" />
            </svg>
            {move || text.get()}
        </span>
    }
}

/// `Start over` с вопросом — в шите финиша и в футере после его закрытия.
#[component]
pub fn StartOver(on_start_over: Callback<()>) -> impl IntoView {
    let (asking, set_asking) = signal(false);
    let trigger_btn = NodeRef::<html::Button>::new();
    view! {
        <Show
            when=move || asking.get()
            fallback=move || {
                view! {
                    <button class="btn raised" node_ref=trigger_btn on:click=move |_| set_asking.set(true)>
                        "Start over"
                    </button>
                }
            }
        >
            <ConfirmRow
                text="Start over from today? History stays."
                big=true
                on_yes=on_start_over
                on_no=Callback::new(move |_| {
                    set_asking.set(false);
                    focus_later(trigger_btn);
                })
            />
        </Show>
    }
}

fn day_class(c: &Challenge, d: NaiveDate, today: NaiveDate) -> String {
    let editable = c.is_editable(d, today);
    let mut cls = String::from(if c.is_done(d) {
        if editable { "done raised-sm" } else { "done" }
    } else if !editable {
        "off"
    } else if d < today {
        "missed raised-sm"
    } else {
        "raised-sm"
    });
    if d == today {
        cls.push_str(" today");
    }
    cls
}

fn day_label(d: NaiveDate, today: NaiveDate) -> String {
    let mut s = d.format("%B %-d").to_string();
    if d == today {
        s.push_str(", today");
    } else if d > today {
        s.push_str(", upcoming");
    }
    s
}

fn day_pressed(c: &Challenge, d: NaiveDate, today: NaiveDate) -> Option<&'static str> {
    let done = c.is_done(d);
    (done || c.is_editable(d, today)).then_some(if done { "true" } else { "false" })
}

#[component]
pub fn Ring(cur: Signal<Challenge>, today: ReadSignal<NaiveDate>) -> impl IntoView {
    let (pulsing, set_pulsing) = signal(false);
    let pulse_timer = StoredValue::new_local(None::<TimeoutHandle>);

    Effect::new(move |prev: Option<usize>| {
        let done = cur.get().done_count();
        if let Some(prev_done) = prev
            && done > prev_done
        {
            pulse_timer.update_value(|slot| {
                if let Some(h) = slot.take() {
                    h.clear();
                }
            });
            set_pulsing.set(true);
            if let Ok(h) = set_timeout_with_handle(
                move || set_pulsing.set(false),
                std::time::Duration::from_millis(560),
            ) {
                pulse_timer.set_value(Some(h));
            }
        }
        done
    });

    view! {
        <div class=move || { if pulsing.get() { "ring-well raised pulsing" } else { "ring-well raised" } }>
            <svg class="ring-svg" viewBox="0 0 200 200" aria-hidden="true">
                <defs>
                    <linearGradient id="ring-grad" x1="0%" y1="0%" x2="100%" y2="100%">
                        <stop offset="0%" stop-color="var(--accent-hi)" />
                        <stop offset="100%" stop-color="var(--accent)" />
                    </linearGradient>
                </defs>
                <circle class="ring-track" cx="100" cy="100" r="84" />
                <circle
                    class="ring-arc"
                    cx="100"
                    cy="100"
                    r="84"
                    stroke-dasharray=format!("{RING_CIRCUMFERENCE:.2}")
                    style=move || {
                        let c = cur.get();
                        let done = c.done_count();
                        let offset = if done == 0 {
                            RING_CIRCUMFERENCE
                        } else {
                            RING_CIRCUMFERENCE * (1.0 - c.progress_fraction())
                        };
                        let opacity = if done == 0 { 0 } else { 1 };
                        format!("stroke-dashoffset: {offset:.2}; opacity: {opacity};")
                    }
                />
            </svg>
            <div class="ring-face raised-sm">
                <div class="ring-count">
                    <span class="ring-num">{move || cur.get().done_count()}</span>
                    <span class="ring-sep">"/"</span>
                    <span class="ring-den">{move || cur.get().length}</span>
                </div>
                {move || {
                    let s = cur.get().current_streak(today.get());
                    (s > 0).then(|| {
                        view! {
                            <div class="ring-streak">
                                <svg
                                    class="streak-flame"
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="1.9"
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    aria-hidden="true"
                                >
                                    <path d="M8.5 14.5A2.5 2.5 0 0 0 11 12c0-1.38-.5-2-1-3-1.072-2.143-.224-4.054 2-6 .5 2.5 2 4.9 4 6.5 2 1.6 3 3.5 3 5.5a7 7 0 1 1-14 0c0-1.153.433-2.294 1-3a2.5 2.5 0 0 0 2.5 2.5z" />
                                </svg>
                                <span>{format!("streak {s}")}</span>
                            </div>
                        }
                    })
                }}
            </div>
        </div>
    }
}

/// Сетка дней. До 35 дней — интерактивные кнопки (roving tabindex + стрелки).
/// Свыше 35 дней — единая обзорная панель, открывающая календарь.
#[component]
pub fn DayGrid(
    cur: Signal<Challenge>,
    today: ReadSignal<NaiveDate>,
    last_toggled: ReadSignal<Option<NaiveDate>>,
    on_toggle: Callback<NaiveDate>,
    on_open: Callback<()>,
) -> impl IntoView {
    let tappable = Memo::new(move |_| cur.get().grid_tappable());
    let grid_meta = Memo::new(move |_| {
        let c = cur.get();
        (c.start, c.length, c.cols())
    });

    view! {
        <Show
            when=move || tappable.get()
            fallback=move || {
                view! {
                    <button
                        class="overview-panel raised"
                        aria-label="Open calendar"
                        on:click=move |_| on_open.run(())
                    >
                        <span class="overview-head">
                            <span class="overview-label">{move || format!("{} days", cur.get().length)}</span>
                            <span class="overview-action">
                                "Calendar"
                                <svg
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="2.2"
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    aria-hidden="true"
                                >
                                    <path d="M9 6l6 6-6 6" />
                                </svg>
                            </span>
                        </span>
                        {move || {
                            let c = cur.get();
                            let t = today.get();
                            let style = format!("--cols:{}", c.cols());
                            let cls = if c.cols() >= 12 { "grid overview dense" } else { "grid overview" };
                            let cells = (0..c.length)
                                .map(|i| {
                                    let d = c.start + Duration::days(i as i64);
                                    view! { <span class=format!("day {}", day_class(&c, d, t))></span> }
                                })
                                .collect_view();
                            view! {
                                <span class=cls style=style>
                                    {cells}
                                </span>
                            }
                        }}
                    </button>
                }
            }
        >
            {move || {
                let (start, length, cols) = grid_meta.get();
                let grid_ref = NodeRef::<html::Div>::new();
                let (focused_day, set_focused_day) = signal(None::<NaiveDate>);
                let style = format!("--cols:{cols}");

                let active_day = move || {
                    let c = cur.get();
                    let t = today.get();
                    let max_d = t.min(c.last_day()).max(c.start);
                    focused_day.get().filter(|d| c.is_editable(*d, t)).unwrap_or(max_d)
                };

                let on_keydown = move |ev: web_sys::KeyboardEvent| {
                    let c = cur.get_untracked();
                    let t = today.get_untracked();
                    let max_d = t.min(c.last_day()).max(c.start);
                    let current = focused_day
                        .get_untracked()
                        .filter(|d| c.is_editable(*d, t))
                        .unwrap_or(max_d);
                    let next = match ev.key().as_str() {
                        "ArrowRight" => current + Duration::days(1),
                        "ArrowLeft" => current - Duration::days(1),
                        "ArrowDown" => current + Duration::days(cols as i64),
                        "ArrowUp" => current - Duration::days(cols as i64),
                        "Home" => c.start,
                        "End" => max_d,
                        _ => return,
                    };
                    ev.prevent_default();
                    let clamped = next.clamp(c.start, max_d);
                    set_focused_day.set(Some(clamped));
                    focus_grid_date(&grid_ref, "data-date", clamped);
                };

                let cells = (0..length)
                    .map(|i| {
                        let d = start + Duration::days(i as i64);
                        let date_attr = d.to_string();
                        view! {
                            <button
                                class=move || {
                                    let c = cur.get();
                                    format!("day {}", day_class(&c, d, today.get()))
                                }
                                data-date=date_attr
                                tabindex=move || { if d == active_day() { "0" } else { "-1" } }
                                aria-label=move || day_label(d, today.get())
                                aria-pressed=move || day_pressed(&cur.get(), d, today.get())
                                disabled=move || !cur.get().is_editable(d, today.get())
                                on:focus=move |_| set_focused_day.set(Some(d))
                                on:click=move |_| {
                                    set_focused_day.set(Some(d));
                                    on_toggle.run(d);
                                }
                            >
                                {move || {
                                    let c = cur.get();
                                    c.is_done(d).then(|| {
                                        let pop = last_toggled.get() == Some(d);
                                        view! {
                                            <svg
                                                class=if pop { "day-tick pop" } else { "day-tick" }
                                                viewBox="0 0 24 24"
                                                fill="none"
                                                stroke="currentColor"
                                                stroke-width="2.8"
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                                aria-hidden="true"
                                            >
                                                <path d="M5 12.5l4.5 4.5L19 7.5" />
                                            </svg>
                                        }
                                    })
                                }}
                            </button>
                        }
                    })
                    .collect_view();

                view! {
                    <div
                        class="grid"
                        node_ref=grid_ref
                        style=style
                        role="group"
                        aria-label="Days"
                        on:keydown=on_keydown
                    >
                        {cells}
                    </div>
                }
            }}
        </Show>
    }
}

#[component]
pub fn CalendarSheet(
    cur: Signal<Challenge>,
    today: ReadSignal<NaiveDate>,
    last_toggled: ReadSignal<Option<NaiveDate>>,
    closing: ReadSignal<bool>,
    on_toggle: Callback<NaiveDate>,
    on_close: Callback<()>,
) -> impl IntoView {
    let t0 = today.get_untracked();
    let first_of_month = NaiveDate::from_ymd_opt(t0.year(), t0.month(), 1).unwrap();
    let (month, set_month) = signal(first_of_month);
    let (month_dir, set_month_dir) = signal(0i8);
    let (month_seq, set_month_seq) = signal(0u32);
    let (focused_day, set_focused_day) = signal(None::<NaiveDate>);
    let cal_ref = NodeRef::<html::Div>::new();

    let today_month = move || {
        let t = today.get();
        NaiveDate::from_ymd_opt(t.year(), t.month(), 1).unwrap()
    };

    let shift = move |forward: bool| {
        set_month_dir.set(if forward { 1 } else { -1 });
        set_month_seq.update(|s| *s = s.wrapping_add(1));
        set_month.update(|m| {
            *m = if forward {
                m.checked_add_months(Months::new(1))
            } else {
                m.checked_sub_months(Months::new(1))
            }
            .unwrap_or(*m);
        });
    };

    let jump_today = move || {
        let target = today_month();
        let cur_m = month.get_untracked();
        if target != cur_m {
            set_month_dir.set(if target > cur_m { 1 } else { -1 });
            set_month_seq.update(|s| *s = s.wrapping_add(1));
            set_month.set(target);
        }
    };

    let editable_bounds = move || {
        let c = cur.get();
        let t = today.get();
        let m = month.get();
        let days = m
            .checked_add_months(Months::new(1))
            .map(|n| (n - m).num_days())
            .unwrap_or(30);
        let month_end = m + Duration::days(days - 1);
        let first_ed = m.max(c.start);
        let last_ed = month_end.min(t.min(c.last_day()));
        (first_ed <= last_ed).then_some((first_ed, last_ed))
    };

    let active_day = move || {
        let Some((first_ed, last_ed)) = editable_bounds() else {
            return None;
        };
        let t = today.get();
        let default = if t >= first_ed && t <= last_ed { t } else { last_ed };
        Some(
            focused_day
                .get()
                .filter(|d| *d >= first_ed && *d <= last_ed)
                .unwrap_or(default),
        )
    };

    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        let Some((first_ed, last_ed)) = editable_bounds() else {
            return;
        };
        let current = active_day().unwrap_or(last_ed);
        let next = match ev.key().as_str() {
            "ArrowRight" => current + Duration::days(1),
            "ArrowLeft" => current - Duration::days(1),
            "ArrowDown" => current + Duration::days(7),
            "ArrowUp" => current - Duration::days(7),
            "Home" => first_ed,
            "End" => last_ed,
            _ => return,
        };
        ev.prevent_default();
        let clamped = next.clamp(first_ed, last_ed);
        set_focused_day.set(Some(clamped));
        focus_grid_date(&cal_ref, "data-cal-date", clamped);
    };

    view! {
        <Sheet title="Calendar" closing=closing on_close=on_close>
            <div class="sheet-body">
                <div class="cal-head">
                    <button
                        class="icon-btn raised-sm"
                        aria-label="Previous month"
                        title="Previous month"
                        on:click=move |_| shift(false)
                    >
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
                            <path d="M15 5l-7 7 7 7" />
                        </svg>
                    </button>
                    <div class="cal-center">
                        <span class="cal-month" aria-live="polite">
                            {move || month.get().format("%B %Y").to_string()}
                        </span>
                        <Show when=move || month.get() != today_month()>
                            <button class="cal-today-btn raised-sm" on:click=move |_| jump_today()>
                                "Today"
                            </button>
                        </Show>
                    </div>
                    <button
                        class="icon-btn raised-sm"
                        aria-label="Next month"
                        title="Next month"
                        on:click=move |_| shift(true)
                    >
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
                            <path d="M9 5l7 7-7 7" />
                        </svg>
                    </button>
                </div>
                <div
                    class=move || {
                        let seq = month_seq.get();
                        match (month_dir.get(), seq % 2) {
                            (1, 0) => "cal-grid slide-next-a",
                            (1, _) => "cal-grid slide-next-b",
                            (-1, 0) => "cal-grid slide-prev-a",
                            (-1, _) => "cal-grid slide-prev-b",
                            _ => "cal-grid",
                        }
                    }
                    node_ref=cal_ref
                    on:keydown=on_keydown
                >
                    {["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"]
                        .map(|d| view! { <div class="cal-dow">{d}</div> })
                        .collect_view()}
                    {(0i64..42)
                        .map(|slot| {
                            let slot_date = Memo::new(move |_| {
                                let m = month.get();
                                let lead = m.weekday().num_days_from_monday() as i64;
                                let days = m
                                    .checked_add_months(Months::new(1))
                                    .map(|n| (n - m).num_days())
                                    .unwrap_or(30);
                                let idx = slot - lead;
                                (idx >= 0 && idx < days).then(|| m + Duration::days(idx))
                            });
                            move || match slot_date.get() {
                                None => view! { <div class="cal-cell cal-blank" /> }.into_any(),
                                Some(d) => {
                                    let date_attr = d.to_string();
                                    view! {
                                        <button
                                            class=move || {
                                                let c = cur.get();
                                                let mut cls = format!("cal-cell {}", day_class(&c, d, today.get()));
                                                if c.is_done(d) && last_toggled.get() == Some(d) {
                                                    cls.push_str(" just-done");
                                                }
                                                cls
                                            }
                                            data-cal-date=date_attr
                                            tabindex=move || {
                                                if active_day() == Some(d) { "0" } else { "-1" }
                                            }
                                            aria-label=move || day_label(d, today.get())
                                            aria-pressed=move || day_pressed(&cur.get(), d, today.get())
                                            disabled=move || !cur.get().is_editable(d, today.get())
                                            on:focus=move |_| set_focused_day.set(Some(d))
                                            on:click=move |_| {
                                                set_focused_day.set(Some(d));
                                                on_toggle.run(d);
                                            }
                                        >
                                            <span class="cal-day-num">{d.day()}</span>
                                        </button>
                                    }
                                        .into_any()
                                }
                            }
                        })
                        .collect_view()}
                </div>
            </div>
        </Sheet>
    }
}

#[component]
pub fn FinishSheet(
    cur: Signal<Challenge>,
    closing: ReadSignal<bool>,
    on_start_over: Callback<()>,
    on_close: Callback<()>,
) -> impl IntoView {
    view! {
        <Sheet title="Challenge complete" closing=closing on_close=on_close>
            <div class="sheet-body">
                <div class="summary">
                    <div class="trophy-medallion raised">
                        <svg
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            aria-hidden="true"
                        >
                            <path d="M6 9H4.5a2.5 2.5 0 0 1 0-5H6" />
                            <path d="M18 9h1.5a2.5 2.5 0 0 0 0-5H18" />
                            <path d="M4 22h16" />
                            <path d="M10 14.7V17c0 .6-.5 1-1 1.2C7.9 18.8 7 20.2 7 22" />
                            <path d="M14 14.7V17c0 .6.5 1 1 1.2 1.1.6 2 2 2 3.8" />
                            <path d="M18 2H6v7a6 6 0 0 0 12 0V2Z" />
                        </svg>
                    </div>
                    <div class="summary-score">
                        {move || {
                            let c = cur.get();
                            format!("{} of {}", c.done_count(), c.length)
                        }}
                    </div>
                    <div class="summary-streak">
                        {move || {
                            let c = cur.get();
                            let pct = c.percent();
                            match c.best_streak() {
                                0 => format!("{pct}% completed"),
                                best => format!("{pct}% completed · best streak {best}"),
                            }
                        }}
                    </div>
                </div>
            </div>
            <div class="sheet-foot">
                <StartOver on_start_over />
            </div>
        </Sheet>
    }
}

/// Настройки и онбординг — один компонент с липким футером для Start / Done.
#[component]
pub fn SettingsSheet(
    cur: Signal<Challenge>,
    today: ReadSignal<NaiveDate>,
    closing: ReadSignal<bool>,
    #[prop(default = false)] first_run: bool,
    set_challenge: WriteSignal<Option<Challenge>>,
    on_close: Callback<()>,
    #[prop(optional)] on_length: Option<Callback<u32>>,
    #[prop(optional)] on_name: Option<Callback<String>>,
    #[prop(optional)] on_first_commit: Option<Callback<Challenge>>,
    #[prop(optional)] on_reset_instant: Option<Callback<()>>,
) -> impl IntoView {
    let c0 = cur.get_untracked();
    let (name, set_name) = signal(if first_run { String::new() } else { c0.name.clone() });
    let is_pill = [30, 60, 100].contains(&c0.length);
    let (pill, set_pill) = signal(is_pill.then_some(c0.length));
    let (custom, set_custom) = signal(if is_pill { String::new() } else { c0.length.to_string() });
    let length = Memo::new(move |_| state::form_length(pill.get(), &custom.get()));

    if let Some(cb) = on_length {
        Effect::new(move |_| {
            if let Some(n) = length.get() {
                cb.run(n);
            }
        });
    }
    if let Some(cb) = on_name {
        Effect::new(move |_| {
            cb.run(name.get());
        });
    }

    let (start, set_start) = signal(c0.start.to_string());
    let (paste, set_paste) = signal(String::new());
    let (bad_json, set_bad_json) = signal(None::<&'static str>);
    let (confirm, set_confirm) = signal(Confirm::None);
    let (import_open, set_import_open) = signal(false);
    let (exported, set_exported) = signal(false);
    let export_timer = StoredValue::new_local(None::<TimeoutHandle>);

    let import_btn = NodeRef::<html::Button>::new();
    let reset_btn = NodeRef::<html::Button>::new();

    let edited = move || {
        let t = today.get_untracked();
        let mut c = cur.get_untracked();
        c.name = name.get_untracked();
        c.length = length.get_untracked().unwrap_or(c.length);
        c.start = NaiveDate::parse_from_str(&start.get_untracked(), "%Y-%m-%d").unwrap_or(c.start);
        c.sanitize(t)
    };

    let commit = move |c: Challenge| {
        set_challenge.update(|slot| {
            if slot.is_some() {
                *slot = Some(c);
            }
        });
    };

    let create = move || {
        let t = today.get_untracked();
        let s = NaiveDate::parse_from_str(&start.get_untracked(), "%Y-%m-%d").unwrap_or(t);
        let new_c = Challenge::new(
            name.get_untracked(),
            s,
            length.get_untracked().unwrap_or(c0.length),
            t,
        );
        if let Some(cb) = on_first_commit {
            cb.run(new_c);
        } else {
            set_challenge.set(Some(new_c));
            on_close.run(());
        }
    };

    let done = Callback::new(move |_| {
        let t = today.get_untracked();
        let c = edited();
        if cur.get_untracked().edit_warning(&c, t).is_some() {
            set_confirm.set(Confirm::Apply);
            return;
        }
        commit(c);
        on_close.run(());
    });

    let apply = Callback::new(move |_| {
        commit(edited());
        on_close.run(());
    });

    let keep = Callback::new(move |_| {
        let t = today.get_untracked();
        let mut c = cur.get_untracked();
        c.name = name.get_untracked();
        commit(c.sanitize(t));
        on_close.run(());
    });

    let cancel_import = Callback::new(move |_| {
        set_confirm.set(Confirm::None);
        focus_later(import_btn);
    });
    let cancel_reset = Callback::new(move |_| {
        set_confirm.set(Confirm::None);
        focus_later(reset_btn);
    });

    let warning = move || {
        length.track();
        start.track();
        let t = today.get();
        (confirm.get() == Confirm::Apply)
            .then(|| cur.get_untracked().edit_warning(&edited(), t))
            .flatten()
    };

    let parsed = move || state::parse_backup(&paste.get_untracked());

    let apply_import = move || match parsed() {
        Ok(c) => {
            let t = today.get_untracked();
            let clean = c.sanitize(t);
            if first_run {
                if let Some(cb) = on_first_commit {
                    cb.run(clean);
                } else {
                    set_challenge.set(Some(clean));
                    on_close.run(());
                }
            } else {
                set_challenge.set(Some(clean));
                on_close.run(());
            }
        }
        Err(e) => set_bad_json.set(Some(e)),
    };

    let marked = move || cur.get_untracked().done.len();
    let replace_text = move || {
        let label = state::short_label(&cur.get_untracked().name, 28);
        match marked() {
            0 => format!("Replace “{label}” with the backup?"),
            n => format!("Replace “{label}” ({n} marked) with the backup?"),
        }
    };
    let erase_text = move || {
        let label = state::short_label(&cur.get_untracked().name, 28);
        match marked() {
            0 => format!("Erase “{label}”?"),
            1 => format!("Erase “{label}” and 1 marked day?"),
            n => format!("Erase “{label}” and {n} marked days?"),
        }
    };

    let import_click = move |_| {
        set_bad_json.set(None);
        match parsed() {
            Ok(_) if !first_run => set_confirm.set(Confirm::Overwrite),
            Ok(_) => apply_import(),
            Err(e) => set_bad_json.set(Some(e)),
        }
    };

    let reset = Callback::new(move |_| {
        state::clear();
        set_challenge.set(None);
        if let Some(cb) = on_reset_instant {
            cb.run(());
        } else {
            on_close.run(());
        }
    });

    let on_export_click = move |_| {
        export_timer.update_value(|slot| {
            if let Some(h) = slot.take() {
                h.clear();
            }
        });
        set_exported.set(true);
        if let Ok(h) = set_timeout_with_handle(
            move || set_exported.set(false),
            std::time::Duration::from_millis(2400),
        ) {
            export_timer.set_value(Some(h));
        }
    };

    view! {
        <Sheet
            title=if first_run { "New challenge" } else { "Settings" }
            closable=!first_run
            closing=closing
            on_close=on_close
        >
            <div class="sheet-body">
                <label class="field">
                    <span class="field-label">"Name"</span>
                    <input
                        class="input raised-sm"
                        type="text"
                        placeholder="Challenge name"
                        prop:value=move || name.get()
                        on:input=move |ev| set_name.set(event_target_value(&ev))
                    />
                </label>

                <div class="field" role="group" aria-label="Length">
                    <span class="field-label" aria-hidden="true">"Length"</span>
                    <div class="pills">
                        {[30u32, 60, 100]
                            .map(|n| {
                                view! {
                                    <button
                                        class=move || {
                                            if pill.get() == Some(n) { "pill on raised-sm" } else { "pill raised-sm" }
                                        }
                                        aria-pressed=move || { if pill.get() == Some(n) { "true" } else { "false" } }
                                        on:click=move |_| {
                                            set_pill.set(Some(n));
                                            set_custom.set(String::new());
                                        }
                                    >
                                        {n}
                                    </button>
                                }
                            })
                            .collect_view()}
                        <input
                            class=move || {
                                if !custom.get().trim().is_empty() && length.get().is_some() {
                                    "input pill-custom on raised-sm"
                                } else {
                                    "input pill-custom raised-sm"
                                }
                            }
                            type="text"
                            inputmode="numeric"
                            pattern="[0-9]*"
                            autocomplete="off"
                            placeholder="Custom"
                            aria-label="Custom length"
                            prop:value=move || custom.get()
                            on:input=move |ev| {
                                let v = event_target_value(&ev);
                                if !v.trim().is_empty() {
                                    set_pill.set(None);
                                }
                                set_custom.set(v);
                            }
                        />
                    </div>
                    <Show when=move || length.get().is_none()>
                        <Alert text=format!("Choose {MIN_LENGTH}–{MAX_LENGTH} days") />
                    </Show>
                </div>

                <label class="field">
                    <span class="field-label">"Start date"</span>
                    <input
                        class="input raised-sm"
                        type="date"
                        max=move || today.get().to_string()
                        prop:value=move || start.get()
                        on:input=move |ev| set_start.set(event_target_value(&ev))
                    />
                </label>

                {(!first_run)
                    .then(|| {
                        view! {
                            <a
                                class=move || {
                                    if exported.get() {
                                        "btn-secondary btn-exported raised-sm sep"
                                    } else {
                                        "btn-secondary raised-sm sep"
                                    }
                                }
                                download="challenge.json"
                                href=move || {
                                    let json = serde_json::to_string(&cur.get()).unwrap_or_default();
                                    format!(
                                        "data:application/json;charset=utf-8,{}",
                                        String::from(js_sys::encode_uri_component(&json)),
                                    )
                                }
                                on:click=on_export_click
                            >
                                {move || {
                                    if exported.get() {
                                        view! {
                                            <svg
                                                class="btn-inline-icon"
                                                viewBox="0 0 24 24"
                                                fill="none"
                                                stroke="currentColor"
                                                stroke-width="2.2"
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                                aria-hidden="true"
                                            >
                                                <path d="M5 12.5l4.5 4.5L19 7.5" />
                                            </svg>
                                            <span>"Backup downloaded"</span>
                                        }
                                            .into_any()
                                    } else {
                                        view! { <span>"Export JSON"</span> }.into_any()
                                    }
                                }}
                            </a>
                        }
                    })}

                <div class="disclosure-group">
                    <button
                        class="btn-secondary btn-disclosure raised-sm"
                        aria-expanded=move || { if import_open.get() { "true" } else { "false" } }
                        on:click=move |_| set_import_open.update(|v| *v = !*v)
                    >
                        <span>"Import JSON"</span>
                        <svg
                            class=move || { if import_open.get() { "chevron open" } else { "chevron" } }
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            aria-hidden="true"
                        >
                            <path d="M6 9l6 6 6-6" />
                        </svg>
                    </button>
                    <div
                        class=move || { if import_open.get() { "collapse open" } else { "collapse" } }
                        inert=move || !import_open.get()
                    >
                        <div class="collapse-inner">
                            <div class="field import-field">
                                <label class="field-label" for="import-json-area">"Paste backup JSON"</label>
                                <textarea
                                    id="import-json-area"
                                    class="input raised-sm"
                                    placeholder="Paste JSON here"
                                    prop:value=move || paste.get()
                                    on:input=move |ev| set_paste.set(event_target_value(&ev))
                                />
                                {move || bad_json.get().map(|e| view! { <Alert text=e /> })}
                                {move || match confirm.get() {
                                    Confirm::Overwrite => {
                                        view! {
                                            <ConfirmRow
                                                text=replace_text()
                                                on_yes=Callback::new(move |_| apply_import())
                                                on_no=cancel_import
                                            />
                                        }
                                            .into_any()
                                    }
                                    _ => {
                                        view! {
                                            <button class="btn-secondary raised-sm" node_ref=import_btn on:click=import_click>
                                                "Import"
                                            </button>
                                        }
                                            .into_any()
                                    }
                                }}
                            </div>
                        </div>
                    </div>
                </div>

                {(!first_run)
                    .then(|| {
                        view! {
                            <div class="sep">
                                {move || match confirm.get() {
                                    Confirm::Reset => {
                                        view! {
                                            <ConfirmRow text=erase_text() on_yes=reset on_no=cancel_reset />
                                        }
                                            .into_any()
                                    }
                                    _ => {
                                        view! {
                                            <button
                                                class="btn-secondary btn-danger raised-sm"
                                                node_ref=reset_btn
                                                on:click=move |_| set_confirm.set(Confirm::Reset)
                                            >
                                                <svg
                                                    class="btn-inline-icon"
                                                    viewBox="0 0 24 24"
                                                    fill="none"
                                                    stroke="currentColor"
                                                    stroke-width="2"
                                                    stroke-linecap="round"
                                                    stroke-linejoin="round"
                                                    aria-hidden="true"
                                                >
                                                    <path d="M3 12a9 9 0 1 0 3-6.7L3 8" />
                                                    <path d="M3 3v5h5" />
                                                </svg>
                                                <span>"Reset everything"</span>
                                            </button>
                                        }
                                            .into_any()
                                    }
                                }}
                            </div>
                        }
                    })}
            </div>

            <div class="sheet-foot">
                {first_run
                    .then(|| {
                        view! {
                            <button
                                class="btn raised"
                                disabled=move || length.get().is_none()
                                on:click=move |_| create()
                            >
                                "Start"
                            </button>
                        }
                    })}

                {(!first_run)
                    .then(|| {
                        view! {
                            <Show
                                when=move || warning().is_some()
                                fallback=move || {
                                    view! {
                                        <button
                                            class="btn raised"
                                            disabled=move || length.get().is_none()
                                            on:click=move |_| done.run(())
                                        >
                                            "Done"
                                        </button>
                                    }
                                }
                            >
                                <ConfirmRow
                                    text=move || warning().unwrap_or_default()
                                    yes="Apply"
                                    no="Keep"
                                    big=true
                                    on_yes=apply
                                    on_no=keep
                                />
                            </Show>
                        }
                    })}
            </div>
        </Sheet>
    }
}
