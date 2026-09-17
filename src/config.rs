use crate::cue::Response;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const MAX_FILE_BYTES: u64 = 64 * 1024;
static NEXT_TEMP_FILE: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AnchorMode {
    #[default]
    Top,
    Posture,
    Overhead,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
pub enum ParryButton {
    #[default]
    LB,
    L1,
    RMB,
}
impl ParryButton {
    pub fn label(self) -> &'static str {
        match self {
            Self::LB => "LB",
            Self::L1 => "L1",
            Self::RMB => "RMB",
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Colors {
    #[serde(with = "color")]
    pub parry: [f32; 4],
    #[serde(with = "color")]
    pub dodge: [f32; 4],
    #[serde(with = "color")]
    pub jump: [f32; 4],
    #[serde(with = "color")]
    pub ready: [f32; 4],
    #[serde(with = "color")]
    pub expired: [f32; 4],
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            parry: rgba(0x2e, 0xf2, 0x45, 0xf2),
            dodge: rgba(0xff, 0x66, 0x1f, 0xff),
            jump: rgba(0x2e, 0xcc, 0xff, 0xff),
            ready: rgba(0xe0, 0xe5, 0xeb, 0xff),
            expired: rgba(0x89, 0x90, 0x99, 0xcc),
        }
    }
}

const fn rgba(r: u8, g: u8, b: u8, a: u8) -> [f32; 4] {
    [
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    ]
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MoveProfile {
    pub model: i32,
    pub animation: i32,
    pub phase: u32,
    pub activation_s: f32,
    pub end_s: f32,
    #[serde(with = "response")]
    pub response: Response,
    pub game_sha256: String,
    pub data_sha256: String,
    pub evidence: String,
    pub encounter: String,
    pub form: String,
    pub contact_min_ms: f32,
    pub contact_max_ms: f32,
    pub early_ms: f32,
    pub late_ms: f32,
    pub preferred_ms: Option<f32>,
    pub trials: u32,
    pub successes: u32,
    pub failures: u32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// Classify the incoming move instead of showing an estimated press window.
    pub incoming_cues: bool,
    /// User preference; this does not read whether the skill is unlocked.
    pub mikiri: bool,
    pub anchor: AnchorMode,
    pub parry_button: ParryButton,
    pub offset_x: f32,
    pub offset_y: f32,
    pub scale: f32,
    pub width: f32,
    pub opacity: f32,
    pub label_size: f32,
    pub safe_margin: f32,
    pub posture_band_top: f32,
    pub posture_gap: f32,
    pub outline_intensity: f32,
    pub glow_intensity: f32,
    pub pulse_intensity: f32,
    pub pulse_duration_ms: f32,
    pub preparation_ms: f32,
    pub display_latency_ms: f32,
    pub input_latency_ms: f32,
    pub parry_lead_ms: f32,
    pub dodge_lead_ms: f32,
    pub jump_lead_ms: f32,
    pub reduced_flash: bool,
    pub parry: bool,
    pub dodge: bool,
    pub jump: bool,
    pub visible: bool,
    pub diagnostics: bool,
    pub diagnostic_logging: bool,
    pub colors: Colors,
    pub profiles: Vec<MoveProfile>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            incoming_cues: true,
            mikiri: true,
            anchor: AnchorMode::Top,
            parry_button: ParryButton::default(),
            offset_x: 0.0,
            offset_y: 0.0,
            scale: 1.0,
            width: 480.0,
            opacity: 0.95,
            label_size: 30.0,
            safe_margin: 24.0,
            posture_band_top: 0.85,
            posture_gap: 12.0,
            outline_intensity: 0.8,
            glow_intensity: 0.35,
            pulse_intensity: 0.6,
            pulse_duration_ms: 80.0,
            preparation_ms: 650.0,
            display_latency_ms: 0.0,
            input_latency_ms: 0.0,
            parry_lead_ms: 150.0,
            dodge_lead_ms: 300.0,
            jump_lead_ms: 300.0,
            reduced_flash: false,
            parry: true,
            dodge: true,
            jump: true,
            visible: true,
            diagnostics: false,
            diagnostic_logging: true,
            colors: Colors::default(),
            profiles: Vec::new(),
        }
    }
}

impl Config {
    // Correct only the untouched 0.8.0 placement/size tuple. Keep custom
    // placements, scales and every unrelated calibration/visibility setting.
    fn migrate_080_layout(&mut self) -> bool {
        if self.anchor == AnchorMode::Posture
            && self.offset_x == 0.0
            && self.offset_y == 0.0
            && self.scale == 1.0
            && self.width == 240.0
            && self.label_size == 20.0
            && self.posture_band_top == 0.90
            && self.posture_gap == 12.0
        {
            self.width = 320.0;
            self.label_size = 30.0;
            self.posture_band_top = 0.85;
            true
        } else {
            false
        }
    }

    pub fn parse(document: &str) -> Result<Self, String> {
        let config: Self =
            toml::from_str(document).map_err(|error| format!("invalid configuration: {error}"))?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), String> {
        bounded("offset_x", self.offset_x, -480.0, 480.0)?;
        bounded("offset_y", self.offset_y, -160.0, 160.0)?;
        bounded("scale", self.scale, 0.5, 1.5)?;
        bounded("width", self.width, 160.0, 640.0)?;
        bounded("opacity", self.opacity, 0.2, 1.0)?;
        bounded("label_size", self.label_size, 14.0, 30.0)?;
        bounded("safe_margin", self.safe_margin, 8.0, 96.0)?;
        bounded("posture_band_top", self.posture_band_top, 0.75, 0.98)?;
        bounded("posture_gap", self.posture_gap, 8.0, 64.0)?;
        bounded("outline_intensity", self.outline_intensity, 0.0, 1.0)?;
        bounded("glow_intensity", self.glow_intensity, 0.0, 1.0)?;
        bounded("pulse_intensity", self.pulse_intensity, 0.0, 1.0)?;
        bounded("pulse_duration_ms", self.pulse_duration_ms, 16.0, 120.0)?;
        bounded("preparation_ms", self.preparation_ms, 350.0, 1500.0)?;
        bounded("display_latency_ms", self.display_latency_ms, 0.0, 120.0)?;
        bounded("input_latency_ms", self.input_latency_ms, 0.0, 120.0)?;
        if self.display_latency_ms + self.input_latency_ms > 150.0 {
            return Err("display_latency_ms + input_latency_ms must be at most 150".into());
        }
        bounded("parry_lead_ms", self.parry_lead_ms, 25.0, 150.0)?;
        bounded("dodge_lead_ms", self.dodge_lead_ms, 25.0, 300.0)?;
        bounded("jump_lead_ms", self.jump_lead_ms, 25.0, 300.0)?;
        for (name, color) in [
            ("parry", self.colors.parry),
            ("dodge", self.colors.dodge),
            ("jump", self.colors.jump),
            ("ready", self.colors.ready),
            ("expired", self.colors.expired),
        ] {
            if color.iter().any(|channel| !channel.is_finite()) || color[3] < 128.0 / 255.0 {
                return Err(format!("colors.{name} alpha must be at least 0x80"));
            }
        }
        let data_sha256 = checked_in_data_sha256();
        let mut identities = HashSet::new();
        for profile in &self.profiles {
            validate_profile(profile, &data_sha256)?;
            if !identities.insert((profile.model, profile.animation, profile.phase)) {
                return Err(format!(
                    "duplicate profile identity {} / {} / {}",
                    profile.model, profile.animation, profile.phase
                ));
            }
        }
        Ok(())
    }
}

pub struct Store {
    path: PathBuf,
    current: Config,
    source_fingerprint: Option<[u8; 32]>,
    diagnostic: Option<String>,
}

impl Store {
    pub fn open(path: PathBuf) -> Self {
        match read_document(&path) {
            Ok(bytes) => match parse_bytes(&bytes) {
                Ok(current) => {
                    let mut store = Self {
                        path,
                        current,
                        source_fingerprint: Some(fingerprint(&bytes)),
                        diagnostic: None,
                    };
                    let _ = store.reload();
                    store
                }
                Err(error) => Self {
                    path,
                    current: Config::default(),
                    source_fingerprint: Some(fingerprint(&bytes)),
                    diagnostic: Some(error),
                },
            },
            Err(_) if is_missing(&path) => {
                let current = Config::default();
                let document = serialize_config(&current);
                match document.and_then(|document| {
                    atomic_write(&path, document.as_bytes(), false, None, |_| {})?;
                    Ok(document)
                }) {
                    Ok(document) => Self {
                        path,
                        current,
                        source_fingerprint: Some(fingerprint(document.as_bytes())),
                        diagnostic: None,
                    },
                    Err(create_error) => match read_document(&path) {
                        Ok(bytes) => match parse_bytes(&bytes) {
                            Ok(current) => Self {
                                path,
                                current,
                                source_fingerprint: Some(fingerprint(&bytes)),
                                diagnostic: None,
                            },
                            Err(error) => Self {
                                path,
                                current,
                                source_fingerprint: Some(fingerprint(&bytes)),
                                diagnostic: Some(error),
                            },
                        },
                        Err(_) => Self {
                            path,
                            current,
                            source_fingerprint: None,
                            diagnostic: Some(create_error),
                        },
                    },
                }
            }
            Err(error) => Self {
                path,
                current: Config::default(),
                source_fingerprint: None,
                diagnostic: Some(error),
            },
        }
    }

    pub fn current(&self) -> &Config {
        &self.current
    }

    pub fn reload(&mut self) -> Result<bool, String> {
        let result = (|| {
            let bytes = read_document(&self.path)?;
            let mut next = parse_bytes(&bytes)?;
            let migrated = next.migrate_080_layout();
            let changed = next != self.current;
            let previous_fingerprint = self.source_fingerprint;
            self.source_fingerprint = Some(fingerprint(&bytes));
            if migrated {
                if let Err(error) = self.persist(&next) {
                    self.source_fingerprint = previous_fingerprint;
                    return Err(error);
                }
            }
            self.current = next;
            Ok(changed)
        })();
        self.finish(result)
    }

    pub fn adjust_placement(&mut self, delta: f32) -> Result<(), String> {
        let result = (|| {
            let mut next = self.current.clone();
            next.offset_y += delta;
            next.validate()?;
            self.persist(&next)?;
            self.current = next;
            Ok(())
        })();
        self.finish(result)
    }

    pub fn reset_placement(&mut self) -> Result<(), String> {
        let result = (|| {
            let mut next = self.current.clone();
            next.offset_x = 0.0;
            next.offset_y = 0.0;
            self.persist(&next)?;
            self.current = next;
            Ok(())
        })();
        self.finish(result)
    }

    pub fn update_visibility(
        &mut self,
        visible: Option<bool>,
        diagnostics: Option<bool>,
    ) -> Result<(), String> {
        let result = (|| {
            let mut next = self.current.clone();
            if let Some(visible) = visible {
                next.visible = visible;
            }
            if let Some(diagnostics) = diagnostics {
                next.diagnostics = diagnostics;
            }
            self.persist(&next)?;
            self.current = next;
            Ok(())
        })();
        self.finish(result)
    }

    pub fn diagnostic(&self) -> Option<&str> {
        self.diagnostic.as_deref()
    }

    fn persist(&mut self, next: &Config) -> Result<(), String> {
        self.persist_with(next, |_| {})
    }

    fn persist_with<F>(&mut self, next: &Config, before_commit: F) -> Result<(), String>
    where
        F: FnOnce(&Path),
    {
        let expected_fingerprint = match read_document(&self.path) {
            Ok(bytes) => {
                parse_bytes(&bytes).map_err(|error| {
                    format!("configuration changed externally and is invalid: {error}")
                })?;
                let fingerprint = fingerprint(&bytes);
                if self.source_fingerprint != Some(fingerprint) {
                    return Err("configuration changed externally; reload before saving".into());
                }
                Some(fingerprint)
            }
            Err(error) if is_missing(&self.path) => {
                if self.source_fingerprint.is_some() {
                    return Err("configuration changed externally; file was removed".into());
                }
                let _ = error;
                None
            }
            Err(error) => return Err(error),
        };

        let document = serialize_config(next)?;
        atomic_write(
            &self.path,
            document.as_bytes(),
            expected_fingerprint.is_some(),
            expected_fingerprint,
            before_commit,
        )?;
        self.source_fingerprint = Some(fingerprint(document.as_bytes()));
        Ok(())
    }

    fn finish<T>(&mut self, result: Result<T, String>) -> Result<T, String> {
        match result {
            Ok(value) => {
                self.diagnostic = None;
                Ok(value)
            }
            Err(error) => {
                self.diagnostic = Some(error.clone());
                Err(error)
            }
        }
    }
}

fn read_document(path: &Path) -> Result<Vec<u8>, String> {
    let file =
        File::open(path).map_err(|error| format!("could not read configuration: {error}"))?;
    let length = file
        .metadata()
        .map_err(|error| format!("could not inspect configuration: {error}"))?
        .len();
    if length > MAX_FILE_BYTES {
        return Err("configuration exceeds the 64 KiB limit".into());
    }
    let mut bytes = Vec::with_capacity(length as usize);
    file.take(MAX_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("could not read configuration: {error}"))?;
    if bytes.len() as u64 > MAX_FILE_BYTES {
        return Err("configuration exceeds the 64 KiB limit".into());
    }
    Ok(bytes)
}

fn parse_bytes(bytes: &[u8]) -> Result<Config, String> {
    let document = std::str::from_utf8(bytes)
        .map_err(|_| "configuration must contain valid UTF-8".to_string())?;
    Config::parse(document)
}

fn fingerprint(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

fn serialize_config(config: &Config) -> Result<String, String> {
    toml::to_string_pretty(config)
        .map_err(|error| format!("could not serialize configuration: {error}"))
}

fn checked_in_data_sha256() -> String {
    format!("{:x}", Sha256::digest(include_bytes!("attack_timings.rs")))
}

fn is_missing(path: &Path) -> bool {
    fs::metadata(path).is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound)
}

fn atomic_write<F>(
    path: &Path,
    bytes: &[u8],
    replace_existing: bool,
    expected_fingerprint: Option<[u8; 32]>,
    before_commit: F,
) -> Result<(), String>
where
    F: FnOnce(&Path),
{
    let directory = path
        .parent()
        .ok_or_else(|| "configuration path has no parent directory".to_string())?;
    fs::create_dir_all(directory)
        .map_err(|error| format!("could not create configuration directory: {error}"))?;
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "configuration path has no valid file name".to_string())?;
    let temp_path = directory.join(format!(
        ".{file_name}.{}.{}.tmp",
        std::process::id(),
        NEXT_TEMP_FILE.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| {
        let mut temp = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
            .map_err(|error| format!("could not create temporary configuration: {error}"))?;
        temp.write_all(bytes)
            .map_err(|error| format!("could not write temporary configuration: {error}"))?;
        temp.flush()
            .map_err(|error| format!("could not flush temporary configuration: {error}"))?;
        temp.sync_all()
            .map_err(|error| format!("could not sync temporary configuration: {error}"))?;
        drop(temp);
        before_commit(path);
        match expected_fingerprint {
            Some(expected) => {
                let current = read_document(path).map_err(|_| {
                    "configuration changed externally before save; reload before saving".to_string()
                })?;
                if fingerprint(&current) != expected {
                    return Err(
                        "configuration changed externally before save; reload before saving".into(),
                    );
                }
            }
            None if path.exists() => {
                return Err("configuration changed externally before save; file appeared".into());
            }
            None => {}
        }
        replace_file(&temp_path, path, replace_existing)
            .map_err(|error| format!("could not replace configuration atomically: {error}"))
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}

#[cfg(windows)]
fn replace_file(from: &Path, to: &Path, replace_existing: bool) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;

    const MOVEFILE_REPLACE_EXISTING: u32 = 0x1;
    const MOVEFILE_WRITE_THROUGH: u32 = 0x8;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn MoveFileExW(existing: *const u16, replacement: *const u16, flags: u32) -> i32;
    }

    let from: Vec<u16> = from.as_os_str().encode_wide().chain(Some(0)).collect();
    let to: Vec<u16> = to.as_os_str().encode_wide().chain(Some(0)).collect();
    // SAFETY: both pointers reference NUL-terminated UTF-16 buffers for this call.
    let flags = MOVEFILE_WRITE_THROUGH
        | if replace_existing {
            MOVEFILE_REPLACE_EXISTING
        } else {
            0
        };
    if unsafe { MoveFileExW(from.as_ptr(), to.as_ptr(), flags) } == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
fn replace_file(from: &Path, to: &Path, replace_existing: bool) -> std::io::Result<()> {
    if replace_existing {
        fs::rename(from, to)
    } else {
        fs::hard_link(from, to)?;
        fs::remove_file(from)
    }
}

fn validate_profile(profile: &MoveProfile, data_sha256: &str) -> Result<(), String> {
    if profile.game_sha256 != crate::reader::RESEARCH_HASH {
        return Err("profile game_sha256 does not match the supported executable".into());
    }
    if profile.data_sha256 != data_sha256 {
        return Err("profile data_sha256 does not match the checked-in timing data".into());
    }
    for (name, value) in [
        ("evidence", &profile.evidence),
        ("encounter", &profile.encounter),
        ("form", &profile.form),
    ] {
        if value.trim().is_empty() {
            return Err(format!("profile {name} must not be empty"));
        }
    }

    let matching: Vec<_> = crate::attack_timings::ATTACKS
        .iter()
        .filter(|entry| entry.0 == profile.model && entry.1 == profile.animation)
        .collect();
    let phase = matching
        .get(profile.phase as usize)
        .ok_or_else(|| "profile does not name an existing phase".to_string())?;
    if !profile.activation_s.is_finite()
        || !profile.end_s.is_finite()
        || profile.activation_s != phase.2
        || profile.end_s != phase.3
    {
        return Err("profile activation_s/end_s do not match the exact phase".into());
    }

    let classified_response = crate::attack_timings::RESPONSES
        .iter()
        .find(|entry| {
            entry.0 == profile.model
                && entry.1 == profile.animation
                && entry.2 == phase.2
                && entry.3 == phase.3
        })
        .map(|entry| match entry.4 {
            1 => Response::Dodge,
            2 => Response::Jump,
            _ => Response::Unverified,
        })
        .unwrap_or(if phase.4 {
            Response::Parry
        } else {
            Response::Unverified
        });
    if profile.response != classified_response {
        return Err("profile response does not match the checked-in classifier".into());
    }
    if profile.response == Response::Unverified {
        return Err("profile response must be parry, dodge, or jump".into());
    }

    bounded(
        "profile contact_min_ms",
        profile.contact_min_ms,
        -500.0,
        500.0,
    )?;
    bounded(
        "profile contact_max_ms",
        profile.contact_max_ms,
        -500.0,
        500.0,
    )?;
    if profile.contact_min_ms > profile.contact_max_ms {
        return Err("profile contact_min_ms must not exceed contact_max_ms".into());
    }
    bounded("profile late_ms", profile.late_ms, 0.0, 300.0)?;
    bounded("profile early_ms", profile.early_ms, 0.0, 300.0)?;
    if profile.late_ms >= profile.early_ms {
        return Err("profile requires late_ms < early_ms".into());
    }
    if profile.contact_max_ms - profile.contact_min_ms >= profile.early_ms - profile.late_ms {
        return Err(
            "profile conservative press interval must be nonempty at nominal 1x speed".into(),
        );
    }
    if profile.preferred_ms.is_some_and(|preferred| {
        !preferred.is_finite() || preferred <= profile.late_ms || preferred > profile.early_ms
    }) {
        return Err("profile preferred_ms must be above late_ms and at most early_ms".into());
    }
    if profile.trials == 0
        || profile
            .successes
            .checked_add(profile.failures)
            .is_none_or(|total| total != profile.trials)
    {
        return Err("profile trials must equal successes + failures and be nonzero".into());
    }
    Ok(())
}

fn bounded(name: &str, value: f32, minimum: f32, maximum: f32) -> Result<(), String> {
    if value.is_finite() && (minimum..=maximum).contains(&value) {
        Ok(())
    } else {
        Err(format!(
            "{name} must be finite and between {minimum} and {maximum}"
        ))
    }
}

mod color {
    use serde::{de::Error, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(color: &[f32; 4], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = color.map(|channel| (channel * 255.0).round() as u8);
        serializer.serialize_str(&format!(
            "#{:02X}{:02X}{:02X}{:02X}",
            bytes[0], bytes[1], bytes[2], bytes[3]
        ))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[f32; 4], D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if value.len() != 9 || !value.starts_with('#') {
            return Err(D::Error::custom("color must be exactly #RRGGBBAA"));
        }
        let mut bytes = [0_u8; 4];
        for (index, byte) in bytes.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&value[1 + index * 2..3 + index * 2], 16)
                .map_err(|_| D::Error::custom("color must be exactly #RRGGBBAA"))?;
        }
        Ok(bytes.map(|channel| f32::from(channel) / 255.0))
    }
}

mod response {
    use crate::cue::Response;
    use serde::{de::Error, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(response: &Response, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(response.label())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Response, D::Error>
    where
        D: Deserializer<'de>,
    {
        match String::deserialize(deserializer)?.as_str() {
            "unverified" => Ok(Response::Unverified),
            "parry" => Ok(Response::Parry),
            "dodge" => Ok(Response::Dodge),
            "jump" => Ok(Response::Jump),
            _ => Err(D::Error::custom(
                "response must be parry, dodge, jump, or unverified",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{checked_in_data_sha256, AnchorMode, Config, Store};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    const VALID_PROFILE: &str = r#"
[[profiles]]
model = 1010
animation = 3000
phase = 0
activation_s = 0.666666687
end_s = 0.800000012
response = "parry"
game_sha256 = "637aca527538c0ec6e1f136c8ed66046e95dfbdbb1f51926e134d9916398b856"
data_sha256 = "__DATA_SHA256__"
evidence = "local calibration log trial-set-1"
encounter = "training soldier"
form = "model-animation-phase"
contact_min_ms = -10
contact_max_ms = 20
early_ms = 100
late_ms = 50
preferred_ms = 75
trials = 3
successes = 2
failures = 1
"#;

    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    fn valid_profile() -> String {
        VALID_PROFILE.replace("__DATA_SHA256__", &checked_in_data_sha256())
    }

    fn test_directory() -> PathBuf {
        let id = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "sekiro-deflect-observer-config-{}-{id}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn empty_document_uses_approved_defaults() {
        let config = Config::parse("").expect("empty TOML should use safe defaults");

        assert_eq!(config, Config::default());
        assert_eq!(config.anchor, AnchorMode::Top);
        assert_eq!(config.offset_x, 0.0);
        assert_eq!(config.offset_y, 0.0);
        assert_eq!(config.scale, 1.0);
        assert_eq!(config.width, 480.0);
        assert_eq!(config.opacity, 0.95);
        assert_eq!(config.label_size, 30.0);
        assert_eq!(config.safe_margin, 24.0);
        assert_eq!(config.posture_band_top, 0.85);
        assert_eq!(config.posture_gap, 12.0);
        assert_eq!(config.preparation_ms, 650.0);
        assert_eq!(config.parry_lead_ms, 150.0);
        assert_eq!(config.dodge_lead_ms, 300.0);
        assert_eq!(config.jump_lead_ms, 300.0);
        assert!(config.parry && config.dodge && config.jump);
        assert!(config.visible && config.diagnostic_logging);
        assert!(!config.diagnostics && !config.reduced_flash);
        assert!(config.profiles.is_empty());
    }

    #[test]
    fn packaged_defaults_parse_to_the_safe_defaults() {
        let packaged = include_str!("../packaging/cue.toml");
        assert_eq!(Config::parse(packaged).unwrap(), Config::default());
    }

    #[test]
    fn parser_accepts_inclusive_bounds_and_hex_colors() {
        let config = Config::parse(
            r##"
anchor = "overhead"
offset_x = -480
offset_y = 160
scale = 0.5
width = 360
opacity = 0.2
label_size = 14
safe_margin = 96
posture_band_top = 0.98
posture_gap = 64
outline_intensity = 0
glow_intensity = 1
pulse_intensity = 0
pulse_duration_ms = 16
preparation_ms = 1500
display_latency_ms = 30
input_latency_ms = 120
parry_lead_ms = 25
dodge_lead_ms = 300
jump_lead_ms = 25

[colors]
parry = "#01020380"
"##,
        )
        .expect("inclusive bounds should be valid");

        assert_eq!(config.anchor, AnchorMode::Overhead);
        assert_eq!(config.offset_x, -480.0);
        assert_eq!(
            config.colors.parry,
            [1.0 / 255.0, 2.0 / 255.0, 3.0 / 255.0, 128.0 / 255.0]
        );
    }

    #[test]
    fn parser_rejects_nonfinite_and_out_of_range_settings() {
        let invalid = [
            "offset_x = 481",
            "offset_y = -161",
            "scale = 0.499",
            "width = 641",
            "opacity = 0.19",
            "label_size = 31",
            "safe_margin = 7",
            "posture_band_top = 0.99",
            "posture_gap = 7",
            "outline_intensity = -0.01",
            "glow_intensity = 1.01",
            "pulse_intensity = nan",
            "pulse_duration_ms = 121",
            "preparation_ms = 349",
            "display_latency_ms = inf",
            "input_latency_ms = 121",
            "display_latency_ms = 80\ninput_latency_ms = 71",
            "parry_lead_ms = 151",
            "dodge_lead_ms = 301",
            "jump_lead_ms = 24",
            "[colors]\nready = \"#FFFFFF7F\"",
        ];

        for document in invalid {
            assert!(
                Config::parse(document).is_err(),
                "unsafe setting was accepted: {document}"
            );
        }
    }

    #[test]
    fn parser_rejects_unknown_invariant_override_keys() {
        for document in [
            "lock_required = false",
            "hash_check = false",
            "freshness_ms = 500",
            "unknown = true",
            "[colors]\nunknown = \"#FFFFFFFF\"",
        ] {
            assert!(Config::parse(document).is_err(), "unknown key was accepted");
        }
    }

    #[test]
    fn parser_accepts_profile_matching_checked_in_classifier() {
        let config = Config::parse(&valid_profile()).expect("matching profile should parse");

        assert_eq!(config.profiles.len(), 1);
        assert_eq!(config.profiles[0].phase, 0);
        assert_eq!(config.profiles[0].preferred_ms, Some(75.0));
    }

    #[test]
    fn parser_rejects_profile_metadata_that_cannot_prove_exact_identity() {
        let valid_profile = valid_profile();
        let invalid_replacements = [
            ("model = 1010", "model = 9999"),
            ("phase = 0", "phase = 1"),
            ("activation_s = 0.666666687", "activation_s = 0.67"),
            ("end_s = 0.800000012", "end_s = 0.81"),
            ("response = \"parry\"", "response = \"dodge\""),
            (
                "637aca527538c0ec6e1f136c8ed66046e95dfbdbb1f51926e134d9916398b856",
                "037aca527538c0ec6e1f136c8ed66046e95dfbdbb1f51926e134d9916398b856",
            ),
            (
                "evidence = \"local calibration log trial-set-1\"",
                "evidence = \" \"",
            ),
            ("encounter = \"training soldier\"", "encounter = \"\""),
            ("form = \"model-animation-phase\"", "form = \"\""),
        ];

        for (from, to) in invalid_replacements {
            let document = valid_profile.replacen(from, to, 1);
            assert!(
                Config::parse(&document).is_err(),
                "invalid replacement: {to}"
            );
        }
        let wrong_data_hash = valid_profile.replacen(
            &checked_in_data_sha256(),
            "0000000000000000000000000000000000000000000000000000000000000000",
            1,
        );
        assert!(Config::parse(&wrong_data_hash).is_err());
    }

    #[test]
    fn parser_rejects_impossible_profile_intervals_and_trial_counts() {
        let valid_profile = valid_profile();
        let invalid_replacements = [
            ("contact_min_ms = -10", "contact_min_ms = -501"),
            ("contact_max_ms = 20", "contact_max_ms = 501"),
            ("contact_min_ms = -10", "contact_min_ms = 21"),
            ("early_ms = 100", "early_ms = 301"),
            ("late_ms = 50", "late_ms = -1"),
            ("late_ms = 50", "late_ms = 100"),
            ("preferred_ms = 75", "preferred_ms = 50"),
            ("preferred_ms = 75", "preferred_ms = 101"),
            ("contact_max_ms = 20", "contact_max_ms = 40"),
            ("trials = 3", "trials = 0"),
            ("successes = 2", "successes = 1"),
            ("contact_max_ms = 20", "contact_max_ms = nan"),
        ];

        for (from, to) in invalid_replacements {
            let document = valid_profile.replacen(from, to, 1);
            assert!(
                Config::parse(&document).is_err(),
                "invalid replacement: {to}"
            );
        }
    }

    #[test]
    fn parser_rejects_profiles_for_unverified_responses() {
        let unverified = valid_profile()
            .replacen("animation = 3000", "animation = 3005", 1)
            .replacen(
                "activation_s = 0.666666687",
                "activation_s = 0.966666639",
                1,
            )
            .replacen("end_s = 0.800000012", "end_s = 1.033333302", 1)
            .replacen("response = \"parry\"", "response = \"unverified\"", 1);

        assert!(Config::parse(&unverified).is_err());
    }

    #[test]
    fn parser_rejects_duplicate_profile_identity() {
        let valid_profile = valid_profile();
        let duplicate = format!("{valid_profile}\n{valid_profile}");
        assert!(Config::parse(&duplicate).is_err());
    }

    #[test]
    fn invalid_reload_preserves_the_entire_last_valid_config() {
        let directory = test_directory();
        let path = directory.join("cue.toml");
        fs::write(&path, "offset_x = 17\nvisible = false\n").unwrap();
        let mut store = Store::open(path.clone());
        let original = store.current().clone();
        fs::write(&path, "offset_x = nan\n").unwrap();

        assert!(store.reload().is_err());
        assert_eq!(store.current(), &original);
        assert!(store.diagnostic().is_some());

        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn reload_rejects_files_larger_than_64_kib() {
        let directory = test_directory();
        let path = directory.join("cue.toml");
        fs::write(&path, "offset_y = 8\n").unwrap();
        let mut store = Store::open(path.clone());
        let original = store.current().clone();
        fs::write(&path, vec![b' '; 65_537]).unwrap();

        assert!(store.reload().is_err());
        assert_eq!(store.current(), &original);

        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn placement_adjustment_atomically_persists_a_complete_config() {
        let directory = test_directory();
        let path = directory.join("cue.toml");
        fs::write(&path, "offset_y = 8\n").unwrap();
        let mut store = Store::open(path.clone());

        store.adjust_placement(8.0).unwrap();

        assert_eq!(store.current().offset_y, 16.0);
        let persisted = fs::read_to_string(&path).unwrap();
        assert_eq!(Config::parse(&persisted).unwrap(), *store.current());
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);

        store.reset_placement().unwrap();
        assert_eq!(store.current().offset_x, 0.0);
        assert_eq!(store.current().offset_y, 0.0);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn placement_persist_reports_external_edit_conflicts_without_overwriting() {
        let directory = test_directory();
        let path = directory.join("cue.toml");
        fs::write(&path, "offset_y = 0\n").unwrap();
        let mut store = Store::open(path.clone());
        fs::write(&path, "offset_y = 20\n").unwrap();

        let error = store.adjust_placement(8.0).unwrap_err();

        assert!(error.contains("changed externally"));
        assert_eq!(store.current().offset_y, 0.0);
        assert_eq!(fs::read_to_string(&path).unwrap(), "offset_y = 20\n");
        assert!(store.reload().unwrap());
        assert_eq!(store.current().offset_y, 20.0);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn placement_persist_rechecks_external_edit_after_temp_is_ready() {
        let directory = test_directory();
        let path = directory.join("cue.toml");
        fs::write(&path, "offset_y = 0\n").unwrap();
        let mut store = Store::open(path.clone());
        let mut next = store.current().clone();
        next.offset_y = 8.0;

        let error = store
            .persist_with(&next, |target| {
                fs::write(target, "offset_y = 20\n").unwrap();
            })
            .unwrap_err();

        assert!(error.contains("changed externally"));
        assert_eq!(store.current().offset_y, 0.0);
        assert_eq!(fs::read_to_string(&path).unwrap(), "offset_y = 20\n");
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn placement_persist_never_replaces_malformed_external_content() {
        let directory = test_directory();
        let path = directory.join("cue.toml");
        fs::write(&path, "offset_y = 0\n").unwrap();
        let mut store = Store::open(path.clone());
        fs::write(&path, "offset_y = nope\n").unwrap();

        assert!(store.adjust_placement(8.0).is_err());
        assert_eq!(store.current().offset_y, 0.0);
        assert_eq!(fs::read_to_string(&path).unwrap(), "offset_y = nope\n");
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn missing_initial_file_is_created_from_safe_defaults() {
        let directory = test_directory();
        let path = directory.join("cue.toml");

        let store = Store::open(path.clone());

        assert_eq!(store.current(), &Config::default());
        assert_eq!(
            Config::parse(&fs::read_to_string(&path).unwrap()).unwrap(),
            Config::default()
        );
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn invalid_initial_file_uses_defaults_without_overwriting_user_content() {
        let directory = test_directory();
        let path = directory.join("cue.toml");
        fs::write(&path, "parry_lead_ms = 999\n").unwrap();

        let store = Store::open(path.clone());

        assert_eq!(store.current(), &Config::default());
        assert!(store.diagnostic().is_some());
        assert_eq!(fs::read_to_string(&path).unwrap(), "parry_lead_ms = 999\n");
        assert_eq!(fs::read_dir(&directory).unwrap().count(), 1);
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn visibility_update_persists_both_runtime_switches() {
        let directory = test_directory();
        let path = directory.join("cue.toml");
        fs::write(&path, "").unwrap();
        let mut store = Store::open(path.clone());

        store.update_visibility(Some(false), None).unwrap();
        store.update_visibility(None, Some(true)).unwrap();

        assert!(!store.current().visible);
        assert!(store.current().diagnostics);
        let persisted = Config::parse(&fs::read_to_string(&path).unwrap()).unwrap();
        assert!(!persisted.visible);
        assert!(persisted.diagnostics);
        fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn untouched_080_layout_migrates_once_without_changing_other_settings() {
        let directory = test_directory();
        let path = directory.join("cue.toml");
        let old = Config {
            anchor: AnchorMode::Posture,
            width: 240.0,
            label_size: 20.0,
            posture_band_top: 0.90,
            input_latency_ms: 17.0,
            reduced_flash: true,
            ..Config::default()
        };
        fs::write(&path, super::serialize_config(&old).unwrap()).unwrap();
        let store = Store::open(path.clone());
        assert_eq!(store.current().width, 320.0);
        assert_eq!(store.current().label_size, 30.0);
        assert_eq!(store.current().posture_band_top, 0.85);
        assert_eq!(store.current().input_latency_ms, 17.0);
        assert!(store.current().reduced_flash);
        let persisted = fs::read(&path).unwrap();
        assert_eq!(super::parse_bytes(&persisted).unwrap(), *store.current());
        assert_eq!(*Store::open(path.clone()).current(), *store.current());
        assert_eq!(fs::read(&path).unwrap(), persisted);
        let custom = Config {
            offset_y: -24.0,
            ..old
        };
        fs::write(&path, super::serialize_config(&custom).unwrap()).unwrap();
        assert_eq!(*Store::open(path).current(), custom);
        fs::remove_dir_all(directory).unwrap();
    }
}
