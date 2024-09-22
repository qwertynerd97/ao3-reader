use crate::view::keyboard::Layout;
use std::path::Path;
use std::collections::{BTreeMap, VecDeque};
use fxhash::FxHashMap;
use chrono::Local;
use globset::Glob;
use walkdir::WalkDir;
use rand_core::SeedableRng;
use rand_xoshiro::Xoroshiro128Plus;

use crate::battery::{Battery, KoboBattery, FakeBattery};
use crate::device::{CURRENT_DEVICE, FrontlightKind};
use crate::dictionary::{Dictionary, load_dictionary_from_file};
use crate::framebuffer::{Framebuffer, Display};
use crate::frontlight::{Frontlight, StandardFrontlight, NaturalFrontlight, PremixedFrontlight, LightLevels};
use crate::lightsensor::{LightSensor, KoboLightSensor};
use crate::view::ViewId;
use crate::helpers::{load_json, IsHidden};
use crate::settings::Settings;
use crate::geom::Rectangle;
use crate::library::Library;
use crate::font::Fonts;
use crate::rtc::Rtc;
use crate::http::HttpClient;

const KEYBOARD_LAYOUTS_DIRNAME: &str = "keyboard-layouts";
const DICTIONARIES_DIRNAME: &str = "dictionaries";
const INPUT_HISTORY_SIZE: usize = 32;

const RTC_DEVICE: &str = "/dev/rtc0";


pub struct Context {
    pub fb: Box<dyn Framebuffer>,
    pub rtc: Option<Rtc>,
    pub display: Display,
    pub settings: Settings,
    pub library: Library,
    pub fonts: Fonts,
    pub dictionaries: BTreeMap<String, Dictionary>,
    pub keyboard_layouts: BTreeMap<String, Layout>,
    pub input_history: FxHashMap<ViewId, VecDeque<String>>,
    pub frontlight: Box<dyn Frontlight>,
    pub battery: Box<dyn Battery>,
    pub lightsensor: Box<dyn LightSensor>,
    pub notification_index: u8,
    pub kb_rect: Rectangle,
    pub rng: Xoroshiro128Plus,
    pub plugged: bool,
    pub covered: bool,
    pub shared: bool,
    pub online: bool,
    pub client: HttpClient,
}

impl Context {
    pub fn new_from_virtual(fb: Box<dyn Framebuffer>) -> Context {
        let battery = Box::new(FakeBattery::new()) as Box<dyn Battery>;
        let frontlight = Box::new(LightLevels::default()) as Box<dyn Frontlight>;
        let lightsensor = Box::new(0u16) as Box<dyn LightSensor>;

        Context::new(fb, None, None, battery, frontlight, lightsensor)
    }

    pub fn new_from_kobo(fb: Box<dyn Framebuffer>) -> Context {
        let settings = Settings::load_settings();

        let rtc = Rtc::new(RTC_DEVICE)
            .map_err(|e| eprintln!("Can't open RTC device: {:#}.", e))
            .ok();

        let battery = Box::new(KoboBattery::new()
            .map_err(|e| eprintln!("Can't create battery: {:#}.", e))
            .unwrap()) as Box<dyn Battery>;

        let lightsensor = if CURRENT_DEVICE.has_lightsensor() {
            Box::new(KoboLightSensor::new()
                .map_err(|e| eprintln!("Can't create light sensor: {:#}.", e))
                .unwrap()) as Box<dyn LightSensor>
        } else {
            Box::new(0u16) as Box<dyn LightSensor>
        };

        let levels = settings.frontlight_levels;
        let frontlight = match CURRENT_DEVICE.frontlight_kind() {
            FrontlightKind::Standard => Box::new(StandardFrontlight::new(levels.intensity)
                .map_err(|e| eprintln!("Can't create standard frontlight: {:#}.", e))
                .unwrap()) as Box<dyn Frontlight>,
            FrontlightKind::Natural => Box::new(NaturalFrontlight::new(levels.intensity, levels.warmth)
                .map_err(|e| eprintln!("Can't create natural frontlight: {:#}.", e))
                .unwrap()) as Box<dyn Frontlight>,
            FrontlightKind::Premixed => Box::new(PremixedFrontlight::new(levels.intensity, levels.warmth)
                .map_err(|e| eprintln!("Can't create premixed frontlight: {:#}.", e))
                .unwrap()) as Box<dyn Frontlight>,
        };

        Context::new(fb, rtc, Some(settings), battery, frontlight, lightsensor)
    }

    pub fn new(fb: Box<dyn Framebuffer>, rtc: Option<Rtc>, opt_settings: Option<Settings>, battery: Box<dyn Battery>,
               frontlight: Box<dyn Frontlight>, lightsensor: Box<dyn LightSensor>) -> Context {
        let mut settings = match opt_settings {
            Some(existing_settings) => existing_settings,
            None => Settings::load_settings()
        };
        let fonts = Fonts::load()
            .map_err(|e| eprintln!("Can't load fonts: {:#}.", e))
            .unwrap();
        let library = settings.get_current_library();
        let dims = fb.dims();
        let rotation = CURRENT_DEVICE.transformed_rotation(fb.rotation());
        let rng = Xoroshiro128Plus::seed_from_u64(Local::now().timestamp_subsec_nanos() as u64);
        let client = HttpClient::new(&mut settings);
        Context {
            fb,
            rtc,
            display: Display { dims, rotation },
            library,
            settings,
            fonts,
            dictionaries: BTreeMap::new(),
            keyboard_layouts: BTreeMap::new(),
            input_history: FxHashMap::default(),
            battery,
            frontlight,
            lightsensor,
            notification_index: 0,
            kb_rect: Rectangle::default(),
            rng,
            plugged: false,
            covered: false,
            shared: false,
            online: false,
            client: client
        }
    }

    pub fn batch_import(&mut self) {
        self.library.import(&self.settings.import);
        let selected_library = self.settings.selected_library;
        for (index, library_settings) in self.settings.libraries.iter().enumerate() {
            if index == selected_library {
                continue;
            }
            if let Ok(mut library) = Library::new(&library_settings.path, library_settings.mode)
                                             .map_err(|e| eprintln!("{:#?}", e)) {
                library.import(&self.settings.import);
                library.flush();
            }
        }
    }

    pub fn load_keyboard_layouts(&mut self) {
        let glob = Glob::new("**/*.json").unwrap().compile_matcher();
        for entry in WalkDir::new(Path::new(KEYBOARD_LAYOUTS_DIRNAME)).min_depth(1)
                             .into_iter().filter_entry(|e| !e.is_hidden()) {
            if entry.is_err() {
                continue;
            }
            let entry = entry.unwrap();
            let path = entry.path();
            if !glob.is_match(path) {
                continue;
            }
            if let Ok(layout) = load_json::<Layout, _>(path)
                                          .map_err(|e| eprintln!("Can't load {}: {:#?}.", path.display(), e)) {
                self.keyboard_layouts.insert(layout.name.clone(), layout);
            }
        }
    }

    pub fn load_dictionaries(&mut self) {
        let glob = Glob::new("**/*.index").unwrap().compile_matcher();
        for entry in WalkDir::new(Path::new(DICTIONARIES_DIRNAME)).min_depth(1)
                             .into_iter().filter_entry(|e| !e.is_hidden()) {
            if entry.is_err() {
                continue;
            }
            let entry = entry.unwrap();
            if !glob.is_match(entry.path()) {
                continue;
            }
            let index_path = entry.path().to_path_buf();
            let mut content_path = index_path.clone();
            content_path.set_extension("dict.dz");
            if !content_path.exists() {
                content_path.set_extension("");
            }
            if let Ok(mut dict) = load_dictionary_from_file(&content_path, &index_path) {
                let name = dict.short_name().ok().unwrap_or_else(|| {
                    index_path.file_stem()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_default()
                });
                self.dictionaries.insert(name, dict);
            }
        }
    }

    pub fn record_input(&mut self, text: &str, id: ViewId) {
        if text.is_empty() {
            return;
        }

        let history = self.input_history.entry(id)
                          .or_insert_with(VecDeque::new);

        if history.front().map(String::as_str) != Some(text) {
            history.push_front(text.to_string());
        }

        if history.len() > INPUT_HISTORY_SIZE {
            history.pop_back();
        }
    }

    pub fn set_frontlight(&mut self, enable: bool) {
        self.settings.frontlight = enable;

        if enable {
            let levels = self.settings.frontlight_levels;
            self.frontlight.set_warmth(levels.warmth);
            self.frontlight.set_intensity(levels.intensity);
        } else {
            self.settings.frontlight_levels = self.frontlight.levels();
            self.frontlight.set_intensity(0.0);
            self.frontlight.set_warmth(0.0);
        }
    }
}


