import { useCallback } from "react";

export type Language = "es" | "en" | "ja";

type Vars = Record<string, string | number>;
type Dict = Record<string, string>;

export const LANGUAGES: { id: Language; label: string }[] = [
  { id: "es", label: "Español" },
  { id: "en", label: "English" },
  { id: "ja", label: "日本語" },
];

const en: Dict = {
  // Navigation and status
  Practicar: "Practice",
  Progreso: "Progress",
  Ajustes: "Settings",
  Secciones: "Sections",
  "repaso pasivo": "passive review",
  "En pausa": "Paused",
  Activo: "Active",
  "{n} pendientes · {m} palabras": "{n} due · {m} words",
  "{due} pendientes · {new} nuevas": "{due} due · {new} new",
  "{n} nuevas": "{n} new",
  "{n} nuevas disponibles": "{n} new available",
  "Nuevas hoy: {n} / {m}": "New today: {n} / {m}",
  "{min}–{max} min entre retos": "{min}–{max} min between challenges",
  "Micro-reto": "Micro-challenge",
  "Saltar al contenido": "Skip to content",
  "{n} pendientes": "{n} due",
  "No hay palabras que practicar": "No words to practice",
  "Nada que repasar todavía": "Nothing to review yet",
  "La próxima palabra vence en {interval}.": "Next word is due in {interval}.",
  "Importa un mazo desde Ajustes o espera a que haya ítems disponibles.":
    "Import a deck from Settings or wait until items are available.",
  Reintentar: "Retry",
  Cargando: "Loading",
  "Preparando…": "Preparing…",
  "Siguiente": "Next",
  "Comprobar": "Check",
  "No lo sé": "I don't know",
  "Ahora no": "Not now",
  "Ocultar": "Hide",
  "Cerrar": "Close",
  "Ocultar widget": "Hide widget",
  "Abrir kotoba": "Open kotoba",
  "Abrir app": "Open app",
  "No hay palabras disponibles.": "No words available.",
  "Reproducir audio": "Play audio",
  "Oír": "Listen",
  "Ver lectura": "Show reading",
  "Ocultar lectura": "Hide reading",
  "Escribe el significado…": "Type the meaning…",
  "Kanji, kana o romaji…": "Kanji, kana or romaji…",
  "Escribe el significado en español": "Type the meaning in Spanish",
  "Escribe la palabra en japonés (kanji, kana o romaji)":
    "Type the Japanese word (kanji, kana or romaji)",
  "日本語 → Español": "Japanese → Spanish",
  "Español → 日本語": "Spanish → Japanese",
  "{n} min": "{n} min",
  "{n} h": "{n} h",
  "{n} d": "{n} d",
  "Correcto": "Correct",
  "Casi": "Almost",
  "Bien hecho, sigue así.": "Well done, keep it up.",
  "La respuesta era:": "The answer was:",
  "Japonés": "Japanese",
  "Significado": "Meaning",
  "Kanji": "Kanji",
  "plantilla": "pattern",
  "irregular": "irregular",
  "Vuelve en {interval}": "Back in {interval}",
  "¡Nivel {n}!": "Level {n}!",
  "+1 salvavidas": "+1 saver",
  "+1 salvavidas de racha": "+1 streak saver",
  "correcto": "correct",
  "correcta": "correct",
  "acierto": "correct",
  "aciertos": "correct",
  "repaso": "review",
  "repasos": "reviews",
  "palabra": "word",
  "palabras": "words",
  "día": "day",
  "días": "days",
  "hoy": "today",

  // Import
  "Bienvenido": "Welcome",
  "Trae tu mazo": "Bring your deck",
  "kotoba repasa por ti el vocabulario de tu mazo {deck}. Importa el archivo {format} exportado de Anki y la app extraerá las palabras y el audio.":
    "kotoba reviews the vocabulary from your {deck} deck for you. Import the {format} file exported from Anki and the app will extract words and audio.",
  "Leyendo el mazo…": "Reading the deck…",
  "Extrayendo notas y audio. Puede tardar unos segundos.":
    "Extracting notes and audio. This may take a few seconds.",
  "Elegir archivo .apkg": "Choose .apkg file",
  "También acepta exports .txt de Anki (con su carpeta de audio)":
    "Also accepts Anki .txt exports (with their audio folder)",
  "Todo se guarda en tu equipo. Nada sale de aquí.":
    "Everything stays on your device. Nothing leaves it.",
  "El progreso de Anki no se toca: kotoba usa su propio repaso.":
    "Anki progress is untouched: kotoba uses its own review.",
  "Puedes reimportar o borrar los datos desde Ajustes.":
    "You can reimport or delete data from Settings.",
  "Mazo importado: {name}. Esto es lo que ha quedado listo para repasar.":
    "Deck imported: {name}. This is ready for review.",
  "audios": "audio files",
  "plantillas": "patterns",
  "notas leídas": "notes read",
  "{n} notas duplicadas se han unido a su palabra principal.":
    "{n} duplicate notes were merged into their main word.",
  "{n} audios no encontrados.": "{n} audio files not found.",
  "{n} avisos durante la importación": "{n} warnings during import",
  "Importar otro": "Import another",
  "Empezar a repasar": "Start reviewing",
  "Mazo de Anki": "Anki deck",
  "Todos": "All",
  "Mapear campos": "Map fields",
  "Asigna cada dato de kotoba a una columna del mazo. Lectura y significado son obligatorios.":
    "Map each kotoba field to a deck column. Reading and meaning are required.",
  Lectura: "Reading",
  "Glosa de kanji": "Kanji gloss",
  "Audio": "Audio",
  "Etiquetas": "Tags",
  "Lección": "Lesson",
  "Subapartado": "Subsection",
  "Sin asignar": "Not assigned",
  "Importar con este mapeo": "Import with this mapping",
  "Importando…": "Importing…",
  Cancelar: "Cancel",
  "Filas de ejemplo": "Sample rows",
  "Tipo de nota": "Note type",
  "notas": "notes",
  "columnas": "columns",
  "Selecciona lectura y significado para continuar.":
    "Select reading and meaning to continue.",
  "Listo para importar.": "Ready to import.",
  "Carpeta de medios": "Media folder",
  "Carpeta hermana con el nombre del .txt":
    "Sibling folder named like the .txt",
  "Elegir carpeta…": "Choose folder…",
  "Quitar": "Clear",
  "Los audios de [sound: …] se copian desde esta carpeta. Si está vacía, se usa la carpeta hermana con el nombre del .txt.":
    "Audio from [sound: …] is copied from this folder. If empty, the sibling folder named like the .txt is used.",

  // Progress
  "Tu progreso": "Your progress",
  "Sin colas ni deberes: solo lo que has ido reteniendo sin darte cuenta.":
    "No queues or homework: just what you have retained without noticing.",
  "Actualizar": "Refresh",
  "Precisión hoy": "Accuracy today",
  "Racha": "Streak",
  "Dominadas": "Mastered",
  "Pendientes": "Due",
  "{correct} de {total} repasos": "{correct} of {total} reviews",
  "día seguido": "day in a row",
  "días seguidos": "days in a row",
  "· mejor {n}": "· best {n}",
  "· {n} salvavidas": "· {n} savers",
  "mejor {n}": "best {n}",
  "{n} salvavidas": "{n} savers",
  "1 salvavidas": "1 saver",
  "con intervalo de 7 días o más": "with a 7-day interval or more",
  "{n} palabras en total": "{n} words in total",
  "Nivel {n} · {title}": "Level {n} · {title}",
  "{n} XP para el siguiente nivel": "{n} XP to the next level",
  "Nivel máximo": "Max level",
  "Objetivo semanal": "Weekly goal",
  "{n} / {m} repasos": "{n} / {m} reviews",
  "Completado": "Completed",
  "Sin objetivo: repasa a tu ritmo.": "No goal: review at your own pace.",
  "Objetivo cumplido. Sigue así.": "Goal met. Keep it up.",
  "Cada repaso cuenta, también los fallos.":
    "Every review counts, mistakes too.",
  "{n} día reparado con salvavidas.": "{n} day repaired with a saver.",
  "{n} días reparados con salvavidas.": "{n} days repaired with savers.",
  "Cada objetivo completado da un salvavidas (máx. {n}).":
    "Each completed goal grants a saver (max {n}).",
  "Logros": "Achievements",
  "{n} de {m} desbloqueados. Los logros no caducan.":
    "{n} of {m} unlocked. Achievements never expire.",
  "Desbloqueado": "Unlocked",
  "Últimos 14 días": "Last 14 days",
  "Barra clara: repasos · barra oscura: aciertos":
    "Light bar: reviews · dark bar: correct",
  "{n} repasos en total": "{n} reviews in total",
  "{date}: {total} repasos, {correct} aciertos":
    "{date}: {total} reviews, {correct} correct",
  "Por lección": "By lesson",
  "Progreso y precisión de cada capítulo. Una lección se domina con al menos el {n}% de sus palabras en 7 días o más.":
    "Progress and accuracy for each chapter. A lesson is mastered with at least {n}% of its words at 7 days or more.",
  "Dominada": "Mastered",
  "Maestría: {n}%": "Mastery: {n}%",
  "Precisión": "Accuracy",
  "Palabras difíciles": "Difficult words",
  "Las que más fallos acumulan.": "The ones with the most mistakes.",
  "Aún no hay fallos registrados. Sigue repasando.":
    "No mistakes recorded yet. Keep reviewing.",
  "{n} fallo": "{n} mistake",
  "{n} fallos": "{n} mistakes",
  "Un micro-reto de 10 segundos vale más que una sesión que nunca empiezas.":
    "A 10-second micro-challenge beats a session you never start.",
  "Practicar ahora": "Practice now",

  // Levels
  Semilla: "Seed",
  Brote: "Sprout",
  Raíz: "Root",
  Rama: "Branch",
  Árbol: "Tree",
  Bosque: "Forest",

  // Settings
  "Tu ritmo": "Your pace",
  "Un equilibrio entre no olvidar y no molestar. Los cambios se aplican al guardar.":
    "A balance between not forgetting and not interrupting. Changes apply on save.",
  Frecuencia: "Frequency",
  "Cuándo y cuánto aparece el repaso.": "When and how often review appears.",
  "Intervalo entre retos": "Interval between challenges",
  "Cada cuánto puede aparecer un micro-reto.":
    "How often a micro-challenge can appear.",
  "mín": "min",
  "máx": "max",
  "Máximo por día": "Daily limit",
  "Sin límite diario.": "No daily limit.",
  "Hasta {n} repasos al día.": "Up to {n} reviews a day.",
  "Pausar repasos": "Pause reviews",
  "La app sigue en la bandeja, pero no interrumpe.":
    "The app stays in the tray but does not interrupt.",
  "No molestar": "Do not disturb",
  "Horas en las que kotoba se queda callado.":
    "Hours when kotoba stays quiet.",
  "Activar horario de silencio": "Enable quiet hours",
  "Útil para la noche o para reuniones.": "Useful at night or in meetings.",
  "Desde / hasta": "From / to",
  "Desde": "From",
  "Hasta": "To",
  "Si la hora final es menor, el rango cruza la medianoche.":
    "If the end time is earlier, the range crosses midnight.",
  "Nivel, racha, logros y objetivos.": "Level, streak, achievements and goals.",
  "Repasos por semana. 0 = sin objetivo.": "Reviews per week. 0 = no goal.",
  "Nuevas por día": "New words per day",
  "Palabras nuevas que se introducen cada día, en orden de lección y subapartado. 0 = ninguna.":
    "New words introduced each day, in lesson and subsection order. 0 = none.",
  "Mezcla de nuevas": "New word mix",
  "0 = primero los repasos vencidos. Con repasos pendientes, este porcentaje decide cuántas nuevas se muestran.":
    "0 = due reviews first. When reviews are pending, this percentage decides how many new words slip in.",
  "Maestría por lección": "Lesson mastery",
  "Porcentaje de palabras con intervalo de 7 días o más para dar una lección por dominada.":
    "Percentage of words with a 7-day interval or more to consider a lesson mastered.",
  "Reparar racha": "Repair streak",
  "Si fallas un día, gastar un salvavidas para mantener la racha. Los salvavidas se ganan al cumplir el objetivo semanal.":
    "If you miss a day, spend a saver to keep the streak. Savers are earned by completing the weekly goal.",
  "Notificaciones de hitos": "Milestone notifications",
  "Avisar al completar el objetivo semanal o desbloquear un logro. Respeta el modo no molestar y la pantalla completa.":
    "Notify when the weekly goal is completed or an achievement is unlocked. Respects do not disturb and fullscreen.",
  "Aparición": "Appearance",
  "Cómo y dónde aparece el micro-reto.": "How and where the micro-challenge appears.",
  "Modo": "Mode",
  "Popup: tarjeta flotante. Widget: ventana siempre visible. Toast: notificación nativa de Windows.":
    "Popup: floating card. Widget: always-visible window. Toast: native Windows notification.",
  "Modo de aparición": "Appearance mode",
  "Silenciar en pantalla completa": "Silence in fullscreen",
  "No interrumpir juegos, vídeos, presentaciones ni el modo silencio de Windows.":
    "Do not interrupt games, videos, presentations or Windows quiet mode.",
  "El toast es una notificación nativa: toca la notificación o el icono de la bandeja para abrir el reto.":
    "Toast is a native notification: tap it or the tray icon to open the challenge.",
  "El widget aparece en la esquina inferior derecha. Puedes arrastrarlo y ocultarlo cuando quieras.":
    "The widget appears in the bottom-right corner. Drag it or hide it whenever you want.",
  Contenido: "Content",
  "Qué se pregunta y cómo.": "What is asked and how.",
  "Dirección del ejercicio": "Exercise direction",
  "JP → ES para reconocer; ES → JP para producir. Alternar mezcla ambas.":
    "JP → ES to recognize; ES → JP to produce. Alternate mixes both.",
  "Dirección": "Direction",
  Aleatoria: "Random",
  Alternar: "Alternate",
  "Incluir plantillas y gramática": "Include patterns and grammar",
  "Notas con 〜, «+ negative» o «...». Recomendado dejarlas fuera al principio.":
    "Notes with 〜, “+ negative” or “...”. Best left out at first.",
  "Incluir plantillas": "Include patterns",
  "Audio automático en JP → ES": "Automatic audio in JP → ES",
  "En ES → JP el audio suena solo después de responder.":
    "In ES → JP audio only plays after answering.",
  "Audio automático": "Automatic audio",
  Sistema: "System",
  "Integración con Windows.": "Windows integration.",
  Actualizaciones: "Updates",
  "Comprueba si hay una versión nueva de kotoba.":
    "Check whether a new kotoba version is available.",
  "Buscar actualizaciones": "Check for updates",
  "Comprobando…": "Checking…",
  "Ya tienes la última versión.": "You're on the latest version.",
  "Actualización disponible": "Update available",
  "kotoba {version} está disponible. ¿Actualizar ahora?":
    "kotoba {version} is available. Update now?",
  "Actualizar ahora": "Update now",
  "Descargando la actualización…": "Downloading the update…",
  "Arrancar con Windows": "Start with Windows",
  "kotoba se inicia en la bandeja sin abrir ninguna ventana.":
    "kotoba starts in the tray without opening any window.",
  "No disponible en la compilación de desarrollo.":
    "Not available in development builds.",
  "Registro de la app": "App log",
  "Abre la carpeta con los logs para diagnosticar problemas.":
    "Open the logs folder to diagnose problems.",
  "Abrir carpeta": "Open folder",
  Datos: "Data",
  "Todo es local. Puedes reimportar el mazo sin perder el progreso de las palabras que ya existen.":
    "Everything is local. You can reimport the deck without losing progress on existing words.",
  "Reimportar mazo": "Reimport deck",
  "Exportar copia": "Export backup",
  "Restaurar copia": "Restore backup",
  "Exportar historial": "Export history",
  "Reiniciar progreso": "Reset progress",
  "Borrar biblioteca": "Delete library",
  "Hay cambios sin guardar.": "You have unsaved changes.",
  "Todo guardado.": "All saved.",
  Descartar: "Discard",
  "Guardar cambios": "Save changes",
  "Guardando…": "Saving…",
  "Ajustes guardados": "Settings saved",
  "Importadas {words} palabras y {audios} audios":
    "Imported {words} words and {audios} audio files",
  "Copia guardada en {folder}": "Backup saved to {folder}",
  "Copia restaurada": "Backup restored",
  "Historial guardado en {path}": "History saved to {path}",
  "Progreso reiniciado": "Progress reset",
  "Biblioteca borrada": "Library deleted",
  "Se reemplazarán las palabras, el audio y el progreso actuales. ¿Continuar?":
    "Current words, audio and progress will be replaced. Continue?",
  "¿Reiniciar el repaso? Se borra el historial y los intervalos, pero se conservan las palabras.":
    "Reset review? History and intervals are deleted, but words are kept.",
  "¿Borrar todas las palabras importadas? Tendrás que volver a importar el mazo.":
    "Delete all imported words? You will need to import the deck again.",
  "Hay cambios sin guardar. ¿Descartarlos?":
    "You have unsaved changes. Discard them?",

  // Appearance
  Apariencia: "Appearance",
  "Tema, idioma y tamaño de la interfaz.":
    "Theme, language and interface size.",
  Tema: "Theme",
  Auto: "Auto",
  Claro: "Light",
  Oscuro: "Dark",
  "Tamaño de la interfaz": "Interface size",
  Idioma: "Language",
  "Sigue el tema de Windows.": "Follows the Windows theme.",
  "Tema claro, papel y tinta.": "Light theme, paper and ink.",
  "Tema oscuro, lectura nocturna.": "Dark theme for night reading.",
  "Cambia los textos de la interfaz y de la bandeja.":
    "Changes the interface and tray texts.",
  "Escala el texto y los controles de la interfaz.":
    "Scales interface text and controls.",

  // Achievements
  "Primer paso": "First step",
  "Completa tu primer repaso.": "Complete your first review.",
  Calentando: "Warming up",
  "Completa 25 repasos.": "Complete 25 reviews.",
  "Cien repasos": "One hundred reviews",
  "Completa 100 repasos.": "Complete 100 reviews.",
  Constancia: "Consistency",
  "Completa 250 repasos.": "Complete 250 reviews.",
  "Medio millar": "Half a thousand",
  "Completa 500 repasos.": "Complete 500 reviews.",
  "Primera semilla": "First seed",
  "Consigue una palabra madura con un intervalo de 21 días o más.":
    "Reach one mature word with an interval of 21 days or more.",
  "Diez semillas": "Ten seeds",
  "Consigue 10 palabras maduras.": "Reach 10 mature words.",
  "Veinticinco semillas": "Twenty-five seeds",
  "Consigue 25 palabras maduras.": "Reach 25 mature words.",
  "Cincuenta semillas": "Fifty seeds",
  "Consigue 50 palabras maduras.": "Reach 50 mature words.",
  "Tres días": "Three days",
  "Repasa 3 días seguidos.": "Review 3 days in a row.",
  "Una semana": "One week",
  "Repasa 7 días seguidos.": "Review 7 days in a row.",
  "Un mes": "One month",
  "Repasa 30 días seguidos.": "Review 30 days in a row.",
  "Lección dominada": "Lesson mastered",
  "Alcanza el umbral de maestría en una lección.":
    "Reach the mastery threshold in one lesson.",
  "Cinco lecciones": "Five lessons",
  "Alcanza el umbral de maestría en cinco lecciones.":
    "Reach the mastery threshold in five lessons.",
  "Nivel 5": "Level 5",
  "Alcanza el nivel 5.": "Reach level 5.",
  "Objetivo cumplido": "Goal completed",
  "Completa un objetivo semanal.": "Complete a weekly goal.",

  // History
  Historial: "History",
  "Tus últimos repasos, del más reciente al más antiguo.":
    "Your latest reviews, newest first.",
  Resultado: "Result",
  Acierto: "Hit",
  Fallo: "Miss",
  Fecha: "Date",
  Latencia: "Latency",
  Calidad: "Quality",
  "Cargar más": "Load more",
  "Aún no hay repasos registrados. Practica para empezar el historial.":
    "No reviews yet. Practice to start your history.",
  "No hay repasos con estos filtros.": "No reviews match these filters.",
  "Limpiar filtros": "Clear filters",
  "{n} de {m} repasos": "{n} of {m} reviews",
};

const ja: Dict = {
  Practicar: "練習",
  Progreso: "進捗",
  Ajustes: "設定",
  Secciones: "セクション",
  "repaso pasivo": "受動的復習",
  "En pausa": "一時停止中",
  Activo: "稼働中",
  "{n} pendientes · {m} palabras": "残り{n}語 · 全{m}語",
  "{due} pendientes · {new} nuevas": "残り{due}語 · 新規{new}語",
  "{n} nuevas": "新規{n}語",
  "{n} nuevas disponibles": "新規{n}語が利用可能",
  "Nuevas hoy: {n} / {m}": "今日の新規：{n} / {m}",
  "{min}–{max} min entre retos": "チャレンジ間隔 {min}–{max}分",
  "Micro-reto": "ミニチャレンジ",
  "Saltar al contenido": "コンテンツへスキップ",
  "{n} pendientes": "残り{n}語",
  "No hay palabras que practicar": "練習する単語がありません",
  "Nada que repasar todavía": "まだ復習する単語がありません",
  "La próxima palabra vence en {interval}.": "次の単語は{interval}後に出題されます。",
  "Importa un mazo desde Ajustes o espera a que haya ítems disponibles.":
    "設定からデッキをインポートするか、単語が利用可能になるまでお待ちください。",
  Reintentar: "再試行",
  Cargando: "読み込み中",
  "Preparando…": "準備中…",
  Siguiente: "次へ",
  Comprobar: "確認",
  "No lo sé": "わかりません",
  "Ahora no": "後で",
  Ocultar: "隠す",
  Cerrar: "閉じる",
  "Ocultar widget": "ウィジェットを隠す",
  "Abrir kotoba": "kotoba を開く",
  "Abrir app": "アプリを開く",
  "No hay palabras disponibles.": "利用できる単語がありません。",
  "Reproducir audio": "音声を再生",
  "Oír": "聞く",
  "Ver lectura": "読み方を見る",
  "Ocultar lectura": "読み方を隠す",
  "Escribe el significado…": "意味を入力…",
  "Kanji, kana o romaji…": "漢字・かな・ローマ字…",
  "Escribe el significado en español": "意味をスペイン語で入力",
  "Escribe la palabra en japonés (kanji, kana o romaji)":
    "日本語を入力（漢字・かな・ローマ字）",
  "日本語 → Español": "日本語 → スペイン語",
  "Español → 日本語": "スペイン語 → 日本語",
  "{n} min": "{n}分",
  "{n} h": "{n}時間",
  "{n} d": "{n}日",
  Correcto: "正解",
  Casi: "おしい",
  "Bien hecho, sigue así.": "よくできました。その調子。",
  "La respuesta era:": "正解は：",
  Japonés: "日本語",
  Significado: "意味",
  Kanji: "漢字",
  plantilla: "パターン",
  irregular: "不規則",
  "Vuelve en {interval}": "{interval}後に再表示",
  "¡Nivel {n}!": "レベル{n}！",
  "+1 salvavidas": "保護+1",
  "+1 salvavidas de racha": "連続記録の保護+1",
  correcto: "正解",
  correcta: "正解",
  acierto: "正解",
  aciertos: "正解",
  repaso: "復習",
  repasos: "復習",
  palabra: "単語",
  palabras: "単語",
  día: "日",
  días: "日",
  hoy: "今日",

  Bienvenido: "ようこそ",
  "Trae tu mazo": "デッキを読み込む",
  "kotoba repasa por ti el vocabulario de tu mazo {deck}. Importa el archivo {format} exportado de Anki y la app extraerá las palabras y el audio.":
    "kotoba が {deck} デッキの単語を復習します。Anki から書き出した {format} ファイルを読み込むと、単語と音声を抽出します。",
  "Leyendo el mazo…": "デッキを読み込み中…",
  "Extrayendo notas y audio. Puede tardar unos segundos.":
    "ノートと音声を抽出しています。数秒かかる場合があります。",
  "Elegir archivo .apkg": ".apkg ファイルを選ぶ",
  "También acepta exports .txt de Anki (con su carpeta de audio)":
    "Anki の .txt 書き出しにも対応（音声フォルダー込み）",
  "Todo se guarda en tu equipo. Nada sale de aquí.":
    "すべてお使いの端末に保存されます。外部には送信されません。",
  "El progreso de Anki no se toca: kotoba usa su propio repaso.":
    "Anki の進捗は変更しません。kotoba は独自の復習を使います。",
  "Puedes reimportar o borrar los datos desde Ajustes.":
    "設定から再インポートやデータ削除ができます。",
  "Mazo importado: {name}. Esto es lo que ha quedado listo para repasar.":
    "デッキ「{name}」を読み込みました。復習の準備ができた内容です。",
  audios: "音声",
  plantillas: "パターン",
  "notas leídas": "読み込んだノート",
  "{n} notas duplicadas se han unido a su palabra principal.":
    "{n} 件の重複ノートを主な単語にまとめました。",
  "{n} audios no encontrados.": "{n} 件の音声が見つかりませんでした。",
  "{n} avisos durante la importación": "インポート中の警告 {n} 件",
  "Importar otro": "別のデッキを読み込む",
  "Empezar a repasar": "復習を始める",
  "Mazo de Anki": "Anki デッキ",
  Todos: "すべて",
  "Mapear campos": "フィールドの対応",
  "Asigna cada dato de kotoba a una columna del mazo. Lectura y significado son obligatorios.":
    "kotoba の各項目をデッキの列に対応させます。読みと意味は必須です。",
  Lectura: "読み",
  "Glosa de kanji": "漢字の意味",
  Audio: "音声",
  Etiquetas: "タグ",
  Lección: "レッスン",
  Subapartado: "サブセクション",
  "Sin asignar": "未割り当て",
  "Importar con este mapeo": "この対応でインポート",
  "Importando…": "インポート中…",
  Cancelar: "キャンセル",
  "Filas de ejemplo": "サンプル行",
  "Tipo de nota": "ノートタイプ",
  notas: "ノート",
  columnas: "列",
  "Selecciona lectura y significado para continuar.":
    "続行するには読みと意味を選択してください。",
  "Listo para importar.": "インポートの準備ができました。",
  "Carpeta de medios": "メディアフォルダー",
  "Carpeta hermana con el nombre del .txt":
    ".txt と同じ名前の隣接フォルダー",
  "Elegir carpeta…": "フォルダーを選択…",
  "Quitar": "クリア",
  "Los audios de [sound: …] se copian desde esta carpeta. Si está vacía, se usa la carpeta hermana con el nombre del .txt.":
    "[sound: …] の音声はこのフォルダーからコピーします。空の場合は .txt と同じ名前の隣接フォルダーを使います。",

  "Tu progreso": "あなたの進捗",
  "Sin colas ni deberes: solo lo que has ido reteniendo sin darte cuenta.":
    "順番待ちも宿題もなし。気づかないうちに身についた分だけ。",
  Actualizar: "更新",
  "Precisión hoy": "今日の正解率",
  Racha: "連続記録",
  Dominadas: "習得済み",
  Pendientes: "残り",
  "{correct} de {total} repasos": "{total}回中{correct}回",
  "día seguido": "日連続",
  "días seguidos": "日連続",
  "· mejor {n}": "・最高{n}",
  "· {n} salvavidas": "・保護{n}",
  "mejor {n}": "最高{n}",
  "{n} salvavidas": "保護{n}",
  "1 salvavidas": "保護1",
  "con intervalo de 7 días o más": "間隔7日以上",
  "{n} palabras en total": "全{n}語",
  "Nivel {n} · {title}": "レベル{n}・{title}",
  "{n} XP para el siguiente nivel": "次のレベルまで{n} XP",
  "Nivel máximo": "最大レベル",
  "Objetivo semanal": "週間目標",
  "{n} / {m} repasos": "{n} / {m}回",
  Completado: "達成",
  "Sin objetivo: repasa a tu ritmo.": "目標なし：自分のペースで。",
  "Objetivo cumplido. Sigue así.": "目標達成。その調子。",
  "Cada repaso cuenta, también los fallos.": "間違いも含めて、すべての復習が力になります。",
  "{n} día reparado con salvavidas.": "保護で{n}日を修復しました。",
  "{n} días reparados con salvavidas.": "保護で{n}日を修復しました。",
  "Cada objetivo completado da un salvavidas (máx. {n}).":
    "目標を達成するごとに保護を1つ獲得（最大{n}）。",
  Logros: "実績",
  "{n} de {m} desbloqueados. Los logros no caducan.":
    "{m}個中{n}個解除済み。実績は失われません。",
  Desbloqueado: "解除日",
  "Últimos 14 días": "直近14日",
  "Barra clara: repasos · barra oscura: aciertos":
    "明るいバー：復習 ・ 濃いバー：正解",
  "{n} repasos en total": "合計{n}回",
  "{date}: {total} repasos, {correct} aciertos":
    "{date}：復習{total}回、正解{correct}回",
  "Por lección": "レッスン別",
  "Progreso y precisión de cada capítulo. Una lección se domina con al menos el {n}% de sus palabras en 7 días o más.":
    "各章の進捗と正解率。単語の{n}%以上が間隔7日以上になるとレッスン制覇です。",
  Dominada: "制覇",
  "Maestría: {n}%": "習得率：{n}%",
  Precisión: "正解率",
  "Palabras difíciles": "苦手な単語",
  "Las que más fallos acumulan.": "間違いが多い単語。",
  "Aún no hay fallos registrados. Sigue repasando.":
    "まだ間違いの記録がありません。復習を続けましょう。",
  "{n} fallo": "間違い{n}回",
  "{n} fallos": "間違い{n}回",
  "Un micro-reto de 10 segundos vale más que una sesión que nunca empiezas.":
    "10秒のミニチャレンジは、始めない勉強より価値があります。",
  "Practicar ahora": "今すぐ練習",

  Semilla: "種",
  Brote: "芽",
  Raíz: "根",
  Rama: "枝",
  Árbol: "木",
  Bosque: "森",

  "Tu ritmo": "あなたのペース",
  "Un equilibrio entre no olvidar y no molestar. Los cambios se aplican al guardar.":
    "忘れないことと邪魔しないことのバランス。変更は保存時に反映されます。",
  Frecuencia: "頻度",
  "Cuándo y cuánto aparece el repaso.": "復習の表示タイミングと回数。",
  "Intervalo entre retos": "チャレンジ間隔",
  "Cada cuánto puede aparecer un micro-reto.": "ミニチャレンジの表示間隔。",
  "mín": "最小",
  "máx": "最大",
  "Máximo por día": "1日の上限",
  "Sin límite diario.": "1日の上限なし。",
  "Hasta {n} repasos al día.": "1日最大{n}回。",
  "Pausar repasos": "復習を一時停止",
  "La app sigue en la bandeja, pero no interrumpe.":
    "アプリは常駐したまま、表示を止めます。",
  "No molestar": "おやすみモード",
  "Horas en las que kotoba se queda callado.": "kotoba が静かにする時間帯。",
  "Activar horario de silencio": "静かな時間を有効化",
  "Útil para la noche o para reuniones.": "夜間や会議に便利です。",
  "Desde / hasta": "開始 / 終了",
  "Desde": "開始",
  "Hasta": "終了",
  "Si la hora final es menor, el rango cruza la medianoche.":
    "終了時刻が開始より早い場合は日をまたぎます。",
  "Nivel, racha, logros y objetivos.": "レベル、連続記録、実績、目標。",
  "Repasos por semana. 0 = sin objetivo.": "週あたりの復習回数。0 = 目標なし。",
  "Nuevas por día": "1日の新しい単語",
  "Palabras nuevas que se introducen cada día, en orden de lección y subapartado. 0 = ninguna.":
    "1日に導入する新しい単語。レッスンとサブセクションの順に進みます。0 = なし。",
  "Mezcla de nuevas": "新しい単語の混ぜ方",
  "0 = primero los repasos vencidos. Con repasos pendientes, este porcentaje decide cuántas nuevas se muestran.":
    "0 = 期限切れの復習を優先。復習が残っている場合、この割合で新しい単語を混ぜます。",
  "Maestría por lección": "レッスン習得",
  "Porcentaje de palabras con intervalo de 7 días o más para dar una lección por dominada.":
    "レッスン制覇とみなす、間隔7日以上の単語の割合。",
  "Reparar racha": "連続記録を修復",
  "Si fallas un día, gastar un salvavidas para mantener la racha. Los salvavidas se ganan al cumplir el objetivo semanal.":
    "1日空いたら保護を1つ使って連続記録を維持します。保護は週間目標の達成で獲得します。",
  "Notificaciones de hitos": "節目の通知",
  "Avisar al completar el objetivo semanal o desbloquear un logro. Respeta el modo no molestar y la pantalla completa.":
    "週間目標の達成や実績解除を通知します。おやすみモードと全画面表示を尊重します。",
  Aparición: "表示",
  "Cómo y dónde aparece el micro-reto.": "ミニチャレンジの表示方法と場所。",
  Modo: "モード",
  "Popup: tarjeta flotante. Widget: ventana siempre visible. Toast: notificación nativa de Windows.":
    "Popup：フローティングカード。Widget：常時表示ウィンドウ。Toast：Windows 標準通知。",
  "Modo de aparición": "表示モード",
  "Silenciar en pantalla completa": "全画面表示中は静かに",
  "No interrumpir juegos, vídeos, presentaciones ni el modo silencio de Windows.":
    "ゲーム、動画、プレゼン、Windows のおやすみモードを妨げません。",
  "El toast es una notificación nativa: toca la notificación o el icono de la bandeja para abrir el reto.":
    "Toast は Windows 通知です。通知またはトレイアイコンをタップして開きます。",
  "El widget aparece en la esquina inferior derecha. Puedes arrastrarlo y ocultarlo cuando quieras.":
    "ウィジェットは右下に表示されます。ドラッグや非表示が可能です。",
  Contenido: "コンテンツ",
  "Qué se pregunta y cómo.": "何をどう出題するか。",
  "Dirección del ejercicio": "出題方向",
  "JP → ES para reconocer; ES → JP para producir. Alternar mezcla ambas.":
    "JP → ES は認識、ES → JP は産出。交互にすると両方を混ぜます。",
  Dirección: "方向",
  Aleatoria: "ランダム",
  Alternar: "交互",
  "Incluir plantillas y gramática": "パターンと文法を含める",
  "Notas con 〜, «+ negative» o «...». Recomendado dejarlas fuera al principio.":
    "〜、「+ negative」、「...」を含むノート。最初は除外がおすすめです。",
  "Incluir plantillas": "パターンを含める",
  "Audio automático en JP → ES": "JP → ES で音声を自動再生",
  "En ES → JP el audio suena solo después de responder.":
    "ES → JP では回答後に音声を再生します。",
  "Audio automático": "音声の自動再生",
  Sistema: "システム",
  "Integración con Windows.": "Windows との連携。",
  Actualizaciones: "アップデート",
  "Comprueba si hay una versión nueva de kotoba.":
    "kotoba の新しいバージョンがあるか確認します。",
  "Buscar actualizaciones": "アップデートを確認",
  "Comprobando…": "確認中…",
  "Ya tienes la última versión.": "最新バージョンです。",
  "Actualización disponible": "アップデートがあります",
  "kotoba {version} está disponible. ¿Actualizar ahora?":
    "kotoba {version} が利用可能です。今すぐ更新しますか？",
  "Actualizar ahora": "今すぐ更新",
  "Descargando la actualización…": "アップデートをダウンロードしています…",
  "Arrancar con Windows": "Windows と同時に起動",
  "kotoba se inicia en la bandeja sin abrir ninguna ventana.":
    "ウィンドウを開かずトレイで起動します。",
  "No disponible en la compilación de desarrollo.":
    "開発ビルドでは利用できません。",
  "Registro de la app": "アプリのログ",
  "Abre la carpeta con los logs para diagnosticar problemas.":
    "問題の診断用にログフォルダーを開きます。",
  "Abrir carpeta": "フォルダーを開く",
  Datos: "データ",
  "Todo es local. Puedes reimportar el mazo sin perder el progreso de las palabras que ya existen.":
    "すべてローカルです。既存の単語の進捗を保ったまま再インポートできます。",
  "Reimportar mazo": "デッキを再インポート",
  "Exportar copia": "バックアップを書き出す",
  "Restaurar copia": "バックアップを復元",
  "Exportar historial": "履歴を書き出す",
  "Reiniciar progreso": "進捗をリセット",
  "Borrar biblioteca": "ライブラリを削除",
  "Hay cambios sin guardar.": "未保存の変更があります。",
  "Todo guardado.": "すべて保存済み。",
  Descartar: "破棄",
  "Guardar cambios": "変更を保存",
  "Guardando…": "保存中…",
  "Ajustes guardados": "設定を保存しました",
  "Importadas {words} palabras y {audios} audios":
    "{words}語と音声{audios}件を読み込みました",
  "Copia guardada en {folder}": "バックアップを保存しました：{folder}",
  "Copia restaurada": "バックアップを復元しました",
  "Historial guardado en {path}": "履歴を保存しました：{path}",
  "Progreso reiniciado": "進捗をリセットしました",
  "Biblioteca borrada": "ライブラリを削除しました",
  "Se reemplazarán las palabras, el audio y el progreso actuales. ¿Continuar?":
    "現在の単語、音声、進捗を置き換えます。続行しますか？",
  "¿Reiniciar el repaso? Se borra el historial y los intervalos, pero se conservan las palabras.":
    "復習をリセットしますか？履歴と間隔は削除されますが、単語は残ります。",
  "¿Borrar todas las palabras importadas? Tendrás que volver a importar el mazo.":
    "読み込んだ単語をすべて削除しますか？デッキの再インポートが必要になります。",
  "Hay cambios sin guardar. ¿Descartarlos?":
    "未保存の変更があります。破棄しますか？",

  Apariencia: "外観",
  "Tema, idioma y tamaño de la interfaz.": "テーマ、言語、表示サイズ。",
  Tema: "テーマ",
  Auto: "自動",
  Claro: "ライト",
  Oscuro: "ダーク",
  "Tamaño de la interfaz": "表示サイズ",
  Idioma: "言語",
  "Sigue el tema de Windows.": "Windows のテーマに合わせます。",
  "Tema claro, papel y tinta.": "紙とインクのライトテーマ。",
  "Tema oscuro, lectura nocturna.": "夜間向けのダークテーマ。",
  "Cambia los textos de la interfaz y de la bandeja.":
    "画面とトレイの表示言語を変更します。",
  "Escala el texto y los controles de la interfaz.":
    "画面の文字とコントロールの大きさを変更します。",

  "Primer paso": "最初の一歩",
  "Completa tu primer repaso.": "最初の復習を完了する。",
  Calentando: "ウォームアップ",
  "Completa 25 repasos.": "25回復習する。",
  "Cien repasos": "100回の復習",
  "Completa 100 repasos.": "100回復習する。",
  Constancia: "継続は力",
  "Completa 250 repasos.": "250回復習する。",
  "Medio millar": "500回の復習",
  "Completa 500 repasos.": "500回復習する。",
  "Primera semilla": "最初の種",
  "Consigue una palabra madura con un intervalo de 21 días o más.":
    "間隔21日以上の成熟した単語を1つにする。",
  "Diez semillas": "10の種",
  "Consigue 10 palabras maduras.": "成熟した単語を10語にする。",
  "Veinticinco semillas": "25の種",
  "Consigue 25 palabras maduras.": "成熟した単語を25語にする。",
  "Cincuenta semillas": "50の種",
  "Consigue 50 palabras maduras.": "成熟した単語を50語にする。",
  "Tres días": "3日連続",
  "Repasa 3 días seguidos.": "3日連続で復習する。",
  "Una semana": "1週間",
  "Repasa 7 días seguidos.": "7日連続で復習する。",
  "Un mes": "1か月",
  "Repasa 30 días seguidos.": "30日連続で復習する。",
  "Lección dominada": "レッスン制覇",
  "Alcanza el umbral de maestría en una lección.": "1つのレッスンで習得基準に達する。",
  "Cinco lecciones": "5レッスン制覇",
  "Alcanza el umbral de maestría en cinco lecciones.":
    "5つのレッスンで習得基準に達する。",
  "Nivel 5": "レベル5",
  "Alcanza el nivel 5.": "レベル5に到達する。",
  "Objetivo cumplido": "目標達成",
  "Completa un objetivo semanal.": "週間目標を達成する。",

  // History
  Historial: "履歴",
  "Tus últimos repasos, del más reciente al más antiguo.":
    "最近の復習を新しい順に表示します。",
  Resultado: "結果",
  Acierto: "正解",
  Fallo: "不正解",
  Fecha: "日時",
  Latencia: "応答時間",
  Calidad: "品質",
  "Cargar más": "さらに読み込む",
  "Aún no hay repasos registrados. Practica para empezar el historial.":
    "まだ復習の記録がありません。練習すると履歴が始まります。",
  "No hay repasos con estos filtros.": "この条件に合う復習はありません。",
  "Limpiar filtros": "フィルターをクリア",
  "{n} de {m} repasos": "{m}件中{n}件",
};

function interpolate(text: string, vars?: Vars): string {
  if (!vars) return text;
  return text.replace(/\{(\w+)\}/g, (match, key: string) =>
    key in vars ? String(vars[key]) : match,
  );
}

export function translate(
  language: Language,
  key: string,
  vars?: Vars,
): string {
  const dict = language === "en" ? en : language === "ja" ? ja : null;
  return interpolate(dict?.[key] ?? key, vars);
}

export type TFunc = (key: string, vars?: Vars) => string;

export function useT(language: Language): TFunc {
  return useCallback<TFunc>(
    (key, vars) => translate(language, key, vars),
    [language],
  );
}
