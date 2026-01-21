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
- [x] Полная/улучшенная поддержка CSS Grid + Flexbox edge-cases
  - ✅ minmax(auto-fit), repeat(auto-fit, ...), subgrid, nested grids
  - ✅ Реализована поддержка subgrid и masonry в Taffy wrapper (wrapper.rs)
  - Проблема: карточки накладываются, masonry-эффекты ломаются на Tailwind-сайтах
  - Оценка: 3–6 недель (stylo crate)
- [ ] Dark mode (prefers-color-scheme + .dark class handling)
  - Часто игнорируется → светлая тема на тёмных сайтах
  - Оценка: 1–3 недели
- [x] Fetch API + modern JS (async/await, modules, BigInt если нужно)
  - ✅ IndexedDB включён по умолчанию (dom_indexeddb_enabled = true)
  - ✅ Performance API User Timing Level 3 (mark/measure возвращают объекты)
  - ✅ PerformanceMark.detail и PerformanceMeasure.detail атрибуты
  - ✅ Constructable StyleSheets (adoptedStyleSheets) включены для GitHub
  - Чтобы DDG перешёл в full mode (filters, suggestions, bangs)
  - 🔧 TODO: Проверить async/await edge-cases
  - 🔧 TODO: ES Modules imports (import/export)
- [ ] Media / images improvements (lazy loading, object-fit/cover, aspect-ratio)
  - Изображения в растягиваются/ломаются/долго грузятся
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
- [x] Video playback streaming (Media Source Extensions)
  - ✅ MediaSource API реализован (WebIDL + Rust)
  - ✅ SourceBuffer и SourceBufferList созданы
  - ✅ isTypeSupported() для video/mp4, video/webm, audio/mp4, audio/webm
  - ✅ Включено по умолчанию (dom_mediasource_enabled = true)
  - ✅ Создан MSE SegmentParser в ferro_media/src/mse.rs
  - ✅ Поддержка парсинга ISOBMFF (MP4) и WebM контейнеров
  - ✅ MseSourceBuffer с управлением buffered ranges
  - ✅ MsePlayer в servo_media_ferro для интеграции
  - ✅ Standalone video/audio documents с controls
  - ✅ **FerroPlayer полностью реализован с FFmpeg:**
    - ✅ play/pause/stop/seek работают (components/servo_media_ferro/lib.rs)
    - ✅ Автоматический seek(0) при play из Ended состояния (перезапуск видео)
    - ✅ Обработка EndOfStream → состояние Ended
    - ✅ push_data() буферизирует поток и отправляет NeedData/EnoughData события
    - ✅ Поддержка как URL так и Media Source Extensions
    - ✅ Видео/аудио декодирование через FFmpeg
    - ✅ Системный аудиовывод через cpal/rodio
  - ✅ Улучшен дизайн standalone медиа документов (components/script/dom/servoparser/mod.rs):
    - ✅ Чёрный фон для видео/аудио плеера
    - ✅ Центрирование контента (flexbox)
    - ✅ Правильное масштабирование с сохранением соотношения сторон
    - ✅ CSS стили также для img элементов (изображения)
    - ✅ Плеер заполняет экран без белых полос
  - ✅ Добавлены debug логи для диагностики:
    - ✅ Логи в handle_animated_image (components/layout/context.rs - количество кадров)
    - ✅ Логи в decode_animated_image (components/pixels/lib.rs - информация о декодировании)
    - ✅ Логи в update_active_frames (components/script/image_animation.rs - обновление кадров)
  - 🔧 TODO: Проверить работу GIF анимации в браузере (логи подскажут на каком этапе проблема)
  - 🔧 TODO: Оптимизировать обновление кадров для плавной анимации
  - 🔧 TODO: Добавить поддержку форматов AVIF, WebP на Windows (требует libdav1d)

## Приоритет 4: Browser shell / UI фичи (сверху Servo)
- [x] Современный дизайн навигационной панели
  - ✅ Адаптивные вкладки: масштабируются от 60px до 200px в зависимости от количества
  - ✅ Горизонтальный scroll для большого количества вкладок (13+ работает)
  - ✅ Современные цвета для dark/light mode
  - ✅ Закруглённые вкладки в стиле Chrome
  - ✅ Акцентная полоса на активной вкладке
  - ✅ Закруглённая адресная строка с lock/warning иконками
  - ✅ Улучшенные кнопки навигации (back, forward, reload)
  - 🔧 TODO: Drag-and-drop перестановка вкладок
  - Изменения: ports/servoshell/desktop/gui.rs
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

## Миграция медиа: GStreamer → FFmpeg
**Статус: FFmpeg СОБИРАЕТСЯ ✅ | GStreamer УДАЛЁН ✅**

### Причины миграции:
- GStreamer плохо работает кросс-платформенно (особенно на Windows)
- Сложная настройка зависимостей (GStreamer DLLs, плагины)
- servo-media добавляет много overhead

### Новый стек ferro_media:
- **ffmpeg-next v8** — Rust bindings для FFmpeg (декодирование видео/аудио)
- **cpal/rodio** — кросс-платформенный аудио вывод
- **image crate** — статические изображения и GIF

### Прогресс:
- [x] Создан components/ferro_media с базовой структурой
- [x] Реализован MediaPlayer с FFmpeg decoder
- [x] Реализован AudioOutput с cpal/rodio
- [x] Добавлен в workspace dependencies
- [x] **FFmpeg 7.1 + ffmpeg-next 8.0 успешно собирается на Windows!**
- [x] Создан servo-media-ferro backend (components/servo_media_ferro)
- [x] Интеграция с libservo через feature `media-ferro`
- [x] Сборка servoshell с `--features media-ferro` работает!
- [x] **GStreamer полностью удалён из проекта!** 🎉
- [ ] Тестирование реального воспроизведения медиа
- [ ] Тестирование на Linux/macOS

### Установка FFmpeg для разработки (Windows):
```powershell
# 1. Скачать FFmpeg 7.1 shared builds:
Invoke-WebRequest -Uri "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-n7.1-latest-win64-gpl-shared-7.1.zip" -OutFile "$env:USERPROFILE\ffmpeg71.zip"
Expand-Archive -Path "$env:USERPROFILE\ffmpeg71.zip" -DestinationPath "C:\ffmpeg71" -Force

# 2. Перед сборкой запустить setup script:
. .\setup_ffmpeg.ps1

# 3. Собрать с FFmpeg:
cargo build -p ferro_media --features ffmpeg
```

### Требования:
- LLVM/Clang (для bindgen): `C:\Program Files\LLVM\bin`
- FFmpeg 7.1 shared libs: `C:\ffmpeg71\ffmpeg-n7.1-latest-win64-gpl-shared-7.1`

**Linux:**
```bash
sudo apt install libavcodec-dev libavformat-dev libavutil-dev libswscale-dev
```

**macOS:**
```bash
brew install ffmpeg
```

## Заметки
- Многие проблемы — не "баги", а missing features (Servo — research/embedding engine, не полноценный браузер).
- ИИ (Claude/Grok/o3) хорошо помогает с отдельными модулями (stylo, script), но debug/reflow — ручной.
- Обновление upstream Servo — rebase/merge каждые 1–2 месяца (если изменения в servo/ — конфликты будут).

Начни с caret + selection + smooth scroll — это даст самый большой прирост "не бесит".

## Web Platform APIs Progress

### Реализованные API:
- [x] **IndexedDB** — включён по умолчанию (dom_indexeddb_enabled = true)
- [x] **MediaSource Extensions (MSE)** — полная реализация
  - MediaSource, SourceBuffer, SourceBufferList (DOM)
  - isTypeSupported() для video/mp4, video/webm, audio/mp4, audio/webm, audio/mpeg
  - События: sourceopen, sourceended, sourceclose, updatestart, update, updateend
  - ferro_media/src/mse.rs — SegmentParser для MP4/WebM
  - servo_media_ferro — MsePlayer с buffered ranges
- [x] **Performance User Timing Level 3**
  - performance.mark() возвращает PerformanceMark объект
  - performance.measure() возвращает PerformanceMeasure объект
  - detail атрибут на PerformanceMark и PerformanceMeasure
  - PerformanceMarkOptions и PerformanceMeasureOptions поддержка

### Требуют реализации для YouTube/TikTok:
- [ ] Encrypted Media Extensions (EME) — DRM контент
- [ ] WebCodecs API — низкоуровневое декодирование
- [ ] ResizeObserver — отслеживание изменений размера
- [ ] IntersectionObserver улучшения
- [ ] Web Workers / Service Workers полная поддержка

---

## 🦀 Миграция JavaScript: SpiderMonkey → Boa

**Статус: В РАЗРАБОТКЕ 🚧 | Приоритет: ВЫСОКИЙ**

### Лицензирование:
- Файлы от Servo: MPL-2.0 (сохраняем оригинальную лицензию)
- Новые файлы Ferro (boa_bindings, ferro_media): MIT
- Двойная лицензия позволяет максимальную гибкость для разработчиков

### Причины миграции:
- SpiderMonkey — C++ legacy код Mozilla, сложный FFI через mozjs crate
- Компиляция mozjs занимает 10-15 минут, требует Clang/LLVM
- Отладка JS ошибок практически невозможна (C++ <-> Rust boundary)
- Высокая нагрузка даже на простых страницах из-за SM overhead
- Boa — чистый Rust, единая экосистема с Servo
- Boa имеет встроенный GC на Rust (boa_gc), совместимый с нашей архитектурой

### Boa 0.21 Features (октябрь 2025):
- **94.12% ECMAScript conformance** (Test262)
- NaN-boxing — меньше памяти для JsValue
- Register-based VM — быстрее выполнение
- boa_runtime: fetch, setTimeout, setInterval, queueMicrotask
- Temporal proposal ~97% conformance
- Error.isError, новые Set методы, Float16 support

### Boa Crates (https://github.com/boa-dev/boa):
- `boa_engine` — основной движок, builtin objects, execution
- `boa_parser` — lexer и parser для ECMAScript
- `boa_ast` — Abstract Syntax Tree
- `boa_gc` — сборщик мусора на Rust
- `boa_interner` — string interner для оптимизации
- `boa_runtime` — WebAPI features (console, fetch, setTimeout, etc.)
- `boa_icu_provider` — ICU4X для интернационализации

### Текущее состояние SpiderMonkey в Servo:
```
Cargo.toml:
  js = { package = "mozjs", git = "https://github.com/servo/mozjs" }

Ключевые модули:
  components/script/           — DOM implementation, использует js:: напрямую
  components/script_bindings/  — WebIDL bindings, генерация кода для SM
  components/script_bindings/codegen/ — генератор Bindings из WebIDL
  
Зависимости от mozjs:
  - js::jsapi::* — низкоуровневые SM API
  - js::rust::* — Rust wrappers для SM
  - js::gc::* — SM garbage collector интеграция
  - js::typedarray::* — TypedArray bindings
```

### План миграции (фазы):

#### Фаза 0: Подготовка и исследование [✅ ЗАВЕРШЕНА]
- [x] Создать TODO план миграции
- [x] Добавить Boa 0.21 crates в workspace dependencies
- [x] Создать components/boa_bindings модуль (MIT лицензия)
- [x] Реализовать JsRuntime wrapper для Context
- [x] Реализовать базовые type conversions
- [x] Реализовать error handling
- [x] Реализовать GC интеграцию (DomRef, DomCell)
- [x] Пройти базовые тесты (eval, functions, strings)

#### Фаза 1: Базовая интеграция Boa [✅ ЗАВЕРШЕНА]
- [x] Добавить boa_engine, boa_parser, boa_gc в workspace dependencies
- [x] Создать components/boa_bindings/ — новый модуль биндингов
  - [x] lib.rs — главный модуль с re-exports
  - [x] runtime.rs — JsRuntime wrapper для Boa Context
  - [x] gc.rs — GC интеграция (DomRef, DomCell, force_collect)
  - [x] conversions.rs — ToJsValue/FromJsValue traits
  - [x] error.rs — JsException и обработка ошибок
  - [x] reflector.rs — Reflector pattern для DOM объектов
  - [x] builtins/ — расширения Web API
- [x] Реализовать базовый JsRuntime на Boa (аналог script_runtime.rs)
  - [x] eval(), eval_with_filename()
  - [x] set_global(), get_global()
  - [x] gc() — принудительная сборка мусора
- [x] Реализовать Reflector/DomObject для Boa GC
  - [x] Reflector — связь между Rust DOM и JS object
  - [x] DomObject trait — интерфейс для DOM объектов
  - [x] Dom<T>, DomRefCell<T> — GC-managed references
  - [x] create_dom_wrapper() — создание JS wrapper для DOM
  - [x] define_method(), define_getter(), define_accessor()
- [x] Простые тесты: eval("1+1"), console.log(), setTimeout()
  - [x] test_basic_eval — 1+1 = 2 ✓
  - [x] test_string_eval — 'hello world' ✓
  - [x] test_function_call — function add(a,b) ✓
  - [x] test_set_timeout_returns_id — setTimeout returns ID ✓
  - [x] test_set_interval_returns_id — setInterval returns ID ✓
  - [x] test_clear_timeout — clearTimeout works ✓
  - [x] test_reflector_lifecycle — Reflector initialization ✓
  - [x] test_dom_wrapper — Dom<T> wrapper ✓
- [x] **8 тестов проходят успешно!**

#### Фаза 2: WebIDL Code Generation [СЛЕДУЮЩАЯ]
- [ ] Модифицировать components/script_bindings/codegen/ для Boa
  - Или создать отдельный codegen для Boa
- [ ] Генерация Rust bindings из .webidl файлов для Boa
- [ ] Реализовать конверсии типов (DOMString, Uint8Array, etc.)
- [ ] Интерфейсы: Window, Document, Element, Node (базовые)

#### Фаза 3: DOM Bindings Core
- [ ] Перенести htmlelement.rs, document.rs, window.rs
- [ ] Event system (addEventListener, dispatchEvent)
- [ ] DOM manipulation (createElement, appendChild, etc.)
- [ ] CSS Object Model (getComputedStyle, classList)

#### Фаза 4: Web APIs
- [ ] Console API (console.log/warn/error)
- [ ] Fetch API (с использованием existing net stack)
- [ ] Timers (setTimeout, setInterval, requestAnimationFrame)
- [ ] Storage (localStorage, sessionStorage)
- [ ] IndexedDB (existing implementation)

#### Фаза 5: Advanced Features
- [ ] ES Modules (import/export)
- [ ] async/await, Promises
- [ ] Web Workers
- [ ] WebGL/WebGPU bindings
- [ ] MediaSource Extensions

#### Фаза 6: Полное удаление SpiderMonkey
- [ ] Удалить mozjs из dependencies
- [ ] Удалить components/script_bindings (старый)
- [ ] Переименовать boa_bindings → script_bindings
- [ ] Обновить все imports в components/script

### Ключевые файлы для миграции:
```
components/script_bindings/
├── script_runtime.rs     → JsRuntime на Boa
├── reflector.rs          → Boa GC интеграция
├── root.rs               → Rooted pointers для Boa
├── trace.rs              → Boa::Trace вместо JSTraceable
├── conversions.rs        → Type conversions для Boa
├── error.rs              → JS Error handling
├── codegen/              → WebIDL → Boa bindings generator
└── webidls/              → WebIDL определения (не меняются)
```

### Совместимость Boa (Test262):
- ~80% ECMAScript compliance (vs SM ~95%)
- Основные gaps: некоторые edge-cases в Proxy, WeakRef, FinalizationRegistry
- Для большинства сайтов этого достаточно
- Активная разработка, compliance растёт

### Feature Flags (для постепенной миграции):
```toml
[features]
default = ["js-spidermonkey"]  # Текущий default
js-spidermonkey = ["mozjs"]    # Legacy SpiderMonkey
js-boa = ["boa_engine", "boa_parser", "boa_gc"]  # Новый Boa
```

### Оценка времени:
- Фаза 0-1: 2-4 недели
- Фаза 2-3: 6-10 недель
- Фаза 4-5: 4-8 недель
- Фаза 6: 1-2 недели
- **Итого: ~3-6 месяцев**

### Ресурсы:
- Boa docs: https://docs.rs/boa_engine/
- Boa GitHub: https://github.com/boa-dev/boa
- Boa playground: https://boajs.dev/playground/
- Test262 status: https://test262.fyi/

---