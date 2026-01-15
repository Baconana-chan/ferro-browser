# Ferro Browser — TODO

**Форк Servo для создания лёгкого, минималистичного браузера**

## Цель проекта
Создать лёгкий, минималистичный браузер на базе Servo (Rust), который:
- Открывает современные Tailwind-сайты (типа lorachi.xyz) без сильных поплывов
- Поддерживает базовый поиск (DDG full mode, не lite)
- Имеет приемлемую интерактивность (caret, selection, smooth scroll)
- Блокирует рекламу "по умолчанию" (за счёт неполной совместимости, пока не пофиксим)
- Работает стабильно на Linux/macOS/Windows (через embedding в свой shell)

Текущий статус: Servo 0.0.3 — proof-of-concept engine, servoshell — минимальный демо. Много missing web platform features.

## Приоритет 1: Критическая usability (чтобы не бесило каждый день)
- [x] Visible caret + blinking в <input>/<textarea> (textinput.rs / layout)
  - ✅ Реализовано мигание каретки с интервалом 530ms
  - ✅ Каретка сбрасывается при вводе текста и смене позиции
  - ✅ Page visibility: мигание останавливается когда вкладка в background (экономия CPU)
  - ✅ Таймеры уникальны per-element (Option<i32> в каждом элементе)
  - 🔧 TODO: prefers-reduced-motion (статичная каретка для accessibility)
  - 🔧 TODO: Оптимизация repaint (только область каретки, не весь input)
  - 🔧 TODO: A11y интеграция (caret position в accessibility tree)
  - Изменения: htmlinputelement.rs, htmltextareaelement.rs, document.rs
- [x] Mouse text selection + visual highlight (components/script/dom/selection)
  - ✅ Выделение мышью: mousedown начинает, mousemove расширяет, mouseup завершает
  - ✅ Shift+click: расширение выделения от текущей позиции каретки
  - ✅ Double-click: выделение слова (Unicode word boundaries)
  - ✅ Triple-click: выделение всей строки
  - ✅ Каретка скрывается при активном выделении (не-collapsed selection)
  - ✅ Ctrl+C/X/V: копирование, вырезание, вставка (уже было реализовано в textinput.rs)
  - ✅ Добавлены методы select_word_at_index(), select_line_at_index(), selection_is_collapsed()
  - ✅ Layout поддерживает отрисовку selection_range (visual highlight)
  - 🔧 TODO: Auto-scroll при drag за пределы видимой области
  - 🔧 TODO: Цвет highlight в preferences
  - Изменения: textinput.rs, htmlinputelement.rs, htmltextareaelement.rs
- [x] Smooth scrolling (momentum/inertia/easing) для mouse wheel/trackpad
  - ✅ Создан модуль smooth_scroll.rs с анимацией и easing (ease-out cubic)
  - ✅ SmoothScrollAnimation: интерполяция позиции с длительностью 300ms
  - ✅ SmoothScrollState: накопление delta, расчёт frame_delta каждый кадр
  - ✅ Интеграция с WebViewRenderer: on_wheel_event → smooth_scroll_state
  - ✅ Автоматический repaint пока анимация активна
  - ✅ Плавное объединение последовательных wheel событий (momentum)
  - 🔧 TODO: Настройка длительности и easing через preferences
  - 🔧 TODO: Отключение smooth scroll для accessibility (reduced-motion)
  - 🔧 TODO: Trackpad-specific momentum (velocity-based deceleration)
  - Изменения: smooth_scroll.rs (новый), webview_renderer.rs, painter.rs, lib.rs
- [x] Clipboard copy-paste (keyboard shortcuts)
  - ✅ Ctrl+C/X/V работают в text inputs (textinput.rs handle_keydown)
  - ✅ Интеграция с OS clipboard через EmbedderClipboardProvider
  - 🔧 TODO: navigator.clipboard JS API
  - 🔧 TODO: Context menu copy/paste
- [ ] Basic form interaction (submit, checkbox/radio visuals, select dropdown)
  - Оценка: 3–6 недель

## Приоритет 2: Совместимость с реальными сайтами (lorachi.xyz, DDG, etc.)
- [ ] Полная/улучшенная поддержка CSS Grid + Flexbox edge-cases
  - minmax(auto-fit), repeat(auto-fit, ...), subgrid, nested grids
  - Проблема: карточки накладываются, masonry-эффекты ломаются на Tailwind-сайтах
  - Оценка: 3–6 недель (stylo crate)
- [ ] Dark mode (prefers-color-scheme + .dark class handling)
  - Часто игнорируется → светлая тема на тёмных сайтах
  - Оценка: 1–3 недели
- [ ] Fetch API + modern JS (async/await, modules, BigInt если нужно)
  - Чтобы DDG перешёл в full mode (filters, suggestions, bangs)
  - Оценка: 4–10 недель (script crate + mozjs/boa)
- [ ] Media / images improvements (lazy loading, object-fit/cover, aspect-ratio)
  - Обложки в lorachi растягиваются/ломаются
  - Оценка: 2–5 недель
- [ ] Shadow DOM + custom elements basics
  - Многие современные UI-компоненты (web components) ломаются
  - Оценка: 4–8 недель (roadmap приоритет)

## Приоритет 3: Performance & Stability
- [ ] Incremental layout / reflow optimization
  - Сейчас часто full relayout → тормоза при вводе/скролле
  - Оценка: 4–8 недель (layout engine)
- [ ] Reduce crashes на complex сайтах (memory leaks, panic в webrender)
  - Оценка: ongoing (mach test-wpt + fuzzing)
- [ ] Video playback streaming (Media Source Extensions)
  - Сейчас только full download → YouTube/Twitch часто не работают
  - Оценка: 4–10 недель

## Приоритет 4: Browser shell / UI фичи (сверху Servo)
- [ ] Tabbed browsing + session restore
- [ ] Bookmarks / history basics
- [ ] Address bar с автодополнением (duckduckgo suggestions если JS пофиксим)
- [ ] Context menu (copy, open in new tab, inspect если devtools)
- [ ] Ad/tracker "блокировка" как фича (пока за счёт несовместимости — потом optional filter lists)
- [ ] Dark/light theme switcher в UI (не зависит от сайта)

## Приоритет 5: Долгосрочные / nice-to-have
- [ ] WebGL/WebGPU full usability (уже частично есть)
- [ ] Accessibility (ARIA, screen reader support)
- [ ] Extensions subset (WebExtensions basics)
- [ ] Android embedding (уже экспериментально)
- [ ] Upstream патчи в servo/servo (чтобы не отставать от main)

## Тестирование & метрики
- Запускать на:
  - lorachi.xyz (Tailwind grid + dark + cards)
  - duckduckgo.com (full mode с фильтрами)
  - wikipedia.org (text selection + smooth scroll)
  - youtube.com (video playback)
- Целевые баллы: HTML5Test > 300–350, Acid3 ~80–90%
- Инструменты: mach test-wpt, wpt.fyi сравнение с Gecko

## Заметки
- Многие проблемы — не "баги", а missing features (Servo — research/embedding engine, не полноценный браузер).
- ИИ (Claude/Grok/o3) хорошо помогает с отдельными модулями (stylo, script), но debug/reflow — ручной.
- Обновление upstream Servo — rebase/merge каждые 1–2 месяца (если изменения в servo/ — конфликты будут).

Начни с caret + selection + smooth scroll — это даст самый большой прирост "не бесит".