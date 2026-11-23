Action Summary

  

  План действий собран строго по твоим приоритетам (сначала паритет и компоненты → затем топ-3 UX-прорыва) и  

  согласованным правилам (монохром без красного с разводкой ошибок/No через $fg и геометрию контура + 6   

  колонок для ≤35 дней; десктопный 13 Main вне объёма этого прохода): 

  

  1. /impeccable audit — Баги паритета, данных и контрастных хвостов (Assessment B):

      • Пересобрать сетку PNsRy на 12 Main - 365 days (QVvBa) ровно под 84 выполненных дня, 27 пропусков и 

      best streak 21 (сейчас 92 и 12).

      • Исправить рамку .trophy-medallion в style.css:723 с rgba(54, 101, 236, 0.28) (1.41:1) на var(--cta-  

      border), поднять $cta-border (82% → 85%) и $border-overview (#788194 → #767f92), чтобы ≥3.0:1 держалось 

      и на углу $surface-lo.

      • Закрыть мелкие расхождения .pen ↔ код: y9Azbo Length Options gap: 6 → 8px, kaBCV выделить День 1 как  

      today и показать иконку на disabled Mark day, убрать плоские $bg-оверрайды с 5 инстансов Pill, выровнять

      стиль раскрытого Import и 1px-касание Length в uamwY, C0Uiqu flame (11 → 12px), подпись 64×44 в t12IK и

      устаревшие числа в [design.md](http://design.md).

  2. /impeccable harden — Разводка семантики ошибок и отмены внутри монохрома (P1 #1):

      • В challange-design.pen (zCs7W, uamwY, t12IK) и style.css перевести невалидные поля с синего $accent-

      ink на 2px пунктирную/чёткую обводку $fg (8.27:1), текст $fg и inline-индикатор ошибки.

      • Перевести кнопку Import (tUFFS в uamwY + src/[ui.rs](http://ui.rs)) в плоский disabled, пока JSON пуст или невалиден.

      • В ConfirmRow (beJbu, rkljQ + style.css) заменить синюю заливку выбора на No/Keep чётким графитовым  

      контуром $fg + 600, чтобы не путать отмену с выбранными пилюлями и CTA.

  3. /impeccable adapt — 6 колонок для ≤35 дней и безопасный 2-строчный ConfirmRow (P1 #2):

      • Перевести интерактивную сетку ≤35 дней с 7 на 6 колонок в challange-design.pen, src/[ui.rs](http://ui.rs), style.css и

      [design.md](http://design.md): 30 дней складываются в ровный 6×5 без 2-клеточного хвоста, уходит ложная иллюзия Пн–Вс, а на

      320×568 (C0Uiqu) клетки вырастают с 38.5px до ≥44px.

      • Перестроить ConfirmRow / ConfirmRow Big на мобильных в 2 строки: вопрос сверху + две равные кнопки

      высотой 44px с зазором 12px снизу (убирает риск промаха по Yes/No на удалении).

  4. /impeccable clarify — Границы челленджа и прошлый цикл в календаре + понятный копирайт (P1 #3):          

      • В 06 Calendar (eCZMw, khaYE, H2wXnv + style.css / src/[ui.rs](http://ui.rs)) оставить рамку 1px $border-off только для

      будущих дней внутри челленджа (Aug 11–23), а даты вне диапазона (Jul 1–24, Aug 24–31) рисовать без рамки

      ($fg-muted на $bg).

      • Для Past Cycle (Dl9lI, t12IK + style.css) добавить не-теневой маркер ($fg-muted цифра + пунктир

      $border-off без $selected-border), чтобы заблокированная история не держалась на тени 1.39:1.   

      • Заменить Export JSON / Import JSON на Export backup / Restore backup (включая 03 Onboarding) и 

      выровнять все кнопки бэкапа и сброса в Settings по единой оси с иконками.           

  5. /impeccable polish — Привязка 5 компонентов, финальный паритет, сборка и верификация:

      • Перепривязать оторванные фреймы в challange-design.pen к PUCnt, ufOZY, WpvbL, beJbu, rkljQ (убрать 8

      ошибочных ссылок Export/Import на Pill i6MH00), добавить $selected-border в Day Cell Check (ZpOkI),

      обновить День 30 в v3L0A и Component States (t12IK), сохранить .pen.

      • Прогнать cargo test, пересобрать docs/ (trunk build --release --dist docs --public-url /challenge-    

      tracker/) и проверить рендер в Firefox (320×568, 375×667, 390×844, 1280×800).

  

  

  Можешь попросить выполнить эти шаги по одному, все сразу или в любом удобном порядке.

  

  После правок запусти /impeccable critique ещё раз, чтобы проверить рост оценки (цель — выход из 22/36

  Acceptable в 28–30/36 Good).