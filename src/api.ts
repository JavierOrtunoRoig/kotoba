import { invoke } from "@tauri-apps/api/core";
import { translate, type Language } from "./i18n";

export type Direction = "jp_to_es" | "es_to_jp";
export type DirectionMode = "random" | "alternate";
export type AppearanceMode = "popup" | "widget" | "toast";

export interface Settings {
  interval_min_minutes: number;
  interval_max_minutes: number;
  max_per_day: number;
  paused: boolean;
  dnd_enabled: boolean;
  dnd_start: string;
  dnd_end: string;
  audio_autoplay: boolean;
  mode: AppearanceMode;
  suppress_fullscreen: boolean;
  direction_mode: DirectionMode;
  include_patterns: boolean;
  launch_at_startup: boolean;
  weekly_goal: number;
  milestone_notifications: boolean;
  lesson_mastery_percent: number;
  streak_repair: boolean;
  theme: Theme;
  font_scale: number;
  language: Language;
  new_per_day: number;
  new_mix_percent: number;
}

export type Theme = "auto" | "light" | "dark";

export interface Item {
  id: number;
  kanji: string | null;
  reading: string;
  meaning: string;
  kanji_gloss: string | null;
  lesson: string;
  category: string;
  is_pattern: boolean;
  irregular: boolean;
  has_audio: boolean;
}

export interface Challenge {
  item: Item;
  direction: Direction;
  due_count: number;
  new_count: number;
  /** Segundos hasta el próximo repaso programado (null si no queda ninguna palabra vista). */
  next_due_seconds: number | null;
}

export interface AnswerResult {
  correct: boolean;
  expected: string[];
  reading: string;
  meaning: string;
  kanji_gloss: string | null;
  has_audio: boolean;
  next_interval_minutes: number;
  xp_gained: number;
  xp_total: number;
  level: number;
  leveled_up: boolean;
  unlocked: Achievement[];
  saver_earned: boolean;
}

export interface DayCount {
  date: string;
  total: number;
  correct: number;
}

export interface LessonStat {
  lesson: string;
  total: number;
  reviewed: number;
  mastered: number;
  reviews: number;
  correct: number;
  accuracy: number;
  mastery_percent: number;
  is_mastered: boolean;
}

export interface Achievement {
  key: string;
  title: string;
  description: string;
  unlocked: boolean;
  unlocked_at: number | null;
  progress: number;
  target: number;
}

export interface DifficultItem {
  id: number;
  japanese: string;
  meaning: string;
  attempts: number;
  failures: number;
  lapses: number;
}

export interface ReviewLogEntry {
  answered_at: number;
  /** Hora local, `YYYY-MM-DD HH:MM:SS`. */
  answered_at_local: string;
  reading: string;
  meaning: string;
  direction: Direction;
  result: boolean;
  quality: number;
  latency_ms: number;
}

export interface ReviewHistoryPage {
  entries: ReviewLogEntry[];
  total: number;
  page: number;
  per_page: number;
  has_more: boolean;
}

export interface ReviewHistoryParams {
  page?: number;
  perPage?: number;
  direction?: Direction | null;
  result?: boolean | null;
  fromTs?: number | null;
  toTs?: number | null;
}

export interface Stats {
  total_items: number;
  total_reviews: number;
  reviewed_today: number;
  correct_today: number;
  accuracy_today: number;
  streak_days: number;
  mastered: number;
  due_now: number;
  new_available: number;
  new_today: number;
  new_per_day: number;
  /** Segundos hasta el próximo repaso programado (null si no queda ninguna palabra vista). */
  next_due_seconds: number | null;
  activity: DayCount[];
  lessons: LessonStat[];
  difficult: DifficultItem[];
  xp_total: number;
  level: number;
  level_title: string;
  level_progress: number;
  next_level_xp: number | null;
  streak_best: number;
  streak_savers: number;
  streak_savers_max: number;
  streak_repairs: number;
  week_reviews: number;
  weekly_goal: number;
  lesson_mastery_percent: number;
  achievements: Achievement[];
}

export interface Bootstrap {
  item_count: number;
  settings: Settings;
  stats: Stats;
  dev_build: boolean;
}

export interface ImportSummary {
  notes: number;
  items_inserted: number;
  items_merged: number;
  sources: number;
  audio_extracted: number;
  audio_reused: number;
  patterns: number;
  deck_name: string;
  warnings: string[];
}

export interface ImportProgress {
  phase: string;
  current: number;
  total: number;
}

export interface FieldMapping {
  reading: number | null;
  kanji: number | null;
  meaning: number | null;
  gloss: number | null;
  audio: number | null;
  uid: number | null;
  tags: number | null;
  lesson: number | null;
  category: number | null;
}

export interface NoteTypeMapping {
  mid: number;
  mapping: FieldMapping;
}

export interface ImportMapping {
  txt: FieldMapping | null;
  note_types: NoteTypeMapping[];
}

export interface NoteTypeInfo {
  mid: number;
  name: string;
  fields: string[];
  count: number;
  sample: string[][];
  suggested: FieldMapping;
}

export interface ImportInspection {
  kind: "txt" | "apkg";
  deck_name: string;
  total_notes: number;
  columns: string[];
  sample: string[][];
  suggested: FieldMapping | null;
  note_types: NoteTypeInfo[];
}

export const api = {
  bootstrap: () => invoke<Bootstrap>("bootstrap"),
  getSettings: () => invoke<Settings>("get_settings"),
  updateSettings: (settings: Settings) =>
    invoke<Settings>("update_settings", { settings }),
  getStats: () => invoke<Stats>("get_stats"),
  importDeck: (path: string, mapping?: ImportMapping | null) =>
    invoke<ImportSummary>("import_deck", { path, mapping: mapping ?? null }),
  inspectImport: (path: string) =>
    invoke<ImportInspection>("inspect_import", { path }),
  nextChallenge: (excludeId?: number) =>
    invoke<Challenge | null>("next_challenge", {
      excludeId: excludeId ?? null,
    }),
  submitAnswer: (
    itemId: number,
    direction: Direction,
    answer: string,
    latencyMs: number,
  ) =>
    invoke<AnswerResult>("submit_answer", {
      itemId,
      direction,
      answer,
      latencyMs,
    }),
  getAudio: (itemId: number) => invoke<number[] | null>("get_audio", { itemId }),
  resetProgress: () => invoke<Stats>("reset_progress"),
  clearLibrary: () => invoke<void>("clear_library"),
  exportBackup: (destination: string) =>
    invoke<string>("export_backup", { destination }),
  restoreBackup: (source: string) =>
    invoke<Settings>("restore_backup", { source }),
  exportHistory: (destination: string) =>
    invoke<string>("export_history", { destination }),
  reviewHistory: (params: ReviewHistoryParams = {}) =>
    invoke<ReviewHistoryPage>("review_history", {
      page: params.page ?? 0,
      perPage: params.perPage ?? 25,
      direction: params.direction ?? null,
      result: params.result ?? null,
      fromTs: params.fromTs ?? null,
      toTs: params.toTs ?? null,
    }),
  practiceNow: () => invoke<void>("practice_now"),
  closePopup: () => invoke<void>("close_popup"),
  hideWidget: () => invoke<void>("hide_widget"),
  showMainWindow: () => invoke<void>("show_main_window"),
  openLogs: () => invoke<void>("open_logs"),
};

export function japaneseDisplay(item: Item): string {
  const kanji = item.kanji?.trim();
  return kanji && kanji.length > 0 ? kanji : item.reading;
}

export function directionLabel(
  direction: Direction,
  language: Language = "es",
): string {
  return translate(
    language,
    direction === "jp_to_es" ? "日本語 → Español" : "Español → 日本語",
  );
}

export function directionHint(
  direction: Direction,
  language: Language = "es",
): string {
  return translate(
    language,
    direction === "jp_to_es"
      ? "Escribe el significado en español"
      : "Escribe la palabra en japonés (kanji, kana o romaji)",
  );
}

export function formatInterval(
  minutes: number,
  language: Language = "es",
): string {
  if (minutes < 60) return translate(language, "{n} min", { n: minutes });
  if (minutes < 1440)
    return translate(language, "{n} h", { n: Math.round(minutes / 60) });
  return translate(language, "{n} d", { n: Math.round(minutes / 1440) });
}
