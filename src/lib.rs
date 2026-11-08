// SPDX-License-Identifier: Apache-2.0
//! SigmaOS-plymouth-themes — SigmaOS plymouth-themes component inspired by Linux Mint / Omarchy / Arch Linux.
//!
//! Theme/asset registry: name -> resource path with uniqueness and non-empty validation.
//! Principles: single responsibility, fail-fast, defensive programming,
//! component isolation, stable API, deterministic tests.

use std::fmt;
use std::collections::BTreeMap;

/// Errors for `AssetError` operations. Fail-fast: invalid input is rejected.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssetError {
    /// Asset name must not be empty
    EmptyName,
    /// Asset path must not be empty
    EmptyPath(String),
    /// Asset name already registered
    Duplicate(String),
    /// Asset not registered
    NotFound(String),
}

impl fmt::Display for AssetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyName => write!(f, "asset name must not be empty"),
            Self::EmptyPath(x) => write!(f, "asset path must not be empty: {x}"),
            Self::Duplicate(x) => write!(f, "asset name already registered: {x}"),
            Self::NotFound(x) => write!(f, "asset not registered: {x}"),
        }
    }
}

impl std::error::Error for AssetError {}

/// Registry mapping asset names to resource paths.
#[derive(Debug, Default)]
pub struct AssetRegistry {
    assets: BTreeMap<String, String>,
}

impl AssetRegistry {
    /// Empty registry.
    pub fn new() -> Self { Self::default() }

    /// Register an asset; fails on empty name/path or duplicate name.
    pub fn register(&mut self, name: impl Into<String>, path: impl Into<String>) -> Result<(), AssetError> {
        let name = name.into();
        let path = path.into();
        if name.trim().is_empty() { return Err(AssetError::EmptyName); }
        if path.trim().is_empty() { return Err(AssetError::EmptyPath(name)); }
        if self.assets.contains_key(&name) { return Err(AssetError::Duplicate(name)); }
        self.assets.insert(name, path);
        Ok(())
    }

    /// Resolve an asset name to its path.
    pub fn resolve(&self, name: &str) -> Result<&str, AssetError> {
        self.assets.get(name).map(String::as_str)
            .ok_or_else(|| AssetError::NotFound(name.to_string()))
    }

    /// Number of assets.
    pub fn len(&self) -> usize { self.assets.len() }

    /// Whether empty.
    pub fn is_empty(&self) -> bool { self.assets.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_resolve() {
        let mut r = AssetRegistry::new();
        r.register("wallpaper", "/usr/share/wall.png").unwrap();
        assert_eq!(r.resolve("wallpaper").unwrap(), "/usr/share/wall.png");
    }

    #[test]
    fn duplicate_fails() {
        let mut r = AssetRegistry::new();
        r.register("a", "/p").unwrap();
        assert!(matches!(r.register("a", "/q"), Err(AssetError::Duplicate(_))));
    }

    #[test]
    fn empty_path_fails() {
        let mut r = AssetRegistry::new();
        assert!(matches!(r.register("a", " "), Err(AssetError::EmptyPath(_))));
    }

    #[test]
    fn resolve_missing_fails() {
        assert!(matches!(AssetRegistry::new().resolve("x"), Err(AssetError::NotFound(_))));
    }

}

/// Backwards-compatible component handle (kept for existing callers/tests).
#[derive(Debug, Default)]
pub struct Component {
    name: &'static str,
}

impl Component {
    /// Create the component handle.
    pub fn new() -> Self {
        Self { name: "SigmaOS Component" }
    }
}

// ─── Plymouth theme domain model ──────────────────────────────────────

/// Plymouth theme
#[derive(Debug, Clone)]
pub struct PlymouthTheme {
    pub name: String,
    pub description: String,
    pub background_image: Option<String>,
    pub background_color: String,
    pub progress_bar_color: String,
    pub progress_bar_background: String,
    pub font_family: String,
    pub font_size: u32,
    pub text_color: String,
    pub logo_image: Option<String>,
    pub animation: PlymouthAnimation,
    pub fade_in: bool,
    pub fade_out: bool,
}

/// Plymouth animation style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlymouthAnimation {
    /// No animation
    None,
    /// Fade in/out
    Fade,
    /// Slide
    Slide,
    /// Spin
    Spin,
    /// Pulse
    Pulse,
}

impl PlymouthTheme {
    /// Create new theme
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            background_image: None,
            background_color: "#000000".into(),
            progress_bar_color: "#ffffff".into(),
            progress_bar_background: "#333333".into(),
            font_family: "Sans".into(),
            font_size: 12,
            text_color: "#ffffff".into(),
            logo_image: None,
            animation: PlymouthAnimation::Fade,
            fade_in: true,
            fade_out: true,
        }
    }

    /// Set background
    pub fn set_background(&mut self, image: Option<&str>, color: &str) {
        self.background_image = image.map(|s| s.into());
        self.background_color = color.into();
    }

    /// Set progress bar colors
    pub fn set_progress_colors(&mut self, fg: &str, bg: &str) {
        self.progress_bar_color = fg.into();
        self.progress_bar_background = bg.into();
    }

    /// Set font
    pub fn set_font(&mut self, family: &str, size: u32) {
        self.font_family = family.into();
        self.font_size = size;
    }

    /// Set animation
    pub fn set_animation(&mut self, anim: PlymouthAnimation) {
        self.animation = anim;
    }

    /// Set logo
    pub fn set_logo(&mut self, image: Option<&str>) {
        self.logo_image = image.map(|s| s.into());
    }

    /// Serialize to plymouth theme script
    pub fn to_script(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!("# Plymouth theme: {}\n", self.name));
        if let Some(bg) = &self.background_image {
            out.push_str(&format!("background_image=\"{}\"\n", bg));
        }
        out.push_str(&format!("background_color={}\n", self.background_color));
        out.push_str(&format!("progress_bar_color={}\n", self.progress_bar_color));
        out.push_str(&format!("progress_bar_background={}\n", self.progress_bar_background));
        out.push_str(&format!("font_family=\"{}\"\n", self.font_family));
        out.push_str(&format!("font_size={}\n", self.font_size));
        out.push_str(&format!("text_color={}\n", self.text_color));
        out.push_str(&format!("animation={:?}\n", self.animation));
        if let Some(logo) = &self.logo_image {
            out.push_str(&format!("logo_image=\"{}\"\n", logo));
        }
        out
    }
}

/// Plymouth theme registry
#[derive(Debug, Default)]
pub struct PlymouthThemeRegistry {
    themes: Vec<PlymouthTheme>,
    active: Option<String>,
}

impl PlymouthThemeRegistry {
    pub fn new() -> Self { Self::default() }

    pub fn register(&mut self, theme: PlymouthTheme) -> bool {
        if self.themes.iter().any(|t| t.name == theme.name) { return false; }
        self.themes.push(theme);
        true
    }

    pub fn activate(&mut self, name: &str) -> bool {
        if self.themes.iter().any(|t| t.name == name) {
            self.active = Some(name.into());
            return true;
        }
        false
    }

    pub fn active(&self) -> Option<&PlymouthTheme> {
        self.active.as_ref().and_then(|n| self.themes.iter().find(|t| &t.name == n))
    }

    pub fn list(&self) -> &[PlymouthTheme] { &self.themes }

    /// Default themes
    pub fn with_defaults() -> Self {
        let mut reg = Self::new();
        
        let mut sigma = PlymouthTheme::new("sigma");
        sigma.set_background(Some("/usr/share/plymouth/themes/sigma/background.png"), "#000000");
        sigma.set_progress_colors("#7c3aed", "#1a1a2e");
        sigma.set_logo(Some("/usr/share/plymouth/themes/sigma/logo.png"));
        reg.register(sigma);
        
        reg.register(PlymouthTheme::new("spinner"));
        reg.register(PlymouthTheme::new("fade-in"));
        reg.activate("sigma");
        reg
    }
}

#[cfg(test)]
mod plymouth_tests {
    use super::*;

    #[test]
    fn test_theme_creation() {
        let theme = PlymouthTheme::new("test");
        assert_eq!(theme.name, "test");
        assert_eq!(theme.animation, PlymouthAnimation::Fade);
        assert_eq!(theme.background_color, "#000000");
        assert!(theme.fade_in);
        assert!(theme.fade_out);
    }

    #[test]
    fn test_animation_variants() {
        let anims = vec![
            PlymouthAnimation::None, PlymouthAnimation::Fade,
            PlymouthAnimation::Slide, PlymouthAnimation::Spin,
            PlymouthAnimation::Pulse,
        ];
        assert_eq!(anims.len(), 5);
    }

    #[test]
    fn test_set_background() {
        let mut theme = PlymouthTheme::new("t");
        theme.set_background(Some("/bg.png"), "#111");
        assert_eq!(theme.background_image, Some("/bg.png".to_string()));
        assert_eq!(theme.background_color, "#111");
    }

    #[test]
    fn test_set_colors() {
        let mut theme = PlymouthTheme::new("t");
        theme.set_progress_colors("#fff", "#333");
        assert_eq!(theme.progress_bar_color, "#fff");
        assert_eq!(theme.progress_bar_background, "#333");
    }

    #[test]
    fn test_animation_set() {
        let mut theme = PlymouthTheme::new("t");
        theme.set_animation(PlymouthAnimation::Spin);
        assert_eq!(theme.animation, PlymouthAnimation::Spin);
    }

    #[test]
    fn test_to_script() {
        let mut theme = PlymouthTheme::new("cfg");
        theme.set_background(Some("/bg.png"), "#000");
        let script = theme.to_script();
        assert!(script.contains("Plymouth theme: cfg"));
        assert!(script.contains("background_image"));
        assert!(script.contains("animation=Fade"));
    }

    #[test]
    fn test_registry() {
        let mut reg = PlymouthThemeRegistry::new();
        assert!(reg.register(PlymouthTheme::new("a")));
        assert!(reg.activate("a"));
        assert_eq!(reg.active().unwrap().name, "a");
        assert_eq!(PlymouthThemeRegistry::with_defaults().list().len(), 3);
    }

    #[test]
    fn test_with_defaults() {
        let reg = PlymouthThemeRegistry::with_defaults();
        let active = reg.active().unwrap();
        assert_eq!(active.name, "sigma");
        assert!(active.background_image.is_some());
        assert!(active.logo_image.is_some());
    }
}
