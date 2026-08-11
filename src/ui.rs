//! Компоненты. Вся композиция, состояния и строки — design.md.

use crate::state::{self, Challenge, MAX_LENGTH, MIN_LENGTH};
use chrono::{Datelike, Duration, Months, NaiveDate};
use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq)]
enum Confirm {
    None,
    Reset,
    Overwrite,
}

/// Общая обёртка bottom sheet.
///
/// Тап по затемнению закрывает, тап внутри панели — нет: клик внутри останавливается на самой
/// панели и до затемнения не доходит. Иначе нативный `<input type="date">` закрывал бы шит
/// прямо во время выбора даты.
#[component]
fn Sheet(
    #[prop(into)] title: String,
    #[prop(default = true)] closable: bool,
    on_close: Callback<()>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="backdrop" on:click=move |_| if closable { on_close.run(()) }>
            <div class="sheet" on:click=|ev| ev.stop_propagation()>
                <div class="sheet-head">
                    <h2 class="sheet-title">{title}</h2>
                    {closable
                        .then(|| {
                            view! {
                                <button
                                    class="icon-btn raised-sm"
                                    aria-label="Close"
                                    on:click=move |_| on_close.run(())
                                >
                                    <svg
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="2"
                                        stroke-linecap="round"
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

#[component]
pub fn Ring(cur: Signal<Challenge>, today: NaiveDate) -> impl IntoView {
    view! {
        <div class="ring-well pressed">
            // --p — один из ровно двух инлайн-стилей в проекте (второй --cols в DayGrid)
            <div class="ring" style=move || format!("--p:{}", cur.get().percent())>
                <div class="ring-face">
                    <div class="ring-count">
                        {move || {
                            let c = cur.get();
                            format!("{}/{}", c.done_count(), c.length)
                        }}
                    </div>
                    <div class="ring-streak">
                        {move || format!("streak {}", cur.get().current_streak(today))}
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn DayGrid(cur: Signal<Challenge>, today: NaiveDate, on_toggle: Callback<NaiveDate>) -> impl IntoView {
    view! {
        <div
            // dense с 12 колонок: на 9 колонках клетка ещё 32px и галочка в неё влезает
            class=move || { if cur.get().cols() >= 12 { "grid dense" } else { "grid" } }
            style=move || format!("--cols:{}", cur.get().cols())
        >
            {move || {
                let c = cur.get();
                (0..c.length)
                    .map(|i| {
                        let d = c.start + Duration::days(i as i64);
                        let editable = c.is_editable(d, today);
                        let mut cls = String::from("day ");
                        cls.push_str(
                            if c.is_done(d) {
                                "done pressed-sm"
                            } else if editable {
                                "raised-sm"
                            } else {
                                "raised-flat"
                            },
                        );
                        if d == today {
                            cls.push_str(" today");
                        }
                        view! {
                            <button
                                class=cls
                                disabled=!(editable && c.grid_tappable())
                                on:click=move |_| on_toggle.run(d)
                            />
                        }
                    })
                    .collect_view()
            }}
        </div>
    }
}

#[component]
pub fn CalendarSheet(
    cur: Signal<Challenge>,
    today: NaiveDate,
    on_toggle: Callback<NaiveDate>,
    on_close: Callback<()>,
) -> impl IntoView {
    let first_of_month = NaiveDate::from_ymd_opt(today.year(), today.month(), 1).unwrap();
    let (month, set_month) = signal(first_of_month);
    let shift = move |forward: bool| {
        set_month
            .update(|m| {
                *m = if forward {
                    m.checked_add_months(Months::new(1))
                } else {
                    m.checked_sub_months(Months::new(1))
                }
                .unwrap_or(*m)
            });
    };

    view! {
        <Sheet title="Calendar" on_close=on_close>
            <div class="cal-head">
                <button class="icon-btn raised-sm" aria-label="Previous month" on:click=move |_| shift(false)>
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                        <path d="M15 5l-7 7 7 7" />
                    </svg>
                </button>
                <span class="cal-month">{move || month.get().format("%B %Y").to_string()}</span>
                <button class="icon-btn raised-sm" aria-label="Next month" on:click=move |_| shift(true)>
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                        <path d="M9 5l7 7-7 7" />
                    </svg>
                </button>
            </div>
            <div class="cal-grid">
                {["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"]
                    .map(|d| view! { <div class="cal-dow">{d}</div> })
                    .collect_view()}
                {move || {
                    let m = month.get();
                    let c = cur.get();
                    let lead = m.weekday().num_days_from_monday() as i64;
                    let days = m
                        .checked_add_months(Months::new(1))
                        .map(|n| (n - m).num_days())
                        .unwrap_or(30);
                    (0..lead)
                        .map(|_| view! { <div class="cal-cell cal-blank" /> }.into_any())
                        .chain(
                            (0..days)
                                .map(|i| {
                                    let d = m + Duration::days(i);
                                    let editable = c.is_editable(d, today);
                                    let mut cls = String::from("cal-cell ");
                                    // Заливка выигрывает у диапазона: день из done виден даже
                                    // выпав из окна после Start over. Тапабельность — отдельно.
                                    cls.push_str(
                                        if c.is_done(d) {
                                            "done pressed-sm"
                                        } else if editable {
                                            "raised-sm"
                                        } else {
                                            "off raised-flat"
                                        },
                                    );
                                    if d == today {
                                        cls.push_str(" today");
                                    }
                                    view! {
                                        <button
                                            class=cls
                                            disabled=!editable
                                            on:click=move |_| on_toggle.run(d)
                                        >
                                            {d.day()}
                                        </button>
                                    }
                                        .into_any()
                                }),
                        )
                        .collect_view()
                }}
            </div>
        </Sheet>
    }
}

#[component]
pub fn FinishSheet(
    cur: Signal<Challenge>,
    on_start_over: Callback<()>,
    on_close: Callback<()>,
) -> impl IntoView {
    view! {
        <Sheet title="Challenge complete" on_close=on_close>
            <p class="sub">
                {move || {
                    let c = cur.get();
                    format!("{} of {} · best streak {}", c.done_count(), c.length, c.best_streak())
                }}
            </p>
            <button class="btn raised" on:click=move |_| on_start_over.run(())>
                "Start over"
            </button>
        </Sheet>
    }
}

/// Настройки и онбординг — один компонент. При `first_run` закрыть нельзя, экспорт и сброс
/// скрыты (сбрасывать и экспортировать нечего), а импорт остаётся: это сценарий
/// «поставил приложение на новый телефон и восстанавливаюсь из бэкапа».
#[component]
pub fn SettingsSheet(
    cur: Signal<Challenge>,
    today: NaiveDate,
    #[prop(default = false)] first_run: bool,
    set_challenge: WriteSignal<Option<Challenge>>,
    on_close: Callback<()>,
) -> impl IntoView {
    let c0 = cur.get_untracked();
    let (name, set_name) = signal(if first_run { String::new() } else { c0.name.clone() });
    let (length, set_length) = signal(c0.length);
    let (start, set_start) = signal(c0.start.to_string());
    let (paste, set_paste) = signal(String::new());
    let (bad_json, set_bad_json) = signal(false);
    let (confirm, set_confirm) = signal(Confirm::None);

    let commit = move |_| {
        let mut c = cur.get_untracked();
        c.name = name.get_untracked();
        c.length = length.get_untracked();
        c.start = NaiveDate::parse_from_str(&start.get_untracked(), "%Y-%m-%d").unwrap_or(c.start);
        set_challenge.set(Some(c.sanitize(today)));
        on_close.run(());
    };

    let parsed = move || serde_json::from_str::<Challenge>(&paste.get_untracked()).ok();

    let apply_import = move || match parsed() {
        Some(c) => {
            set_challenge.set(Some(c.sanitize(today)));
            on_close.run(());
        }
        None => set_bad_json.set(true),
    };

    let import_click = move |_| {
        set_bad_json.set(false);
        match parsed() {
            // Импорт затирает челлендж так же безвозвратно, как сброс, поэтому спрашивает так же.
            // Но спрашивает только про валидные данные — на мусор сразу отвечаем ошибкой.
            Some(_) if !first_run => set_confirm.set(Confirm::Overwrite),
            Some(_) => apply_import(),
            None => set_bad_json.set(true),
        }
    };

    let reset = move |_| {
        state::clear();
        set_challenge.set(None);
        on_close.run(());
    };

    let main_button = move || {
        view! {
            <button class="btn raised" on:click=commit>
                {if first_run { "Start" } else { "Save" }}
            </button>
        }
    };

    view! {
        <Sheet
            title=if first_run { "New challenge" } else { "Settings" }
            closable=!first_run
            on_close=on_close
        >
            <div class="field">
                <span class="field-label">"Name"</span>
                <input
                    class="input pressed-sm"
                    type="text"
                    placeholder="Challenge name"
                    prop:value=move || name.get()
                    on:input=move |ev| set_name.set(event_target_value(&ev))
                />
            </div>

            <div class="field">
                <span class="field-label">"Length"</span>
                <div class="pills">
                    {[30u32, 60, 100]
                        .map(|n| {
                            view! {
                                <button
                                    class=move || {
                                        if length.get() == n { "pill on pressed-sm" } else { "pill raised-sm" }
                                    }
                                    on:click=move |_| set_length.set(n)
                                >
                                    {n}
                                </button>
                            }
                        })
                        .collect_view()}
                    <input
                        class="input pill-custom pressed-sm"
                        type="number"
                        min=MIN_LENGTH
                        max=MAX_LENGTH
                        placeholder="Custom"
                        prop:value=move || length.get().to_string()
                        on:input=move |ev| {
                            if let Ok(n) = event_target_value(&ev).parse::<u32>() {
                                set_length.set(n);
                            }
                        }
                    />
                </div>
            </div>

            <div class="field">
                <span class="field-label">"Start date"</span>
                // max: сдвинуть старт назад нужно (челлендж уже идёт), вперёд — нет.
                // Это и убирает состояние «Day 0 of 30» по построению.
                <input
                    class="input pressed-sm"
                    type="date"
                    max=today.to_string()
                    prop:value=move || start.get()
                    on:input=move |ev| set_start.set(event_target_value(&ev))
                />
            </div>

            // В онбординге главная кнопка стоит ПОСЛЕ импорта: главное действие внизу,
            // в зоне большого пальца. В настройках Save стоит сразу под полями, к которым
            // относится, и подальше от Reset everything.
            {(!first_run).then(main_button)}

            {(!first_run)
                .then(|| {
                    view! {
                        // ponytail: data: URL вместо Blob + createObjectURL — тот же результат в одну строку
                        <a
                            class="btn raised"
                            download="challenge.json"
                            href=move || {
                                let json = serde_json::to_string(&cur.get()).unwrap_or_default();
                                format!(
                                    "data:application/json;charset=utf-8,{}",
                                    String::from(js_sys::encode_uri_component(&json)),
                                )
                            }
                        >
                            "Export JSON"
                        </a>
                    }
                })}

            <div class="field">
                <span class="field-label">"Import JSON"</span>
                // ponytail: textarea вместо FileReader и его async-обвязки
                <textarea
                    class="input pressed-sm"
                    placeholder="Paste JSON here"
                    prop:value=move || paste.get()
                    on:input=move |ev| set_paste.set(event_target_value(&ev))
                />
                <Show when=move || bad_json.get()>
                    <span class="hint">"Invalid JSON"</span>
                </Show>
                {move || match confirm.get() {
                    Confirm::Overwrite => {
                        view! {
                            <div class="confirm">
                                <span class="confirm-text">"Overwrite?"</span>
                                <button class="btn-slim yes pressed-sm" on:click=move |_| apply_import()>
                                    "Yes"
                                </button>
                                <button
                                    class="btn-slim raised-sm"
                                    on:click=move |_| set_confirm.set(Confirm::None)
                                >
                                    "No"
                                </button>
                            </div>
                        }
                            .into_any()
                    }
                    _ => {
                        view! {
                            <button class="btn raised" on:click=import_click>
                                "Import"
                            </button>
                        }
                            .into_any()
                    }
                }}
            </div>

            {first_run.then(main_button)}

            {(!first_run)
                .then(|| {
                    view! {
                        {move || match confirm.get() {
                            // Подтверждение внутри той же панели: ни window.confirm,
                            // ни модалки над модалкой — один backdrop, один z-index.
                            Confirm::Reset => {
                                view! {
                                    <div class="confirm">
                                        <span class="confirm-text">"Sure?"</span>
                                        <button class="btn-slim yes pressed-sm" on:click=reset>
                                            "Yes"
                                        </button>
                                        <button
                                            class="btn-slim raised-sm"
                                            on:click=move |_| set_confirm.set(Confirm::None)
                                        >
                                            "No"
                                        </button>
                                    </div>
                                }
                                    .into_any()
                            }
                            _ => {
                                view! {
                                    <button
                                        class="btn raised"
                                        on:click=move |_| set_confirm.set(Confirm::Reset)
                                    >
                                        "Reset everything"
                                    </button>
                                }
                                    .into_any()
                            }
                        }}
                    }
                })}
        </Sheet>
    }
}
