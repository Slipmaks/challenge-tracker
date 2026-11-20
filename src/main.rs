mod state;
mod ui;

use chrono::{Local, NaiveDate};
use leptos::{ev, html, prelude::*, wasm_bindgen::JsCast};
use state::{Challenge, load, save};
use std::time::Duration;
use ui::{CalendarSheet, DayGrid, FinishSheet, Ring, SettingsSheet, StartOver};

#[derive(Clone, Copy, PartialEq)]
enum Sheet {
    None,
    Settings,
    Calendar,
}

fn main() {
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    // Реактивное «сегодня»: обновляется при возврате во вкладку, фокусе и раз в минуту
    let (today, set_today) = signal(Local::now().date_naive());
    let refresh_today = move || {
        let now = Local::now().date_naive();
        if now != today.get_untracked() {
            set_today.set(now);
        }
    };
    let vis_handle = window_event_listener(ev::visibilitychange, move |_| refresh_today());
    let focus_handle = window_event_listener(ev::focus, move |_| refresh_today());
    let pageshow_handle = window_event_listener(ev::pageshow, move |_| refresh_today());
    let tick_handle = set_interval_with_handle(move || refresh_today(), Duration::from_secs(60));
    on_cleanup(move || {
        vis_handle.remove();
        focus_handle.remove();
        pageshow_handle.remove();
        if let Ok(h) = tick_handle {
            h.clear();
        }
    });

    // Option, а не Challenge: None — это «челленджа ещё нет», и именно оно включает онбординг.
    let (challenge, set_challenge) = signal(load());
    let (sheet, set_sheet) = signal(Sheet::None);
    let (sheet_closing, set_sheet_closing) = signal(false);
    let (finish_closing, set_finish_closing) = signal(false);
    let (onboarding_closing, set_onboarding_closing) = signal(false);
    // Не bool, а какой именно финиш уже показали. Не сохраняется: is_complete — производное.
    let (finish_seen, set_finish_seen) = signal(None::<(NaiveDate, u32)>);
    let (last_toggled, set_last_toggled) = signal(None::<NaiveDate>);

    Effect::new(move |_| {
        // Только Some: иначе первый же рендер сохранил бы пустышку и онбординг больше
        // никогда бы не появился
        if let Some(c) = challenge.get() {
            save(&c);
        }
    });

    // За обязательным онбордингом виден живой предпросмотр главного экрана (имя + длина).
    let (draft_len, set_draft_len) = signal(30u32);
    let (draft_name, set_draft_name) = signal(String::new());
    let cur = Signal::derive(move || {
        let t = today.get();
        challenge
            .get()
            .unwrap_or_else(|| Challenge::new(draft_name.get(), t, draft_len.get(), t))
    });

    // Челлендж, чей итог уже посмотрели: другой старт или другая длина — другой финиш
    let finish_id = move || {
        let c = cur.get();
        (c.start, c.length)
    };

    let toggle = Callback::new(move |d: NaiveDate| {
        let t = today.get_untracked();
        set_last_toggled.set(Some(d));
        set_challenge.update(|c| {
            if let Some(c) = c {
                c.toggle(d, t);
            }
        });
    });

    let settings_btn = NodeRef::<html::Button>::new();
    let calendar_btn = NodeRef::<html::Button>::new();
    let footer_ref = NodeRef::<html::Div>::new();

    let focus_footer_cta = move || {
        request_animation_frame(move || {
            if let Some(el) = footer_ref.get_untracked()
                && let Ok(Some(btn)) = el.query_selector("button:not(:disabled)")
                && let Ok(html_btn) = btn.dyn_into::<web_sys::HtmlElement>()
            {
                let _ = html_btn.focus();
            }
        });
    };

    let marked = move || cur.get().is_done(today.get());
    let markable = move || {
        let t = today.get();
        challenge.get().is_some() && cur.get().is_editable(t, t)
    };
    let finished = move || challenge.get().is_some() && cur.get().is_finished(today.get());

    // is_complete, а не is_finished: финиш приходит на тапе, который отметил последний день.
    let finish_open = move || {
        challenge.get().is_some()
            && cur.get().is_complete(today.get())
            && finish_seen.get() != Some(finish_id())
    };

    // Фокус возвращается на кнопку, открывшую шит, после 190ms анимации закрытия.
    let close = Callback::new(move |_: ()| {
        if sheet_closing.get_untracked() {
            return;
        }
        let back = match sheet.get_untracked() {
            Sheet::Settings => settings_btn,
            Sheet::Calendar => calendar_btn,
            Sheet::None => return,
        };
        set_sheet_closing.set(true);
        let _ = set_timeout_with_handle(
            move || {
                set_sheet.set(Sheet::None);
                set_sheet_closing.set(false);
                if !finish_open() {
                    request_animation_frame(move || {
                        if let Some(b) = back.get_untracked() {
                            let _ = b.focus();
                        }
                    });
                }
            },
            Duration::from_millis(190),
        );
    });

    // При полном сбросе онбординг сразу заменяет настройки без двойного мигания backdrop.
    let close_instant = Callback::new(move |_: ()| {
        set_sheet.set(Sheet::None);
        set_sheet_closing.set(false);
    });

    let noop = Callback::new(|_: ()| ());

    let dismiss_finish = Callback::new(move |_: ()| {
        if finish_closing.get_untracked() {
            return;
        }
        set_finish_closing.set(true);
        let id = finish_id();
        let _ = set_timeout_with_handle(
            move || {
                set_finish_seen.set(Some(id));
                set_finish_closing.set(false);
                focus_footer_cta();
            },
            Duration::from_millis(190),
        );
    });

    let start_over_inline = Callback::new(move |_: ()| {
        let t = today.get_untracked();
        set_challenge.update(|c| {
            if let Some(c) = c {
                c.start = t;
            }
        });
        focus_footer_cta();
    });

    let start_over_from_finish = Callback::new(move |_: ()| {
        if finish_closing.get_untracked() {
            return;
        }
        set_finish_closing.set(true);
        let _ = set_timeout_with_handle(
            move || {
                let t = today.get_untracked();
                set_challenge.update(|c| {
                    if let Some(c) = c {
                        c.start = t;
                    }
                });
                set_finish_closing.set(false);
                focus_footer_cta();
            },
            Duration::from_millis(190),
        );
    });

    let complete_onboarding = Callback::new(move |c: Challenge| {
        if onboarding_closing.get_untracked() {
            return;
        }
        set_onboarding_closing.set(true);
        let _ = set_timeout_with_handle(
            move || {
                set_challenge.set(Some(c));
                set_onboarding_closing.set(false);
                set_draft_name.set(String::new());
                set_draft_len.set(30);
                focus_footer_cta();
            },
            Duration::from_millis(190),
        );
    });

    // Под открытым шитом главный экран inert и не прокручивается.
    let sheet_open = move || challenge.get().is_none() || finish_open() || sheet.get() != Sheet::None;
    Effect::new(move |_| {
        let open = sheet_open();
        if let Some(body) = document().body() {
            let _ = body.class_list().toggle_with_force("sheet-open", open);
        }
    });

    view! {
        <div class="app" inert=sheet_open>
            <h1 class="title">{move || cur.get().name}</h1>

            <Ring cur today />

            <p class="sub">
                {move || {
                    let c = cur.get();
                    let t = today.get();
                    // Нулевая лучшая серия не показывается — как и нулевая текущая в кольце
                    let best = match c.best_streak() {
                        0 => String::new(),
                        b => format!(" · best streak {b}"),
                    };
                    if c.is_complete(t) {
                        format!("Finished{best}")
                    } else {
                        format!("Day {} of {}{best}", c.day_number(t), c.length)
                    }
                }}
            </p>

            <DayGrid
                cur
                today
                last_toggled
                on_toggle=toggle
                on_open=Callback::new(move |_| {
                    if !sheet_closing.get_untracked() {
                        set_sheet.set(Sheet::Calendar);
                    }
                })
            />

            <div class="footer" node_ref=footer_ref>
                <Show
                    when=finished
                    fallback=move || {
                        view! {
                            <button
                                class=move || { if marked() { "btn marked raised" } else { "btn raised" } }
                                disabled=move || !markable()
                                aria-pressed=move || { if marked() { "true" } else { "false" } }
                                on:click=move |_| toggle.run(today.get_untracked())
                            >
                                {move || {
                                    if marked() {
                                        view! {
                                            <svg
                                                class="tick-svg"
                                                viewBox="0 0 24 24"
                                                fill="none"
                                                stroke="currentColor"
                                                stroke-width="2.5"
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                                aria-hidden="true"
                                            >
                                                <path d="M5 12.5l4.5 4.5L19 7.5" />
                                            </svg>
                                            <span>"Marked"</span>
                                        }
                                            .into_any()
                                    } else {
                                        view! {
                                            <svg
                                                class="cta-icon"
                                                viewBox="0 0 24 24"
                                                fill="none"
                                                stroke="currentColor"
                                                stroke-width="2"
                                                stroke-linecap="round"
                                                stroke-linejoin="round"
                                                aria-hidden="true"
                                            >
                                                <circle cx="12" cy="12" r="9" />
                                                <path d="M8.5 12.5l2.5 2.5 4.5-5" />
                                            </svg>
                                            <span>"Mark day"</span>
                                        }
                                            .into_any()
                                    }
                                }}
                            </button>
                        }
                    }
                >
                    <StartOver on_start_over=start_over_inline />
                </Show>
                <div class="icons">
                    <button
                        class="icon-btn raised-sm"
                        node_ref=settings_btn
                        aria-label="Settings"
                        title="Settings"
                        on:click=move |_| {
                            if !sheet_closing.get_untracked() {
                                set_sheet.set(Sheet::Settings);
                            }
                        }
                    >
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
                            <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
                            <circle cx="12" cy="12" r="3" />
                        </svg>
                    </button>
                    <button
                        class="icon-btn raised-sm"
                        node_ref=calendar_btn
                        aria-label="Calendar"
                        title="Calendar"
                        on:click=move |_| {
                            if !sheet_closing.get_untracked() {
                                set_sheet.set(Sheet::Calendar);
                            }
                        }
                    >
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
                            <rect x="3" y="5" width="18" height="16" rx="3" />
                            <path d="M3 10h18M8 3v4M16 3v4" />
                        </svg>
                    </button>
                </div>
            </div>
        </div>

        // Онбординг — отдельный слот, чтобы сброс гарантированно размонтировал форму настроек.
        <Show when=move || challenge.get().is_none()>
            <SettingsSheet
                cur
                today
                closing=onboarding_closing
                first_run=true
                set_challenge
                on_close=noop
                on_length=Callback::new(move |n| set_draft_len.set(n))
                on_name=Callback::new(move |n| set_draft_name.set(n))
                on_first_commit=complete_onboarding
            />
        </Show>

        {move || {
            if challenge.get().is_none() {
                return None;
            }
            match sheet.get() {
                Sheet::Settings => Some(
                    view! {
                        <SettingsSheet
                            cur
                            today
                            closing=sheet_closing
                            set_challenge
                            on_close=close
                            on_reset_instant=close_instant
                        />
                    }
                    .into_any(),
                ),
                Sheet::Calendar => Some(
                    view! {
                        <CalendarSheet
                            cur
                            today
                            last_toggled
                            closing=sheet_closing
                            on_toggle=toggle
                            on_close=close
                        />
                    }
                    .into_any(),
                ),
                Sheet::None => {
                    if finish_open() {
                        Some(
                            view! {
                                <FinishSheet
                                    cur
                                    closing=finish_closing
                                    on_start_over=start_over_from_finish
                                    on_close=dismiss_finish
                                />
                            }
                            .into_any(),
                        )
                    } else {
                        None
                    }
                }
            }
        }}
    }
}
