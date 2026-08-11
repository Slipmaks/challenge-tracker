mod state;
mod ui;

use chrono::{Local, NaiveDate};
use leptos::prelude::*;
use state::{Challenge, load, save};
use ui::{CalendarSheet, DayGrid, FinishSheet, Ring, SettingsSheet};

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
    // Единственное место в проекте, где спрашивается текущее время
    let today: NaiveDate = Local::now().date_naive();

    // Option, а не Challenge: None — это «челленджа ещё нет», и именно оно включает онбординг.
    let (challenge, set_challenge) = signal(load());
    let (sheet, set_sheet) = signal(Sheet::None);
    // Флаг «итог уже видел» не сохраняется: is_finished — производное, а производное не храним
    let (finish_seen, set_finish_seen) = signal(false);

    Effect::new(move |_| {
        // Только Some: иначе первый же рендер сохранил бы пустышку и онбординг больше
        // никогда бы не появился
        if let Some(c) = challenge.get() {
            save(&c);
        }
    });

    // За обязательным онбордингом виден главный экран с дефолтами: пустое серое полотно
    // смотрелось бы как незагрузившееся приложение
    let cur = Signal::derive(move || {
        challenge
            .get()
            .unwrap_or_else(|| Challenge::new(String::new(), today, 30, today))
    });

    let toggle = Callback::new(move |d: NaiveDate| {
        set_challenge.update(|c| {
            if let Some(c) = c {
                c.toggle(d, today);
            }
        })
    });
    let close = Callback::new(move |_: ()| set_sheet.set(Sheet::None));
    let noop = Callback::new(|_: ()| ());
    let dismiss_finish = Callback::new(move |_: ()| set_finish_seen.set(true));
    let start_over = Callback::new(move |_: ()| {
        set_challenge.update(|c| {
            if let Some(c) = c {
                c.start = today; // историю не трогаем, меняется только окно
            }
        });
        set_finish_seen.set(true);
    });

    let marked = move || cur.get().is_done(today);
    // Третий случай кнопки: today вне диапазона — челлендж завершён либо ещё не начался
    let markable = move || challenge.get().is_some() && cur.get().is_editable(today, today);

    view! {
        <div class="app">
            <h1 class="title">{move || cur.get().name}</h1>

            <Ring cur today />

            <p class="sub">
                {move || {
                    let c = cur.get();
                    if c.is_finished(today) {
                        format!("Finished · best streak {}", c.best_streak())
                    } else {
                        format!(
                            "Day {} of {} · best streak {}",
                            c.day_number(today),
                            c.length,
                            c.best_streak(),
                        )
                    }
                }}
            </p>

            <DayGrid cur today on_toggle=toggle />

            <div class="footer">
                <button
                    class=move || { if marked() { "btn marked pressed" } else { "btn raised" } }
                    disabled=move || !markable()
                    on:click=move |_| toggle.run(today)
                >
                    {move || if marked() { "Marked" } else { "Mark day" }}
                </button>
                <div class="icons">
                    <button
                        class="icon-btn raised-sm"
                        aria-label="Settings"
                        on:click=move |_| set_sheet.set(Sheet::Settings)
                    >
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                            // Шестерёнка: внешнее кольцо обязательно, без него восемь спиц
                            // читаются как солнце, а не как настройки
                            <circle cx="12" cy="12" r="6.3" />
                            <circle cx="12" cy="12" r="2.4" />
                            <path d="M12 2.2v3.6M12 18.2v3.6M2.2 12h3.6M18.2 12h3.6M5.2 5.2l2.6 2.6M16.2 16.2l2.6 2.6M18.8 5.2l-2.6 2.6M7.8 16.2l-2.6 2.6" />
                        </svg>
                    </button>
                    <button
                        class="icon-btn raised-sm"
                        aria-label="Calendar"
                        on:click=move |_| set_sheet.set(Sheet::Calendar)
                    >
                        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                            <rect x="3" y="5" width="18" height="16" rx="3" />
                            <path d="M3 10h18M8 3v4M16 3v4" />
                        </svg>
                    </button>
                </div>
            </div>
        </div>

        {move || {
            if challenge.get().is_none() {
                // Онбординг: закрыть нельзя, поэтому on_close никогда не зовётся
                return Some(
                    view! {
                        <SettingsSheet cur today first_run=true set_challenge on_close=noop />
                    }
                        .into_any(),
                );
            }
            if cur.get().is_finished(today) && !finish_seen.get() {
                return Some(
                    view! {
                        <FinishSheet cur on_start_over=start_over on_close=dismiss_finish />
                    }
                        .into_any(),
                );
            }
            match sheet.get() {
                Sheet::Settings => {
                    Some(view! { <SettingsSheet cur today set_challenge on_close=close /> }.into_any())
                }
                Sheet::Calendar => {
                    Some(
                        view! { <CalendarSheet cur today on_toggle=toggle on_close=close /> }
                            .into_any(),
                    )
                }
                Sheet::None => None,
            }
        }}
    }
}
