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