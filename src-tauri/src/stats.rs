// Yerel Calma Gecmisi ve Istatistikler ("Bugun 3 saat 12 dk dinledin, en cok Simge") + Haftalik Ozet Karti.
// Tum veriler yalnizca yerel diskte (%APPDATA%\com.island.dynamic\listening_stats.json) tutulur.
// Yalnizca Ayarlar -> "Calma gecmisi ve istatistik" (listening_stats) acikken kayit yapar.
use crate::util::{data_dir, emit};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Mutex;
use windows::Win32::System::SystemInformation::GetLocalTime;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct DayStat {
    #[serde(default)]
    pub total_secs: u64,
    #[serde(default)]
    pub artists: HashMap<String, u64>, // sanatci -> saniye
    #[serde(default)]
    pub tracks: HashMap<String, u64>,  // "Sarki — Sanatci" -> saniye
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct StatsDb {
    #[serde(default)]
    pub days: HashMap<String, DayStat>, // "YYYY-MM-DD" -> DayStat
}

static DB: Mutex<Option<StatsDb>> = Mutex::new(None);
static DIRTY_TICKS: Mutex<u32> = Mutex::new(0);

fn file() -> std::path::PathBuf {
    data_dir().join("listening_stats.json")
}

/// Yerel tarihi "YYYY-MM-DD" olarak dondurur
fn today_key() -> String {
    let st = unsafe { GetLocalTime() };
    format!("{:04}-{:02}-{:02}", st.wYear, st.wMonth, st.wDay)
}

/// Son N gunun tarih anahtarlarini ("YYYY-MM-DD") hesaplar
fn recent_days(n: usize) -> Vec<String> {
    let st = unsafe { GetLocalTime() };
    let (mut y, mut m, mut d) = (st.wYear as i32, st.wMonth as i32, st.wDay as i32);
    let days_in_month = |yr: i32, mo: i32| -> i32 {
        match mo {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 => if (yr % 4 == 0 && yr % 100 != 0) || (yr % 400 == 0) { 29 } else { 28 },
            _ => 30,
        }
    };
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        out.push(format!("{:04}-{:02}-{:02}", y, m, d));
        d -= 1;
        if d < 1 {
            m -= 1;
            if m < 1 {
                m = 12;
                y -= 1;
            }
            d = days_in_month(y, m);
        }
    }
    out
}

pub fn init() {
    let db: StatsDb = std::fs::read_to_string(file())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    *DB.lock().unwrap() = Some(db);
}

fn save_db(db: &StatsDb) {
    let _ = std::fs::write(file(), serde_json::to_string(db).unwrap_or_default());
}

/// Medya dongusunden her ~400ms'de bir cagrilir (sadece calma devam ederken)
pub fn record_tick(title: &str, artist: &str, delta_ms: u64) {
    if !crate::settings::get().listening_stats || title.trim().is_empty() {
        return;
    }
    // 1 saniyelik birikimlerle kaydet (400ms * 2.5 -> yuvarlatilmis)
    static ACC_MS: Mutex<u64> = Mutex::new(0);
    let add_secs = {
        let mut acc = ACC_MS.lock().unwrap();
        *acc += delta_ms;
        if *acc < 1000 {
            return;
        }
        let s = *acc / 1000;
        *acc %= 1000;
        s
    };

    let day_key = today_key();
    let clean_artist = artist
        .split(|c| c == ',' || c == '&')
        .next()
        .unwrap_or(artist)
        .trim()
        .to_string();
    let track_label = if clean_artist.is_empty() {
        title.trim().to_string()
    } else {
        format!("{} — {}", title.trim(), clean_artist)
    };

    let mut g = DB.lock().unwrap();
    let db = g.get_or_insert_with(StatsDb::default);
    let entry = db.days.entry(day_key).or_default();
    entry.total_secs = entry.total_secs.saturating_add(add_secs);
    if !clean_artist.is_empty() {
        *entry.artists.entry(clean_artist).or_insert(0) += add_secs;
    }
    *entry.tracks.entry(track_label).or_insert(0) += add_secs;

    // 30 gunden eski kayitlari temizle
    if db.days.len() > 35 {
        let keep = recent_days(30);
        db.days.retain(|k, _| keep.contains(k));
    }

    let mut dt = DIRTY_TICKS.lock().unwrap();
    *dt += 1;
    if *dt >= 10 {
        *dt = 0;
        let clone = db.clone();
        drop(g);
        drop(dt);
        save_db(&clone);
        emit("stats", summary());
    }
}

fn top_items(map: &HashMap<String, u64>, n: usize) -> Vec<Value> {
    let mut v: Vec<(&String, &u64)> = map.iter().collect();
    v.sort_by(|a, b| b.1.cmp(a.1));
    v.into_iter()
        .take(n)
        .map(|(name, secs)| json!({ "name": name, "secs": *secs }))
        .collect()
}

/// Gunluk ve haftalik ozet verisi
pub fn summary() -> Value {
    let g = DB.lock().unwrap();
    let Some(db) = g.as_ref() else {
        return json!({ "todaySecs": 0, "weekSecs": 0, "topArtistToday": "", "topArtistsWeek": [], "topTracksWeek": [], "dailyBars": [] });
    };
    let today = today_key();
    let today_stat = db.days.get(&today).cloned().unwrap_or_default();
    let top_artist_today = top_items(&today_stat.artists, 1)
        .first()
        .and_then(|v| v["name"].as_str().map(String::from))
        .unwrap_or_default();

    let week_keys = recent_days(7);
    let mut week_secs = 0u64;
    let mut week_artists: HashMap<String, u64> = HashMap::new();
    let mut week_tracks: HashMap<String, u64> = HashMap::new();
    let mut daily_bars: Vec<Value> = Vec::new();

    for k in week_keys.iter().rev() {
        let st = db.days.get(k);
        let s = st.map(|d| d.total_secs).unwrap_or(0);
        week_secs += s;
        if let Some(d) = st {
            for (a, sec) in &d.artists {
                *week_artists.entry(a.clone()).or_insert(0) += *sec;
            }
            for (t, sec) in &d.tracks {
                *week_tracks.entry(t.clone()).or_insert(0) += *sec;
            }
        }
        daily_bars.push(json!({ "date": k, "secs": s }));
    }

    json!({
        "todaySecs": today_stat.total_secs,
        "topArtistToday": top_artist_today,
        "weekSecs": week_secs,
        "topArtistsWeek": top_items(&week_artists, 5),
        "topTracksWeek": top_items(&week_tracks, 5),
        "dailyBars": daily_bars,
    })
}

#[tauri::command]
pub fn get_stats() -> Value {
    summary()
}
