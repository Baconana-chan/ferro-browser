# Ferro Browser — состояние и roadmap

Форк Servo для лёгкого, минималистичного браузера с упором на реальную ежедневную пригодность, а не только на то, чтобы «что-то рендерилось».

## Легенда
- `[!]` критический блокер
- `[~]` в активной работе
- `[ ]` запланировано
- `[x]` уже сделано и это стоит сохранять

## Текущее состояние на апрель 2026

### Что уже есть
- `[x]` Собираемый shell на базе Servo
- `[x]` Современная верхняя панель и вкладки в servoshell
- `[x]` Caret, базовое выделение текста мышью, copy/paste, smooth scroll
- `[x]` FFmpeg-стек вместо GStreamer
- `[x]` MSE / базовое медиавоспроизведение
- `[x]` Boa интегрирован как основной JS backend на уровне crates/features
- `[x]` В репо уже есть devtools backend:
  - remote devtools server
  - script/devtools handlers
  - devtools traits/messages
  - тесты для devtools-протокола

### Что реально ещё не готово
- `[!]` Boa runtime в живом браузере пока нестабилен: page load всё ещё упирается в null/realm/object shim проблемы
- `[!]` Нельзя честно считать миграцию SpiderMonkey → Boa завершённой, пока браузер не проходит обычный сценарий запуска сайтов без abort/panic
- `[!]` User-facing devtools в shell по сути нет: есть backend/remote plumbing, но нет нормального окна/панели инспектора для пользователя

### Ближайшая цель
Довести Ferro до состояния, где он:
- открывает `lorachi.xyz`, `duckduckgo.com`, `wikipedia.org` без падений
- даёт базовую интерактивность без раздражающих регрессий
- имеет понятный технический план дальнейшего развития

---

## Приоритет 0 — Разблокировать runtime браузера

### Цель
Убрать системные runtime-краши Boa backend и получить стабильную загрузку хотя бы нескольких реальных сайтов.

### Сейчас в работе
- `[~]` Починка Boa JSAPI shim:
  - realm/global tracking
  - fake JSObject storage
  - reserved slots
  - DOM reflector wiring
  - principals / private pointers
- `[~]` Замена `panic!`, `unimplemented!`, null-stub функций на минимально рабочие реализации
- `[~]` Аудит путей `create_global_object -> interface/prototype install -> DOM wrap -> callbacks`

### Нужно закрыть в первую очередь
- `[ ]` Убрать оставшиеся null-stub в `components/boa_bindings/js_compat/`
- `[ ]` Довести до рабочего состояния:
  - `CurrentGlobalOrNull`
  - `GetRealmGlobalOrNull`
  - `GetFunctionRealm`
  - `CallSetup` / callback path
  - object/class/proxy/unwrap helpers
- `[ ]` Проверить создание глобала, интерфейсов и прототипов без null dereference
- `[ ]` Закрыть все runtime-assert, завязанные на SpiderMonkey-предположения
- `[ ]` Прогнать ручной smoke test:
  - `about:blank`
  - простая HTML-страница
  - `wikipedia.org`
  - `duckduckgo.com`
  - `lorachi.xyz`

### Definition of done
- `[ ]` `cargo check -p servoshell --features js-boa` стабильно чистый
- `[ ]` `ferro.exe` не падает при старте и первой навигации
- `[ ]` хотя бы 3 целевых сайта открываются без abort/panic

---

## Приоритет 1 — Браузер, которым можно пользоваться каждый день

### Уже сделано
- `[x]` Visible caret + blinking
- `[x]` Mouse text selection + highlight
- `[x]` Smooth scrolling
- `[x]` Clipboard shortcuts для text inputs
- `[x]` Современный верхний бар / адресная строка / вкладки

### Следующие обязательные вещи
- `[ ]` Basic form interaction
  - корректный submit
  - checkbox/radio visuals
  - select dropdown
  - disabled/readOnly edge-cases
- `[ ]` Address bar как полноценная команда ввода
  - URL vs search detection
  - нормальный search fallback
  - навигация по Enter / paste / reload
- `[ ]` Tabbed browsing + session restore
- `[ ]` Bookmarks / history basics
- `[ ]` Context menu
- `[ ]` Загрузка файлов и базовые диалоги выбора файла

### Полировка UX
- `[ ]` Auto-scroll при drag selection за край viewport
- `[ ]` Prefers-reduced-motion для caret / smooth scroll
- `[ ]` Цвета selection / highlight через preferences
- `[ ]` Better focus rings / keyboard navigation

---

## Приоритет 2 — Совместимость с реальными сайтами

### Layout / CSS
- `[x]` Значительная работа по Grid/Flex edge-cases уже была сделана
- `[ ]` Проверить текущий реальный эффект на Tailwind-сайтах, а не только по локальным тестам
- `[ ]` `prefers-color-scheme` и корректный dark mode
- `[ ]` Container queries / responsive edge-cases
- `[ ]` object-fit / aspect-ratio / lazy loading regressions

### DOM / JS / platform APIs
- `[ ]` Shadow DOM basics
- `[ ]` Custom Elements basics
- `[ ]` ResizeObserver usable implementation
- `[ ]` IntersectionObserver improvements
- `[ ]` ES modules runtime edge-cases
- `[ ]` Worker / Service Worker story: честно разделить stubs и реальные возможности
- `[ ]` Navigator / Clipboard / Permissions довести до usable уровня, а не только до формы API

### Целевые сайты для совместимости
- `[ ]` `lorachi.xyz`
- `[ ]` `duckduckgo.com` full mode
- `[ ]` `github.com`
- `[ ]` `wikipedia.org`
- `[ ]` хотя бы базовая выживаемость на типичном SPA без немедленного краша

---

## Приоритет 3 — Devtools для пользователя

### Честный статус
- `[x]` Backend для devtools уже существует
- `[x]` Есть remote devtools server и протокольные типы
- `[x]` Есть integration/tests для devtools-коммуникации
- `[ ]` Нет нормального user-facing devtools внутри браузера

### Рекомендуемый план

#### Этап A — сделать существующий backend реально полезным
- `[ ]` Проверить, что remote devtools server реально запускается и цепляется к текущему shell
- `[ ]` Добавить понятный CLI/setting для включения devtools server
- `[ ]` Нормально логировать старт порта, токен и ошибки подключения
- `[ ]` Проверить совместимость с внешним фронтендом инспектора

#### Этап B — первый user-facing devtools
- `[ ]` Открытие devtools как отдельного окна shell, а не docked UI
- `[ ]` Минимальный набор панелей:
  - Elements
  - Console
  - Network
  - Sources
- `[ ]` Переключение devtools для активной вкладки
- `[ ]` Базовый inspect element

#### Этап C — нормальный встроенный инструмент разработчика
- `[ ]` Docked/undocked режим
- `[ ]` Live DOM tree
- `[ ]` Computed styles / box model
- `[ ]` Console evaluation в текущем global
- `[ ]` Network request list и ошибки
- `[ ]` Source viewer / breakpoints
- `[ ]` Performance timeline / reflow markers

### Практичный вывод
Если полноценный встроенный frontend слишком дорогой по времени, сначала стоит довести remote devtools до состояния, где им можно пользоваться из внешнего клиента, а уже потом делать свой UI в shell.

---

## Приоритет 4 — Stability и performance

- `[ ]` Incremental layout / reflow optimization
- `[ ]` Уменьшить число full relayout при вводе и скролле
- `[ ]` Panic audit для `script`, `script_bindings`, `boa_bindings`
- `[ ]` Memory leak audit
- `[ ]` Regression suite на реальные страницы
- `[ ]` Crash triage workflow:
  - воспроизведение
  - минимальный URL/fixture
  - фикс корневой причины
  - smoke test после фикса

### Метрики успеха
- `[ ]` Старт shell без panic
- `[ ]` Навигация между несколькими вкладками без деградации
- `[ ]` Нет частых full-abort при сложных DOM/CSS страницах

---

## Приоритет 5 — Медиа и графика

### Уже сделано
- `[x]` FFmpeg migration
- `[x]` Базовый MSE path
- `[x]` Standalone media documents

### Дальше
- `[ ]` Проверить реальное воспроизведение на YouTube-подобных сценариях, где это возможно без DRM
- `[ ]` GIF / animated image smoothness
- `[ ]` AVIF / WebP / Windows codec story
- `[ ]` WebGL usability
- `[ ]` WebGPU usability

### Осознанные ограничения
- `[ ]` EME / DRM пока не приоритет для ближайшего milestone
- `[ ]` WebCodecs имеет смысл только после стабилизации базового runtime

---

## Приоритет 6 — Shell и продуктовая оболочка

- `[ ]` Полноценные настройки
- `[ ]` Search engine settings
- `[ ]` Theme switcher
- `[ ]` Download manager basics
- `[ ]` Простая страница новой вкладки
- `[ ]` Session persistence
- `[ ]` Packaging для Windows/Linux/macOS

---

## Boa migration — честный статус

### Что правда уже сделано
- `[x]` Boa подключён в workspace и feature flags
- `[x]` Существует большой слой совместимости для `script` / `script_bindings`
- `[x]` Есть набор unit/integration тестов для Boa crate-level логики
- `[x]` SpiderMonkey уже не единственный путь сборки

### Что пока нельзя считать завершённым
- `[!]` Миграция не завершена на уровне реального браузерного runtime
- `[!]` Пока shell падает на реальных сайтах, это всё ещё активный инфраструктурный проект, а не finished migration

### Критерий завершения миграции
- `[ ]` Boa default build стабильно запускает shell
- `[ ]` Проходит smoke test по реальным сайтам
- `[ ]` Devtools/console/eval не ломают runtime
- `[ ]` Нет системных null-stub путей в критическом JSAPI слое

---

## Целевые сайты и сценарии тестирования

### Daily smoke test
- `[ ]` `about:blank`
- `[ ]` локальная тестовая HTML-страница
- `[ ]` `wikipedia.org`
- `[ ]` `duckduckgo.com`
- `[ ]` `lorachi.xyz`

### Дополнительные сценарии
- `[ ]` форма логина / поиск / textarea
- `[ ]` copy-paste / selection / keyboard navigation
- `[ ]` открытие второй вкладки и возврат назад/вперёд
- `[ ]` видео / аудио / изображение / GIF
- `[ ]` открытие devtools server и подключение клиента

### Инструменты
- `mach test-wpt`
- локальные regression fixtures
- точечные smoke tests после каждого crash fix
- devtools protocol tests

---

## Следующий milestone

### Milestone: «Browser boots and survives»
- `[ ]` shell запускается с Boa по умолчанию
- `[ ]` первая навигация не падает
- `[ ]` DOM wrapping / realm tracking не ломают page init
- `[ ]` можно открыть хотя бы 3 реальных сайта подряд
- `[ ]` есть понятный devtools plan и хотя бы рабочий remote path

После этого уже имеет смысл aggressively расширять web-platform coverage и polishing shell UI.
