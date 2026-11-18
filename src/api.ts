import { invoke } from "@tauri-apps/api/core";

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
}

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
}

export interface AnswerResult {
  correct: boolean;
  expected: string[];
  reading: string;
  meaning: string;
  kanji_gloss: string | null;
  has_audio: boolean;
  next_interval_minutes: number;
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
}

export interface DifficultItem {
  id: number;
  japanese: string;
  meaning: string;
  attempts: number;
  failures: number;
  lapses: number;
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
  activity: DayCount[];
  lessons: LessonStat[];
  difficult: DifficultItem[];
}

export interface Bootstrap {
  item_count: number;
  settings: Settings;
  stats: Stats;
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

export const api = {
  bootstrap: () => invoke<Bootstrap>("bootstrap"),
  getSettings: () => invoke<Settings>("get_settings"),
  updateSettings: (settings: Settings) =>
    invoke<Settings>("update_settings", { settings }),
  getStats: () => invoke<Stats>("get_stats"),
  importDeck: (path: string) => invoke<ImportSummary>("import_deck", { path }),
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

export function directionLabel(direction: Direction): string {
  return direction === "jp_to_es" ? "日本語 → Español" : "Español → 日本語";
}

export function directionHint(direction: Direction): string {
  return direction === "jp_to_es"
    ? "Escribe el significado en español"
    : "Escribe la palabra en japonés (kanji, kana o romaji)";
}

export function formatInterval(minutes: number): string {
  if (minutes < 60) return `${minutes} min`;
  if (minutes < 1440) return `${Math.round(minutes / 60)} h`;
  return `${Math.round(minutes / 1440)} d`;
}
