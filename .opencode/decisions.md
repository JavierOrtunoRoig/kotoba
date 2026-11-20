# Decisiones

Formato ligero de ADR: contexto → decisión → consecuencias. Las decisiones no se borran: si cambian,
se añade una nueva entrada que sustituye a la anterior.

---

## D-001 — Stack Tauri 2 + React 19 + TypeScript + Vite + SQLite

**Contexto:** app de escritorio Windows, siempre encendida, con requisito de RAM baja.
**Decisión:** Tauri 2 (Rust + WebView2) con frontend React 19/TS/Vite y SQLite embebido con
`rusqlite` (`bundled`).
**Consecuencias:** binario y consumo menores que Electron; se paga con complejidad de Rust y
WebView2. `rusqlite` da control total del esquema y del importador.

## D-002 — Solo Windows

**Contexto:** el usuario objetivo usa Windows y el requerimiento original lo fija.
**Decisión:** no se mantienen ramas macOS/Linux en la Fase 1. La bandeja, el autostart y las
ventanas se implementan con las APIs de Windows.
**Consecuencias:** menos superficie de pruebas; si se porta, habrá que revisar bandeja, DND y
detección de pantalla completa.

## D-003 — Importación principal desde `.apkg`

**Contexto:** el mazo `Genki 1` incluye audio; el `.txt` de Anki solo guarda referencias
`[sound:...]`.
**Decisión:** `.apkg` es el formato principal. `.txt`/`.tsv`/`.csv` se aceptan como fallback sin
audio.
**Consecuencias:** el importador debe leer ZIP + zstd + SQLite schema 18 + protobuf. El `.txt`
queda limitado a vocabulario sin audio y asume el orden de campos de Genki.

## D-004 — Leer el formato nuevo de Anki

**Contexto:** los `.apkg` actuales traen `collection.anki2` como placeholder y la base real en
`collection.anki21b` comprimida con zstd.
**Decisión:** buscar en orden `collection.anki21b` → `collection.anki21` → `collection.anki2`,
descomprimir zstd si hace falta y validar la cabecera `SQLite format 3\0`.
**Consecuencias:** funciona con Anki moderno; los `.apkg` antiguos siguen funcionando si su
`collection.anki2` es SQLite real. El mensaje “actualice Anki” del placeholder se ignora.

## D-005 — Mapear campos por nombre, con fallback posicional

**Contexto:** el notetype `Genki Model ES` tiene 8 campos, pero otros mazos pueden variar.
**Decisión:** mapear por nombre (`kana`, `kanjis`, `español`, `significado`, `sonido`, `uid`) y, si
no hay coincidencia, usar el orden de Genki: 0 lectura, 1 kanji, 3 significado, 4 glosa, 5 audio,
7 uid.
**Consecuencias:** el importador es reutilizable; el fallback posicional puede equivocarse en mazos
muy distintos y habrá que añadir una pantalla de mapeo si se generaliza.

## D-006 — Deduplicar por lectura + significado

**Contexto:** 1.076 notas producen 1.012 ítems porque el vocabulario se repite entre subdecks.
**Decisión:** clave de deduplicación = `normalize_key(reading) + "|" + normalize_key(meaning)`.
Se conserva el primer ítem y todas las fuentes en `item_sources`.
**Consecuencias:** menos tarjetas duplicadas; se pierde la noción de “una tarjeta por lección”
salvo por `item_sources`. Reimportar es idempotente por `source_uid`.

## D-007 — Plantillas y gramática etiquetadas, excluidas por defecto

**Contexto:** 66 notas usan `〜`, `+ negative` o `...` y no son vocabulario tipeable limpio.
**Decisión:** marcarlas con `is_pattern = true`; el scheduler las excluye salvo que
`settings.include_patterns = true`.
**Consecuencias:** el MVP muestra vocabulario “puro”; el usuario puede activarlas en Ajustes.

## D-008 — Audio automático solo en JP → ES

**Contexto:** el audio revela la respuesta en ES → JP.
**Decisión:** en JP → ES se reproduce al mostrar la tarjeta si `audio_autoplay` está activo; en
ES → JP se reproduce después de responder.
**Consecuencias:** UX alineada con las plantillas de Anki. El audio es opcional: si falta o falla,
la tarjeta sigue funcionando.

## D-009 — SRS ligero provisional

> Sustituida por [D-022](#d-022--sm-2-completo-sustituye-a-d-009).

**Contexto:** el plan deja el SM-2 completo para la Fase 2.
**Decisión:** intervalos fijos `[10 min, 1 h, 6 h, 1 d, 3 d, 7 d, 14 d, 30 d]`; acierto sube un
escalón, fallo vuelve a 10 min. `ease` se guarda pero aún no se usa.
**Consecuencias:** la app ya reparte repasos de forma útil; la Fase 2 sustituirá la lógica sin
cambiar el esquema.

## D-010 — Todo local, sin red

**Contexto:** requisito de privacidad y de app siempre encendida.
**Decisión:** SQLite y audio en `%APPDATA%\com.kotoba.app`; ninguna llamada de red en la Fase 1.
**Consecuencias:** no hay sincronización ni telemetría. AnkiConnect queda para Fase 3 y será opt-in.

## D-011 — Un solo modo de aparición en Fase 1: popup

> Ampliada por [D-023](#d-023--tres-modos-de-aparición-y-foco-sustituye-a-d-011).

**Contexto:** el plan pide 3 modos (widget, popup, toast) en Fase 2.
**Decisión:** la Fase 1 implementa solo el popup frameless/always-on-top, creado bajo demanda.
**Consecuencias:** menos trabajo inicial; el scheduler y los ajustes ya están preparados para añadir
los otros dos modos.

## D-012 — Estética “washi” con fuentes del sistema

**Contexto:** app japonesa, de uso pasivo, que no debe parecer otra app genérica.
**Decisión:** papel cálido, tinta, bermellón, sello 言, tipografía serif/Mincho; fuentes del sistema
(`Yu Mincho`, `Constantia`, `Yu Gothic UI`). Sin fuentes externas ni red.
**Consecuencias:** identidad visual clara y offline. En equipos sin esas fuentes hay fallbacks;
si se quiere consistencia total, se pueden empaquetar fuentes en Fase 2.

## D-013 — Ignorar el scheduling de Anki

**Contexto:** el `.apkg` trae 352 tarjetas en repaso y 1.151 entradas de `revlog`.
**Decisión:** importar todo como nuevo; kotoba mantiene su propio SRS.
**Consecuencias:** no se pisa el progreso de Anki y no se heredan intervalos que no corresponden a
la app. El usuario puede reiniciar el progreso sin tocar Anki.

## D-014 — Conventional Commits

**Contexto:** el usuario lo pidió explícitamente.
**Decisión:** todos los commits usan `tipo(scope): resumen` con cuerpo opcional.
**Consecuencias:** historial legible y compatible con changelogs automáticos.

## D-015 — Versionar `Cargo.lock`

**Contexto:** es una aplicación binaria, no una librería.
**Decisión:** `Cargo.lock` se commitea para builds reproducibles. `skills-lock.json` y `.agents/`
quedan fuera del control de versiones por ser tooling de agentes.
**Consecuencias:** builds más deterministas; hay que actualizar el lock al cambiar dependencias.

## D-016 — `significado` es glosa de kanji; `tipo` está vacío

**Contexto:** el mazo usa `español` para el significado y `significado` para una glosa kanji a
kanji (“Diez - Cuatro”). El campo `tipo` está vacío en las 1.076 notas.
**Decisión:** ignorar `tipo`; usar `español` como significado y `significado` como
`kanji_gloss` opcional.
**Consecuencias:** el feedback puede mostrar el desglose del kanji sin contaminar la respuesta.

## D-017 — `uid` como clave estable; `id_orden` no es global

**Contexto:** `id_orden` solo llega hasta 61 y se repite por subdeck; `uid` es único
(`Genki 1::L00::01 Saludos_0`).
**Decisión:** usar `uid` como `source_uid` para idempotencia. `id_orden` no se usa como orden global.
**Consecuencias:** reimportar no duplica fuentes; el orden de presentación lo decide el scheduler.

## D-018 — Reglas de normalización de respuestas

**Contexto:** el mazo mezcla kanji, kana, variantes con `／`, opcionales `（…）`, `*` y `〜`.
**Decisión:** NFKC, minúsculas, sin diacríticos, sin espacios ni puntuación. En ES → JP se
aceptan kanji y todas las variantes de lectura. En JP → ES se aceptan las acepciones separadas por
`;` y las variantes sin paréntesis ni `...`.
**Consecuencias:** tolerante con el usuario; puede aceptar alguna variante de más. La Fase 2
ampliará con más casos de IME y partículas.

## D-019 — Ventana principal oculta si ya hay biblioteca

**Contexto:** la app debe vivir en la bandeja y no abrirse sola.
**Decisión:** en el arranque, mostrar la ventana principal solo si `items` está vacía (onboarding).
Si hay biblioteca, arranca en la bandeja. Cerrar la ventana la oculta, no la destruye.
**Consecuencias:** el primer arranque es descubrible; los siguientes son discretos. “Salir” del menú
de bandeja termina el proceso.

## D-020 — `max_per_day` cuenta repasos respondidos

**Contexto:** el usuario puede cerrar un popup sin responder (“Ahora no”).
**Decisión:** solo se registra en `review_log` al enviar una respuesta (correcta o incorrecta).
Cerrar sin responder no cuenta para el máximo diario.
**Consecuencias:** el límite mide esfuerzo real; el scheduler puede reintentar más tarde.

## D-021 — El contador de pendientes respeta `include_patterns`

**Contexto:** la barra lateral y la tarjeta mostraban cifras distintas (1.012 vs 949) porque una
incluía plantillas y la otra no.
**Decisión:** `stats.due_now` se filtra con `include_patterns`, igual que `next_challenge`.
**Consecuencias:** ambas cifras coinciden y el usuario ve el mismo universo de repaso.

---

## D-022 — SM-2 completo (sustituye a D-009)

**Contexto:** el SRS ligero de la Fase 1 no usaba `ease` ni distinguía la calidad de la respuesta.
**Decisión:** calidad 1-5 (5 = acierto rápido, 4 = acierto, 2 = fallo, 1 = no lo sé); pasos de
aprendizaje de 10 min y 1 día; después, intervalo = intervalo anterior × `ease`; `ease` se actualiza
con la fórmula SM-2 y suelo 1.3; un fallo reinicia `reps`, vuelve a 10 min e incrementa `lapses`.
**Consecuencias:** el repaso se adapta al rendimiento real. La calidad se guarda en
`review_log.quality` para futuras estadísticas.

## D-023 — Tres modos de aparición y foco (sustituye a D-011)

**Contexto:** el plan pide widget, popup y toast; además, el popup no debe robar el foco.
**Decisión:** tres modos configurables (`popup`, `widget`, `toast`) con selector en Ajustes y
submenú “Modo” en la bandeja. El widget es persistente; el popup se crea bajo demanda; el toast usa
`tauri-plugin-notification` y su click abre el popup. Para no robar foco, se muestra la ventana con
`window.show()` y se devuelve el foco a la ventana anterior con `SetForegroundWindow`. La pantalla
completa, presentaciones, juegos y el modo silencio se detectan con `SHQueryUserNotificationState`.
**Consecuencias:** el usuario elige el nivel de intrusión. El popup aparece sin quedarse con el
teclado; el widget requiere un click para escribir.

## D-024 — Migraciones con `PRAGMA user_version`

**Contexto:** la Fase 1 solo usaba `CREATE TABLE IF NOT EXISTS`; añadir `quality` a `review_log` no
se aplicaba a bases existentes.
**Decisión:** leer `PRAGMA user_version` y aplicar migraciones incrementales idempotentes
(`ensure_column`). La versión 1 añade `review_log.quality`.
**Consecuencias:** las bases antiguas se actualizan sin perder datos. Las próximas migraciones
añaden un bloque `if version < N`.

## D-025 — Romaji como entrada alternativa

**Contexto:** no todos los usuarios tienen un IME japonés activo.
**Decisión:** en ES → JP se acepta romaji además de kanji y kana. El conversor es Hepburn
aproximado, con variantes para vocales largas (`ou`/`oo` → `o`), kunrei (`shi`/`si`), nasal
(`nb` → `mb`) y は final → wa.
**Consecuencias:** se puede practicar sin IME. Algunas variantes pueden aceptar una forma poco
ortodoxa (por ejemplo, を como `o`), pero la lectura en kana sigue siendo la fuente principal.

## D-026 — Gamificación silenciosa y XP por calidad

**Contexto:** la gamificación clásica (rachas con urgencia, XP por volumen, notificaciones de
“racha en peligro”) choca con la identidad pasiva de kotoba y puede corromper el SM-2.
**Decisión:** gamificación híbrida. El XP solo se gana por aciertos ponderados por `quality`
(3→+1, 4→+2, 5→+3); la racha es flexible y sin castigo; las notificaciones de hito son opt-in y
respetan pausa/DND/pantalla completa; sin cuentas ni social. Se entrega en fases: **R3-04a**
racha + objetivo semanal + XP/nivel, **R3-04b** logros/maestría y **R3-04c** congelar racha y
refinamiento.
**Consecuencias:** se puede retener sin convertir la app en una obligación diaria. La curva de
niveles y el catálogo de logros quedan para b/c; el XP no incentiva responder rápido o mal.

## D-027 — Logros permanentes y maestría por lección

**Contexto:** R3-04b necesita hitos que celebren constancia y retención sin introducir presión
diaria ni comparaciones sociales.
**Decisión:** catálogo fijo de 16 logros calculados a partir de datos reales (repasos, palabras
dominadas, racha, lecciones, nivel y objetivos semanales). El desbloqueo se guarda en la tabla
`achievements` y no se pierde con el tiempo. Una lección se considera dominada cuando el porcentaje
de palabras con intervalo ≥ 7 días alcanza el umbral configurable (default 90 %). Las
notificaciones de desbloqueo son opt-in y respetan pausa/DND/pantalla completa.
**Consecuencias:** el progreso retroactivo se materializa en el siguiente repaso; no hay logros
sociales ni de velocidad. El umbral de maestría puede ajustarse entre 50 % y 100 %.

## D-028 — Salvavidas de racha y niveles con títulos

**Contexto:** una racha estricta castiga un día perdido y contradice el carácter pasivo. A la vez,
el XP necesita una progresión legible a largo plazo.
**Decisión:** cada objetivo semanal completado concede un salvavidas (máximo 2). Si el usuario
vuelve tras un día perdido, se gasta automáticamente uno para reparar solo ese hueco (desactivable
en Ajustes) y se registra en `streak_repairs`. La curva de niveles pasa a 12 umbrales de 0 a 5.800
XP, con títulos por tramo: Semilla, Brote, Raíz, Rama, Árbol y Bosque.
**Consecuencias:** la racha sigue siendo flexible y no se castiga con avisos. Los salvavidas no se
gastan sin actividad reciente ni reparan varios huecos de golpe. Las métricas de retención quedan
fuera del MVP por no usar nube ni telemetría.

## D-029 — Tema oscuro, idioma y escala de interfaz

**Contexto:** RF2-07 pide completar la accesibilidad. La app se usa de noche y en sesiones cortas,
y parte del vocabulario objetivo son usuarios que prefieren la interfaz en inglés o japonés.
**Decisión:** añadir tres preferencias locales: `theme` (`auto`, `light`, `dark`), `language`
(`es`, `en`, `ja`) y `font_scale` (90-125 %). `auto` sigue el tema de Windows en vivo. La escala se
aplica con `zoom` compensado en `#root` y las ventanas flotantes crecen proporcionalmente para no
recortar contenido. Toda la UI, la bandeja y las notificaciones usan el idioma elegido; los errores
técnicos y el contenido del mazo se mantienen en su idioma original. La navegación por teclado usa
elementos nativos, `Enter`/`Esc`, `aria-current`, foco visible y un enlace de salto.
**Consecuencias:** las preferencias son locales y sin dependencias externas. El tema oscuro no
introduce animaciones nuevas y respeta `prefers-reduced-motion`. Los textos de logros se traducen
por título en el frontend y por clave en las notificaciones del backend.
