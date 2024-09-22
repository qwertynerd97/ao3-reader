mod preset;
mod ao3_settings;

use std::env;
use std::ops::Index;
use std::fmt::{self, Debug};
use std::path::Path;
use std::path::PathBuf;
use std::collections::{BTreeMap, HashMap};
use fxhash::FxHashSet;
use serde::{Serialize, Deserialize};
use crate::metadata::{SortMethod, TextAlign};
use crate::frontlight::LightLevels;
use crate::color::BLACK;
use crate::device::CURRENT_DEVICE;
use crate::library::Library;
use crate::unit::mm_to_px;
use self::ao3_settings::Ao3Settings;
use crate::helpers::{load_toml};

pub use self::preset::{LightPreset, guess_frontlight};

pub const SETTINGS_PATH: &str = "Settings.toml";
pub const DEFAULT_FONT_PATH: &str = "/mnt/onboard/fonts";
pub const INTERNAL_CARD_ROOT: &str = "/mnt/onboard";
pub const EXTERNAL_CARD_ROOT: &str = "/mnt/sd";
pub const LOGO_SPECIAL_PATH: &str = "logo:";
pub const COVER_SPECIAL_PATH: &str = "cover:";
// Default font size in points.
pub const DEFAULT_FONT_SIZE: f32 = 11.0;
// Default margin width in millimeters.
pub const DEFAULT_MARGIN_WIDTH: i32 = 8;
// Default line height in ems.
pub const DEFAULT_LINE_HEIGHT: f32 = 1.2;
// Default font family name.
pub const DEFAULT_FONT_FAMILY: &str = "Libertinus Serif";
// Default text alignment.
pub const DEFAULT_TEXT_ALIGN: TextAlign = TextAlign::Left;
pub const HYPHEN_PENALTY: i32 = 50;
pub const STRETCH_TOLERANCE: f32 = 1.26;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RotationLock {
    Landscape,
    Portrait,
    Current,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ButtonScheme {
    Natural,
    Inverted,
}

impl fmt::Display for ButtonScheme {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntermKind {
    Suspend,
    PowerOff,
    Share,
}

impl IntermKind {
    pub fn text(&self) -> &str {
        match self {
            IntermKind::Suspend => "Sleeping",
            IntermKind::PowerOff => "Powered off",
            IntermKind::Share => "Shared",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Intermissions {
    suspend: PathBuf,
    power_off: PathBuf,
    share: PathBuf,
}

impl Index<IntermKind> for Intermissions {
    type Output = PathBuf;

    fn index(&self, key: IntermKind) -> &Self::Output {
        match key {
            IntermKind::Suspend => &self.suspend,
            IntermKind::PowerOff => &self.power_off,
            IntermKind::Share => &self.share,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Settings {
    pub selected_library: usize,
    pub keyboard_layout: String,
    pub frontlight: bool,
    pub wifi: bool,
    pub inverted: bool,
    pub sleep_cover: bool,
    pub auto_share: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation_lock: Option<RotationLock>,
    pub button_scheme: ButtonScheme,
    pub auto_suspend: f32,
    pub auto_power_off: f32,
    pub time_format: String,
    pub date_format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_urls_queue: Option<PathBuf>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub libraries: Vec<LibrarySettings>,
    pub intermissions: Intermissions,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub frontlight_presets: Vec<LightPreset>,
    pub home: HomeSettings,
    pub reader: ReaderSettings,
    pub import: ImportSettings,
    pub dictionary: DictionarySettings,
    pub sketch: SketchSettings,
    pub calculator: CalculatorSettings,
    pub battery: BatterySettings,
    pub frontlight_levels: LightLevels,
    pub ao3: Ao3Settings
}

impl Settings {
    pub fn load_settings() -> Settings {
        let path = Path::new(SETTINGS_PATH);
        let settings = if path.exists() {
            load_toml::<Settings, _>(path)
                .map_err(|e| eprintln!("Can't open Settings.toml: {:#}.", e))
                .unwrap()
        } else {
            Default::default()
        };
        settings
    }

    pub fn get_current_library(&self) -> Library {
        let selected_valid = self.selected_library < self.libraries.len();
        let selected = if selected_valid { self.selected_library } else { 0 };

        if self.libraries.is_empty() {
            let default_lib: LibrarySettings = Default::default();
            return Library::new(&default_lib.path, default_lib.mode)
                .map_err(|e| eprintln!("Can't open Library: {:#}.", e))
                .unwrap();
        }

        let library_settings = &self.libraries[selected];
        Library::new(&library_settings.path, library_settings.mode)
            .map_err(|e| eprintln!("Can't open Library: {:#}.", e))
            .unwrap()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LibraryMode {
    Database,
    Filesystem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct LibrarySettings {
    pub name: String,
    pub path: PathBuf,
    pub mode: LibraryMode,
    pub sort_method: SortMethod,
    pub first_column: FirstColumn,
    pub second_column: SecondColumn,
    pub thumbnail_previews: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub hooks: Vec<Hook>,
}

impl Default for LibrarySettings {
    fn default() -> Self {
        LibrarySettings {
            name: "Unnamed".to_string(),
            path: env::current_dir().ok()
                      .unwrap_or_else(|| PathBuf::from("/")),
            mode: LibraryMode::Database,
            sort_method: SortMethod::Opened,
            first_column: FirstColumn::TitleAndAuthor,
            second_column: SecondColumn::Progress,
            thumbnail_previews: true,
            hooks: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ImportSettings {
    pub unshare_trigger: bool,
    pub startup_trigger: bool,
    pub sync_metadata: bool,
    pub metadata_kinds: FxHashSet<String>,
    pub allowed_kinds: FxHashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct DictionarySettings {
    pub margin_width: i32,
    pub font_size: f32,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub languages: BTreeMap<String, Vec<String>>,
}

impl Default for DictionarySettings {
    fn default() -> Self {
        DictionarySettings {
            font_size: 11.0,
            margin_width: 4,
            languages: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct SketchSettings {
    pub save_path: PathBuf,
    pub notify_success: bool,
    pub pen: Pen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct CalculatorSettings {
    pub font_size: f32,
    pub margin_width: i32,
    pub history_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Pen {
    pub size: i32,
    pub color: u8,
    pub dynamic: bool,
    pub amplitude: f32,
    pub min_speed: f32,
    pub max_speed: f32,
}

impl Default for Pen {
    fn default() -> Self {
        Pen {
            size: 2,
            color: BLACK,
            dynamic: true,
            amplitude: 4.0,
            min_speed: 0.0,
            max_speed: mm_to_px(254.0, CURRENT_DEVICE.dpi),
        }
    }
}

impl Default for SketchSettings {
    fn default() -> Self {
        SketchSettings {
            save_path: PathBuf::from("Sketches"),
            notify_success: true,
            pen: Pen::default(),
        }
    }
}

impl Default for CalculatorSettings {
    fn default() -> Self {
        CalculatorSettings {
            font_size: 8.0,
            margin_width: 2,
            history_size: 4096,
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Columns {
    first: FirstColumn,
    second: SecondColumn,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FirstColumn {
    TitleAndAuthor,
    FileName,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SecondColumn {
    Progress,
    Year,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct Hook {
    pub path: PathBuf,
    pub program: PathBuf,
    pub sort_method: Option<SortMethod>,
    pub first_column: Option<FirstColumn>,
    pub second_column: Option<SecondColumn>,
}

impl Default for Hook {
    fn default() -> Self {
        Hook {
            path: PathBuf::default(),
            program: PathBuf::default(),
            sort_method: None,
            first_column: None,
            second_column: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct HomeSettings {
    pub address_bar: bool,
    pub navigation_bar: bool,
    pub max_levels: usize,
    pub max_trash_size: u64,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct RefreshRateSettings {
    #[serde(flatten)]
    pub global: RefreshRatePair,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub by_kind: HashMap<String, RefreshRatePair>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct RefreshRatePair {
    pub regular: u8,
    pub inverted: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ReaderSettings {
    pub finished: FinishedAction,
    pub south_east_corner: SouthEastCornerAction,
    pub bottom_right_gesture: BottomRightGestureAction,
    pub south_strip: SouthStripAction,
    pub west_strip: WestStripAction,
    pub east_strip: EastStripAction,
    pub strip_width: f32,
    pub corner_width: f32,
    pub font_path: String,
    pub font_family: String,
    pub font_size: f32,
    pub min_font_size: f32,
    pub max_font_size: f32,
    pub text_align: TextAlign,
    pub margin_width: i32,
    pub min_margin_width: i32,
    pub max_margin_width: i32,
    pub line_height: f32,
    pub continuous_fit_to_width: bool,
    pub ignore_document_css: bool,
    pub dithered_kinds: FxHashSet<String>,
    pub paragraph_breaker: ParagraphBreakerSettings,
    pub refresh_rate: RefreshRateSettings,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct ParagraphBreakerSettings {
    pub hyphen_penalty: i32,
    pub stretch_tolerance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct BatterySettings {
    pub warn: f32,
    pub power_off: f32,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FinishedAction {
    Notify,
    Close,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SouthEastCornerAction {
    NextPage,
    GoToPage,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BottomRightGestureAction {
    ToggleDithered,
    ToggleInverted,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SouthStripAction {
    ToggleBars,
    NextPage,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EastStripAction {
    PreviousPage,
    NextPage,
    None,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WestStripAction {
    PreviousPage,
    NextPage,
    None,
}

impl Default for RefreshRateSettings {
    fn default() -> Self {
        RefreshRateSettings {
            global: RefreshRatePair { regular: 8, inverted: 2 },
            by_kind: HashMap::new(),
        }
    }
}

impl Default for HomeSettings {
    fn default() -> Self {
        HomeSettings {
            address_bar: false,
            navigation_bar: true,
            max_levels: 3,
            max_trash_size: 32 * (1 << 20),
        }
    }
}

impl Default for ParagraphBreakerSettings {
    fn default() -> Self {
        ParagraphBreakerSettings {
            hyphen_penalty: HYPHEN_PENALTY,
            stretch_tolerance: STRETCH_TOLERANCE,
        }
    }
}

impl Default for ReaderSettings {
    fn default() -> Self {
        ReaderSettings {
            finished: FinishedAction::Close,
            south_east_corner: SouthEastCornerAction::GoToPage,
            bottom_right_gesture: BottomRightGestureAction::ToggleDithered,
            south_strip: SouthStripAction::ToggleBars,
            west_strip: WestStripAction::PreviousPage,
            east_strip: EastStripAction::NextPage,
            strip_width: 0.6,
            corner_width: 0.4,
            font_path: DEFAULT_FONT_PATH.to_string(),
            font_family: DEFAULT_FONT_FAMILY.to_string(),
            font_size: DEFAULT_FONT_SIZE,
            min_font_size: DEFAULT_FONT_SIZE / 2.0,
            max_font_size: 3.0 * DEFAULT_FONT_SIZE / 2.0,
            text_align: DEFAULT_TEXT_ALIGN,
            margin_width: DEFAULT_MARGIN_WIDTH,
            min_margin_width: DEFAULT_MARGIN_WIDTH.saturating_sub(8),
            max_margin_width: DEFAULT_MARGIN_WIDTH.saturating_add(2),
            line_height: DEFAULT_LINE_HEIGHT,
            continuous_fit_to_width: true,
            ignore_document_css: false,
            dithered_kinds: ["cbz", "png", "jpg", "jpeg"].iter().map(|k| k.to_string()).collect(),
            paragraph_breaker: ParagraphBreakerSettings::default(),
            refresh_rate: RefreshRateSettings::default(),
        }
    }
}

impl Default for ImportSettings {
    fn default() -> Self {
        ImportSettings {
            unshare_trigger: true,
            startup_trigger: true,
            sync_metadata: true,
            metadata_kinds: ["epub", "pdf", "djvu"].iter().map(|k| k.to_string()).collect(),
            allowed_kinds: ["pdf", "djvu", "epub", "fb2", "txt",
                            "xps", "oxps", "mobi", "cbz"].iter().map(|k| k.to_string()).collect(),
        }
    }
}

impl Default for BatterySettings {
    fn default() -> Self {
        BatterySettings {
            warn: 10.0,
            power_off: 3.0,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            selected_library: 0,
            libraries: vec![
                LibrarySettings {
                    name: "On Board".to_string(),
                    path: PathBuf::from(INTERNAL_CARD_ROOT),
                    hooks: vec![
                        Hook {
                            path: PathBuf::from("Articles"),
                            program: PathBuf::from("bin/article_fetcher/article_fetcher"),
                            sort_method: Some(SortMethod::Added),
                            first_column: Some(FirstColumn::TitleAndAuthor),
                            second_column: Some(SecondColumn::Progress),
                        }
                    ],
                    .. Default::default()
                },
                LibrarySettings {
                    name: "Removable".to_string(),
                    path: PathBuf::from(EXTERNAL_CARD_ROOT),
                    .. Default::default()
                },
                LibrarySettings {
                    name: "Dropbox".to_string(),
                    path: PathBuf::from("/mnt/onboard/.kobo/dropbox"),
                    .. Default::default()
                },
                LibrarySettings {
                    name: "KePub".to_string(),
                    path: PathBuf::from("/mnt/onboard/.kobo/kepub"),
                    .. Default::default()
                },
            ],
            external_urls_queue: Some(PathBuf::from("bin/article_fetcher/urls.txt")),
            keyboard_layout: "English".to_string(),
            frontlight: true,
            wifi: false,
            inverted: false,
            sleep_cover: true,
            auto_share: false,
            rotation_lock: None,
            button_scheme: ButtonScheme::Natural,
            auto_suspend: 30.0,
            auto_power_off: 3.0,
            time_format: "%H:%M".to_string(),
            date_format: "%A, %B %-d, %Y".to_string(),
            intermissions: Intermissions {
                suspend: PathBuf::from(LOGO_SPECIAL_PATH),
                power_off: PathBuf::from(LOGO_SPECIAL_PATH),
                share: PathBuf::from(LOGO_SPECIAL_PATH),
            },
            home: HomeSettings::default(),
            reader: ReaderSettings::default(),
            import: ImportSettings::default(),
            dictionary: DictionarySettings::default(),
            sketch: SketchSettings::default(),
            calculator: CalculatorSettings::default(),
            battery: BatterySettings::default(),
            frontlight_levels: LightLevels::default(),
            frontlight_presets: Vec::new(),
            ao3: Ao3Settings::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use reqwest::Url;
    use crate::helpers::{save_toml};

    #[test]
    #[allow(non_snake_case)]
    fn GIVEN_aSettingFileExists_WHEN_loadSettingsIsCalled_THEN_theSettingsAreLoadedFromTheFile() {
        // GIVEN a Settings file Exists
        let mut file_settings: Settings = Default::default();
        file_settings.ao3.faves.push(
            ("fake fave search".to_string(), Url::parse("https://fakeo3.org/tags/super-fake").expect("Test URL")));
        file_settings.ao3.username = Some("testUser".to_string());
        file_settings.ao3.password = Some("superFakePass123".to_string());
        let path = Path::new(SETTINGS_PATH);
        let _result = save_toml(&file_settings, path);

        // WHEN load_settings is called
        let settings = Settings::load_settings();

        // THEN the settings are loaded from the file
        // Checking the AO3 faves, username, and password, since that is what is currently set
        // by users in order to get anything to display on the home screen
        assert_eq!(settings.ao3.faves, file_settings.ao3.faves);
        assert_eq!(settings.ao3.username, Some("testUser".to_string()));
        assert_eq!(settings.ao3.password, Some("superFakePass123".to_string()));
    }

    #[test]
    #[allow(non_snake_case)]
    fn GIVEN_noSettingFileExists_WHEN_loadSettingsIsCalled_THEN_theDefaultSettingsAreLoaded() {
        // GIVEN no Settings file Exists
        let path = Path::new(SETTINGS_PATH);
        let _result = fs::remove_file(path);

        // WHEN load_settings is called
        let settings = Settings::load_settings();

        // THEN the default settings are loaded
        let default_settings: Settings = Default::default();
        // Checking the AO3 faves, username, and password, since that is what is currently set
        // by users in order to get anything to display on the home screen
        assert_eq!(settings.ao3.faves, default_settings.ao3.faves);
        assert_eq!(settings.ao3.username, default_settings.ao3.username);
        assert_eq!(settings.ao3.password, default_settings.ao3.password);
    }

    #[test]
    #[allow(non_snake_case)]
    fn WHEN_getCurrentLibraryIsCalled_THEN_theAppropriateLibraryIsCreated() {
        // GIVEN a Settings object with 4 libraries
        let mut default_settings: Settings = Default::default();
        default_settings.libraries.push({ LibrarySettings {
            name: "TEST_LIBRARY".to_string(),
            path: PathBuf::from("artworks"),
            .. Default::default()
        }});
        // AND the fifth library is selected
        default_settings.selected_library = 4;
        // WHEN get_current_library is called
        let library = default_settings.get_current_library();
        // THEN the appropriate library is created
        assert_eq!(library.home, PathBuf::from("artworks"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn GIVEN_anInvalidSelectedLibrary_WHEN_getCurrentLibraryIsCalled_THEN_theFirstLibraryIsCreated() {
        // GIVEN an invalid selected library
        let mut default_settings: Settings = Default::default();
        default_settings.libraries = Vec::new();
        default_settings.libraries.push({ LibrarySettings {
            name: "TEST_LIBRARY".to_string(),
            path: PathBuf::from("artworks"),
            .. Default::default()
        }});
        default_settings.libraries.push({ LibrarySettings {
            name: "TEST_LIBRARY_2".to_string(),
            path: PathBuf::from("icons"),
            .. Default::default()
        }});
        default_settings.selected_library = 5;
        // WHEN get_current_library is called
        let library = default_settings.get_current_library();
        // THEN the first library is created
        assert_eq!(library.home, PathBuf::from("artworks"));
    }

    #[test]
    #[allow(non_snake_case)]
    fn GIVEN_noLibrariesExist_WHEN_getCurrentLibraryIsCalled_THEN_theDefaultLibraryIsCreated() {
        // GIVEN an invalid selected library
        let mut default_settings: Settings = Default::default();
        default_settings.libraries = Vec::new();
        default_settings.selected_library = 0;
        // WHEN get_current_library is called
        let library = default_settings.get_current_library();
        // THEN the first library is created
        assert_eq!(library.home, PathBuf::from("/opt/ao3-reader/crates/core"));
    }
}
