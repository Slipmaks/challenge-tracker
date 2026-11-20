//! Компоненты. Вся композиция, состояния и строки — design.md.

use crate::state::{self, Challenge, MAX_LENGTH, MIN_LENGTH};
use chrono::{Datelike, Duration, Months, NaiveDate};
use leptos::{html, prelude::*};

#[derive(Clone, Copy, PartialEq)]
enum Confirm {
    None,
    Reset,
    Overwrite,
    Apply,
}

/// Общая обёртка bottom sheet.
///
/// Тап по затемнению закрывает, тап внутри панели — нет: клик внутри останавливается на самой
/// панели и до затемнения не доходит. Иначе нативный `<input type="date">` закрывал бы шит
/// прямо во время выбора даты.
///
/// Для скринридера и клавиатуры это диалог: фокус при открытии уходит в панель, Esc закрывает
/// так же, как X. Главный экран под затемнением в это время `inert` (main.rs). id заголовка
/// один на всех: открыт всегда ровно один шит.
#[component]
fn Sheet(
    #[prop(into)] title: String,
    #[prop(default = true)] closable: bool,
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
            class="backdrop"
            on:click=move |_| if closable { on_close.run(()) }
            on:keydown=move |ev| if closable && ev.key() == "Escape" { on_close.run(()) }
        >
            <div
                class="sheet"
                node_ref=panel
                role="dialog"
                aria-modal="true"
                aria-labelledby="sheet-title"
                tabindex="-1"
                on:click=|ev| ev.stop_propagation()
            >
                <div class="sheet-head">
                    <h2 class="sheet-title" id="sheet-title">{title}</h2>
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

/// Инлайн-подтверждение: кнопка на своём месте становится плашкой «вопрос · да · нет».
/// Кнопка, которой задали вопрос, исчезла из DOM вместе с фокусом — фокус переезжает на
/// безопасный ответ. Это же прокручивает плашку в видимую часть шита на коротком экране.
/// `big` — плашка на месте главной кнопки 56px, а не второстепенной 48px.
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
            // role=alert: фокус стоит на «No», и без этого скринридер не прочитал бы сам вопрос
            <span class="confirm-text" role="alert">{move || text.get()}</span>
            // Тинт — у безопасного ответа, а не у «да»: тинт значит «выбрано», и выбран тут
            // ответ по умолчанию — тот, на котором фокус. Цвет и фокус говорят одно и то же.
            <button class="btn-slim raised-sm" on:click=move |_| on_yes.run(())>{yes}</button>
            <button class="btn-slim safe raised-sm" node_ref=safe on:click=move |_| on_no.run(())>{no}</button>
        </div>
    }
}

/// Ошибка или подсказка под полем. Своя форма — иконка и текст `--fg` 14/500, — а не `.hint`
/// цвета подписи поля: та же 13/400 `--fg-muted` читалась подписью к следующему полю.
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

/// `Start over` с вопросом — в шите финиша и в футере после его закрытия. Без вопроса один тап
/// после 100 дней превращал итог в `0/30 · Day 1 of 30`.
#[component]
pub fn StartOver(on_start_over: Callback<()>) -> impl IntoView {
    let (asking, set_asking) = signal(false);
    view! {
        <Show
            when=move || asking.get()
            fallback=move || {
                view! {
                    <button class="btn raised" on:click=move |_| set_asking.set(true)>
                        "Start over"
                    </button>
                }
            }
        >
            <ConfirmRow
                text="Start over from today? History stays."
                big=true
                on_yes=on_start_over
                on_no=Callback::new(move |_| set_asking.set(false))
            />
        </Show>
    }
}

/// Вид клетки дня — одно правило для сетки и календаря. Четыре состояния различаются формой,
/// а не только тенью: галочка — выполнено, точка — пропущено, плоская обводка — нельзя менять
/// (будущее или вне окна), чистый рельеф — сегодня, ещё не отмечено.
///
/// Выполненный день вне окна (прошлый цикл после `Start over`) — тинт без рельефа: рельеф
/// значит «можно нажать», а этот день не переключается.
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

/// Имя клетки для скринридера: у клетки сетки нет текста вовсе, у календарной — одно число.
/// «Выполнено» говорит aria-pressed, поэтому в имени только дата и то, чего pressed не скажет.
fn day_label(d: NaiveDate, today: NaiveDate) -> String {
    let mut s = d.format("%B %-d").to_string();
    if d == today {
        s.push_str(", today");
    } else if d > today {
        s.push_str(", upcoming");
    }
    s
}

/// aria-pressed у дня, который был или может быть отмечен; у будущего его нет вовсе.
fn day_pressed(c: &Challenge, d: NaiveDate, today: NaiveDate) -> Option<&'static str> {
    let done = c.is_done(d);
    (done || c.is_editable(d, today)).then_some(if done { "true" } else { "false" })
}

#[component]
pub fn Ring(cur: Signal<Challenge>, today: NaiveDate) -> impl IntoView {
    view! {
        <div class="ring-well raised">
            // --p — один из ровно двух инлайн-стилей в проекте (второй --cols в DayGrid)
            <div class="ring" style=move || format!("--p:{}", cur.get().percent())>
                <div class="ring-face">
                    <div class="ring-count">
                        {move || {
                            let c = cur.get();
                            format!("{}/{}", c.done_count(), c.length)
                        }}
                    </div>
                    // Нулевая серия не показывается: на первом дне экран был из трёх нулей
                    {move || {
                        let s = cur.get().current_streak(today);
                        (s > 0).then(|| view! { <div class="ring-streak">{format!("streak {s}")}</div> })
                    }}
                </div>
            </div>
        </div>
    }
}

/// Две сетки по `grid_tappable`. До 35 дней — клетки-кнопки. Дальше клетка меньше пальца, и
/// сетка становится обзором: плоские клетки внутри одной выпуклой панели, а тап по панели
/// открывает календарь, где правится любой день. Выпуклая панель — потому что рельеф значит
/// «можно нажать», и нажать здесь можно ровно её.
#[component]
pub fn DayGrid(
    cur: Signal<Challenge>,
    today: NaiveDate,
    on_toggle: Callback<NaiveDate>,
    on_open: Callback<()>,
) -> impl IntoView {
    move || {
        let c = cur.get();
        let style = format!("--cols:{}", c.cols());
        let days = (0..c.length).map(|i| c.start + Duration::days(i as i64));
        if c.grid_tappable() {
            let cells = days
                .map(|d| {
                    view! {
                        <button
                            class=format!("day {}", day_class(&c, d, today))
                            aria-label=day_label(d, today)
                            aria-pressed=day_pressed(&c, d, today)
                            disabled=!c.is_editable(d, today)
                            on:click=move |_| on_toggle.run(d)
                        />
                    }
                })
                .collect_view();
            view! {
                <div class="grid" style=style role="group" aria-label="Days">
                    {cells}
                </div>
            }
            .into_any()
        } else {
            // dense — с 12 колонок: зазор 4 и точка пропуска 4px
            let cls = if c.cols() >= 12 { "grid overview dense" } else { "grid overview" };
            let cells = days
                .map(|d| view! { <span class=format!("day {}", day_class(&c, d, today))></span> })
                .collect_view();
            view! {
                <button class="overview-panel raised" aria-label="Open calendar" on:click=move |_| on_open.run(())>
                    <span class=cls style=style>
                        {cells}
                    </span>
                </button>
            }
            .into_any()
        }
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
                <span class="cal-month" aria-live="polite">
                    {move || month.get().format("%B %Y").to_string()}
                </span>
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
                                    // Заливка выигрывает у диапазона (day_class проверяет done
                                    // первым): день из done виден даже выпав из окна после
                                    // Start over. Тапабельность — отдельно, is_editable.
                                    view! {
                                        <button
                                            class=format!("cal-cell {}", day_class(&c, d, today))
                                            aria-label=day_label(d, today)
                                            aria-pressed=day_pressed(&c, d, today)
                                            disabled=!c.is_editable(d, today)
                                            on:click=move |_| on_toggle.run(d)
                                        >
                                            {d.day()}
                                        </button>
                                    }
                                        .into_any()
                                }),
                        )
                        // Всегда 6 недель: иначе шит на 5-недельном месяце ниже на ряд, и стрелка
                        // месяца уезжает из-под пальца при листании
                        .chain((lead + days..42).map(|_| view! { <div class="cal-cell cal-blank" /> }.into_any()))
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
            <div class="summary">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" aria-hidden="true">
                    <path d="M6 9H4.5a2.5 2.5 0 0 1 0-5H6" />
                    <path d="M18 9h1.5a2.5 2.5 0 0 0 0-5H18" />
                    <path d="M4 22h16" />
                    <path d="M10 14.7V17c0 .6-.5 1-1 1.2C7.9 18.8 7 20.2 7 22" />
                    <path d="M14 14.7V17c0 .6.5 1 1 1.2 1.1.6 2 2 2 3.8" />
                    <path d="M18 2H6v7a6 6 0 0 0 12 0V2Z" />
                </svg>
                <div class="summary-score">
                    {move || {
                        let c = cur.get();
                        format!("{} of {}", c.done_count(), c.length)
                    }}
                </div>
                <div class="summary-streak">
                    {move || format!("best streak {}", cur.get().best_streak())}
                </div>
            </div>
            <StartOver on_start_over />
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
    /// Онбординг: длина из формы, чтобы кольцо и сетка за шитом показывали выбранную, а не 30
    #[prop(optional)]
    on_length: Option<Callback<u32>>,
) -> impl IntoView {
    let c0 = cur.get_untracked();
    let (name, set_name) = signal(if first_run { String::new() } else { c0.name.clone() });
    // Длина — ровно то, что видно в форме: выбранная пилюля или число в Custom
    // (state::form_length). None — не выбрано ничего или число вне 1–365: Start и Done тогда
    // недоступны, а не сохраняют невидимое значение и не зажимают молча.
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
    let (start, set_start) = signal(c0.start.to_string());
    let (paste, set_paste) = signal(String::new());
    let (bad_json, set_bad_json) = signal(None::<&'static str>);
    let (confirm, set_confirm) = signal(Confirm::None);
    let (import_open, set_import_open) = signal(false);

    // Челлендж, каким он станет после сохранения: поля формы поверх текущего
    let edited = move || {
        let mut c = cur.get_untracked();
        c.name = name.get_untracked();
        c.length = length.get_untracked().unwrap_or(c.length);
        c.start = NaiveDate::parse_from_str(&start.get_untracked(), "%Y-%m-%d").unwrap_or(c.start);
        c.sanitize(today)
    };

    // Настройки правят существующий челлендж и только его. Если челленджа нет — снесли через
    // Reset everything — закрытие шита не должно воскрешать его из полей формы: старый старт
    // плюс короткая длина дают мгновенный Finished с пустым done.
    let commit = move |c: Challenge| {
        set_challenge.update(|slot| {
            if slot.is_some() {
                *slot = Some(c);
            }
        });
    };

    // Создание — отдельное действие онбординга, а не тот же commit: только здесь появление
    // челленджа из ничего законно. Challenge::new сам зовёт sanitize и даёт пустой done.
    let create = move || {
        let s = NaiveDate::parse_from_str(&start.get_untracked(), "%Y-%m-%d").unwrap_or(today);
        set_challenge.set(Some(Challenge::new(
            name.get_untracked(),
            s,
            length.get_untracked().unwrap_or(c0.length),
            today,
        )));
    };

    // Сохраняет только Done. X, тап по затемнению и Esc — отмена, как везде: раньше закрытие и
    // было сохранением, и Esc, который на любой платформе значит «отмена», молча записывал
    // правку. Правка, которая отнимает прогресс (edit_warning), и на Done не проходит молча:
    // кнопка на своём месте становится вопросом Apply / Keep.
    // Импорт и сброс сохраняют сами и закрывают шит сырым on_close, мимо Done: иначе поверх
    // свежих данных легли бы старые значения полей формы.
    let done = Callback::new(move |_| {
        let c = edited();
        if cur.get_untracked().edit_warning(&c, today).is_some() {
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
    // Keep откатывает только опасное — длину и старт; новое имя сохраняется
    let keep = Callback::new(move |_| {
        let mut c = cur.get_untracked();
        c.name = name.get_untracked();
        commit(c.sanitize(today));
        on_close.run(());
    });
    let cancel = Callback::new(move |_| set_confirm.set(Confirm::None));
    let warning = move || {
        length.track();
        start.track();
        (confirm.get() == Confirm::Apply)
            .then(|| cur.get_untracked().edit_warning(&edited(), today))
            .flatten()
    };

    let parsed = move || state::parse_backup(&paste.get_untracked());

    let apply_import = move || match parsed() {
        Ok(c) => {
            set_challenge.set(Some(c.sanitize(today)));
            on_close.run(());
        }
        Err(e) => set_bad_json.set(Some(e)),
    };

    // Вопросы называют, что именно пропадёт: «Overwrite?» не говорил, что с чем.
    // Счёт — весь done, а не done_count: стирается и история прошлых циклов.
    let marked = move || cur.get_untracked().done.len();
    let replace_text = move || match marked() {
        0 => format!("Replace “{}” with the backup?", cur.get_untracked().name),
        n => format!("Replace “{}” ({n} marked) with the backup?", cur.get_untracked().name),
    };
    let erase_text = move || match marked() {
        0 => format!("Erase “{}”?", cur.get_untracked().name),
        1 => format!("Erase “{}” and 1 marked day?", cur.get_untracked().name),
        n => format!("Erase “{}” and {n} marked days?", cur.get_untracked().name),
    };

    let import_click = move |_| {
        set_bad_json.set(None);
        match parsed() {
            // Импорт затирает челлендж так же безвозвратно, как сброс, поэтому спрашивает так же.
            // Но спрашивает только про валидные данные — на мусор сразу отвечаем ошибкой.
            Ok(_) if !first_run => set_confirm.set(Confirm::Overwrite),
            Ok(_) => apply_import(),
            Err(e) => set_bad_json.set(Some(e)),
        }
    };

    let reset = Callback::new(move |_| {
        state::clear();
        set_challenge.set(None);
        on_close.run(());
    });

    view! {
        <Sheet
            title=if first_run { "New challenge" } else { "Settings" }
            closable=!first_run
            on_close=on_close
        >
            // <label>, а не <div>: подпись над полем — она же его имя для скринридера. У длины
            // внутри кнопки, а кнопку в label класть нельзя, поэтому там group с тем же именем.
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
                                    aria-pressed=move || if pill.get() == Some(n) { "true" } else { "false" }
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
                    // Тинт только у числа, которое и станет длиной: 400 в поле не выбрано
                    <input
                        class=move || {
                            if !custom.get().trim().is_empty() && length.get().is_some() {
                                "input pill-custom on raised-sm"
                            } else {
                                "input pill-custom raised-sm"
                            }
                        }
                        type="number"
                        min=MIN_LENGTH
                        max=MAX_LENGTH
                        placeholder="Custom"
                        aria-label="Custom length"
                        inputmode="numeric"
                        // Своё поле, а не отражение длины: стёртое поле остаётся пустым, а не
                        // хранит невидимую последнюю цифру. Число в поле снимает выбор с пилюль —
                        // выбрано всегда что-то одно.
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
                    <Alert text="Choose 1–365 days" />
                </Show>
            </div>

            <label class="field">
                <span class="field-label">"Start date"</span>
                // max: сдвинуть старт назад нужно (челлендж уже идёт), вперёд — нет.
                // Это и убирает состояние «Day 0 of 30» по построению.
                <input
                    class="input raised-sm"
                    type="date"
                    max=today.to_string()
                    prop:value=move || start.get()
                    on:input=move |ev| set_start.set(event_target_value(&ev))
                />
            </label>

            // Группы шита разведены воздухом, а не линиями: внутри группы 20, между — 32 (.sep).
            // Челлендж | бэкап | сброс | Done. В онбординге групп меньше, и он остаётся макетным.
            {(!first_run)
                .then(|| {
                    view! {
                        // ponytail: data: URL вместо Blob + createObjectURL — тот же результат в одну строку
                        <a
                            class="btn-secondary raised-sm sep"
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

            // В макете импорт — одна плашка «Import JSON», без поля для вставки. Поле
            // раскрывается по тапу: в покое шит совпадает с макетом по высоте, а textarea
            // не занимает 130px впустую в сценарии, который случается раз в жизни телефона.
            <Show
                when=move || import_open.get()
                fallback=move || {
                    view! {
                        <button
                            class="btn-secondary raised-sm"
                            on:click=move |_| set_import_open.set(true)
                        >
                            "Import JSON"
                        </button>
                    }
                }
            >
                <div class="field">
                    <span class="field-label">"Import JSON"</span>
                    // ponytail: textarea вместо FileReader и его async-обвязки
                    // aria-label, а не label-обёртка: рядом кнопки Import/Yes/No, а им в label нельзя
                    <textarea
                        class="input raised-sm"
                        placeholder="Paste JSON here"
                        aria-label="Import JSON"
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
                                    on_no=cancel
                                />
                            }
                                .into_any()
                        }
                        _ => {
                            view! {
                                <button class="btn-secondary raised-sm" on:click=import_click>
                                    "Import"
                                </button>
                            }
                                .into_any()
                        }
                    }}
                </div>
            </Show>

            // Главное действие стоит внизу, в зоне большого пальца: Start в онбординге, Done в
            // настройках. Один и тот же ярус кнопки — шит говорит одним языком в обоих режимах.
            {first_run
                .then(|| {
                    view! {
                        <button
                            class="btn raised"
                            disabled=move || length.get().is_none()
                            on:click=move |_| {
                                create();
                                on_close.run(());
                            }
                        >
                            "Start"
                        </button>
                    }
                })}

            {(!first_run)
                .then(|| {
                    view! {
                        // Сброс — своя группа: прямо под Import он читался как третья операция
                        // бэкапа, а это единственное действие, которое стирает всё
                        <div class="sep">
                            {move || match confirm.get() {
                                // Подтверждение внутри той же панели: ни window.confirm,
                                // ни модалки над модалкой — один backdrop, один z-index.
                                Confirm::Reset => {
                                    view! {
                                        <ConfirmRow text=erase_text() on_yes=reset on_no=cancel />
                                    }
                                        .into_any()
                                }
                                _ => {
                                    view! {
                                        <button
                                            class="btn-secondary raised-sm"
                                            on:click=move |_| set_confirm.set(Confirm::Reset)
                                        >
                                            "Reset everything"
                                        </button>
                                    }
                                        .into_any()
                                }
                            }}
                        </div>
                        // Done на своём месте становится вопросом, если правка отнимает прогресс.
                        // Текст живой: поправил длину при открытом вопросе — вопрос пересчитался
                        // или вернулся Done. Show, а не if: плашка не пересоздаётся на каждую
                        // цифру в Custom и не уводит фокус из поля.
                        <div class="sep">
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
                        </div>
                    }
                })}
        </Sheet>
    }
}
