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
    // Не bool, а какой именно финиш уже показали. Флагом он не возвращался в false: после смены
    // длины (или после сброса и нового челленджа) шит финиша больше не появлялся, и Start over
    // приходил только с перезапуском приложения. Не сохраняется: is_finished — производное.
    let (finish_seen, set_finish_seen) = signal(None::<(NaiveDate, u32)>);

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

    // Челлендж, чей итог уже посмотрели: другой старт или другая длина — другой финиш
    let finish_id = move || {
        let c = cur.get();
        (c.start, c.length)
    };

    let toggle = Callback::new(move |d: NaiveDate| {
        set_challenge.update(|c| {
            if let Some(c) = c {
                c.toggle(d, today);
            }
        })
    });
    let close = Callback::new(move |_: ()| set_sheet.set(Sheet::None));
    let noop = Callback::new(|_: ()| ());
    let dismiss_finish = Callback::new(move |_: ()| set_finish_seen.set(Some(finish_id())));
    // Флаг тут больше не нужен: после сдвига старта челлендж не завершён, и шит уходит сам
    let start_over = Callback::new(move |_: ()| {
        set_challenge.update(|c| {
            if let Some(c) = c {
                c.start = today; // историю не трогаем, меняется только окно
            }
        })
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
                    class=move || { if marked() { "btn marked raised" } else { "btn raised" } }
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
                            // Шестерёнка макета — lucide settings. Своё «кольцо плюс восемь спиц»
                            // на 20px читалось солнцем: спицы есть, зубьев нет.
                            <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z" />
                            <circle cx="12" cy="12" r="3" />
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

        // Онбординг — отдельный слот, а не ветка выражения ниже: в одном слоте у обеих панелей
        // один тип view (SettingsSheet), и Leptos пересобирает панель на месте вместо того чтобы
        // размонтировать. Своим слотом настройки после сброса умирают вместе со своей формой.
        // Закрыть онбординг нельзя, поэтому on_close никогда не зовётся.
        <Show when=move || challenge.get().is_none()>
            <SettingsSheet cur today first_run=true set_challenge on_close=noop />
        </Show>

        {move || {
            if challenge.get().is_none() {
                return None;
            }
            if cur.get().is_finished(today) && finish_seen.get() != Some(finish_id()) {
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
