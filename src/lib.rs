use std::{
    mem::size_of,
    sync::atomic::{AtomicUsize, Ordering},
};

use once_cell::sync::Lazy;
use windows_sys::Win32::{
    Foundation::*,
    Graphics::Dwm::*,
    System::LibraryLoader::{GetModuleHandleA, GetProcAddress},
    UI::Controls::MARGINS,
};

#[link(name = "ntdll")]
extern "system" {
    fn RtlGetVersion(os_ver_info: *mut OsVersionInfo) -> u32;
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct OsVersionInfo {
    os_version_info_size: u32,
    major_version: u32,
    minor_version: u32,
    build_number: u32,
    platform_id: u32,
    csd_version: [u16; 128],
}

fn build_no() -> u32 {
    let mut os_ver_info: OsVersionInfo = unsafe { std::mem::zeroed() };
    os_ver_info.os_version_info_size = size_of::<OsVersionInfo>() as _;
    unsafe { RtlGetVersion(&mut os_ver_info) };
    os_ver_info.build_number
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
#[allow(dead_code)]
enum WindowCompositionAttribute {
    Undefined = 0,
    NcrenderingEnabled = 1,
    NcrenderingPolicy = 2,
    TransitionsForcedisabled = 3,
    AllowNcpaint = 4,
    CaptionButtonBounds = 5,
    NonclientRtlLayout = 6,
    ForceIconicRepresentation = 7,
    ExtendedFrameBounds = 8,
    HasIconicBitmap = 9,
    ThemeAttributes = 10,
    NcrenderingExiled = 11,
    Ncadornmentinfo = 12,
    ExcludedFromLivepreview = 13,
    VideoOverlayActive = 14,
    ForceActivewindowAppearance = 15,
    DisallowPeek = 16,
    Cloak = 17,
    Cloaked = 18,
    AccentPolicy = 19,
    FreezeRepresentation = 20,
    EverUncloaked = 21,
    VisualOwner = 22,
    Holographic = 23,
    ExcludedFromDda = 24,
    Passiveupdatemode = 25,
    Usedarkmodecolors = 26,
    Last = 27,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct WindowCompositionAttributeData {
    attrib: WindowCompositionAttribute,
    pv_data: *mut std::ffi::c_void,
    cb_data: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
#[repr(C)]
enum AccentState {
    Disabled = 0,
    EnableGradient = 1,
    EnableTransparentgradient = 2,
    EnableBlurbehind = 3,
    EnableAcrylicblurbehind = 4,
    EnableHostbackdrop = 5,
    InvalidState = 6,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct AccentPolicy {
    accent_state: AccentState,
    accent_flags: u32,
    gradient_color: u32,
    animation_id: u32,
}

static SET_WINDOW_COMPOSITION_ATTRIBUTE: Lazy<
    extern "system" fn(
        hwnd: HWND,
        wnd_comp_attrib_data: *const WindowCompositionAttributeData,
    ) -> BOOL,
> = Lazy::new(|| unsafe {
    let user32 = GetModuleHandleA(b"user32.dll\0".as_ptr() as _);
    std::mem::transmute(GetProcAddress(
        user32,
        b"SetWindowCompositionAttribute\0".as_ptr() as _,
    ))
});

unsafe fn set_window_composition_attribute<T>(
    hwnd: HWND,
    attrib: WindowCompositionAttribute,
    data: T,
) -> bool {
    let window_composition_attribute = WindowCompositionAttributeData {
        attrib,
        pv_data: &data as *const _ as _,
        cb_data: size_of::<T>(),
    };
    (*SET_WINDOW_COMPOSITION_ATTRIBUTE)(hwnd, &window_composition_attribute) != 0
}
unsafe fn dwm_set_window_attribute<T>(hwnd: HWND, attrib: DWMWINDOWATTRIBUTE, data: T) -> bool {
    DwmSetWindowAttribute(hwnd, attrib, &data as *const _ as _, size_of::<T>() as _) == 0
}

static LAST_EFFECT: AtomicUsize = AtomicUsize::new(Effect::None as _);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum Effect {
    None,
    Solid,
    Transparent,
    Aero,
    Acrylic,
    Mica,
    Tabbed,
}

impl From<Effect> for AccentState {
    fn from(effect: Effect) -> Self {
        match effect {
            Effect::None => AccentState::Disabled,
            Effect::Solid => AccentState::EnableGradient,
            Effect::Transparent => AccentState::EnableTransparentgradient,
            Effect::Aero => AccentState::EnableBlurbehind,
            Effect::Acrylic => AccentState::EnableAcrylicblurbehind,
            Effect::Mica => AccentState::EnableHostbackdrop,
            Effect::Tabbed => AccentState::InvalidState,
        }
    }
}

const NEGATIVE_ONE_MARGIN: MARGINS = MARGINS {
    cxLeftWidth: -1,
    cxRightWidth: -1,
    cyBottomHeight: -1,
    cyTopHeight: -1,
};

const RESTORATIVE_MARGIN: MARGINS = MARGINS {
    cxLeftWidth: 0,
    cxRightWidth: 0,
    cyBottomHeight: 0,
    cyTopHeight: 1,
};

/// Freshly introduced in Windows 11 build 22523.
///
/// Allows *Tabbed* style.
///
/// Acrylic => 3i32
/// Mica => 2i32
/// Tabbed => 4i32
const DWMWA_NEW_BACKDROP_MODE: DWMWINDOWATTRIBUTE = 38;

/// Introduced in Windows 11 22000.
///
/// Takes [BOOL].
const DWMWA_USE_MICA_BACKDROP: DWMWINDOWATTRIBUTE = 1029;

/// Set window backdrop effect to the specified handle.
///
/// Color is in (R,G,B,A) format.
///
/// # Panics
///
/// Panics if non-mica effect is requested with no color specified.
///
/// # Safety
///
/// The handle `hwnd` must be valid.
pub unsafe fn set_effect(
    hwnd: HWND,
    effect: Effect,
    dark_mode: bool,
    color: Option<(u8, u8, u8, u8)>,
) {
    // Set [ACCENT_DISABLED] as [ACCENT_POLICY] in
    // [SetWindowCompositionAttribute] to apply styles properly.
    set_window_composition_attribute(
        hwnd,
        WindowCompositionAttribute::AccentPolicy,
        AccentPolicy {
            accent_state: AccentState::Disabled,
            accent_flags: 2,
            gradient_color: 0,
            animation_id: 0,
        },
    );

    let windows_build_no = build_no();

    // Only on later Windows 11 versions and if effect is WindowEffect.mica,
    // WindowEffect.acrylic or WindowEffect.tabbed, otherwise fallback to old
    // approach.
    if windows_build_no >= 22523 && effect > Effect::Aero {
        DwmExtendFrameIntoClientArea(hwnd, &NEGATIVE_ONE_MARGIN);
        dwm_set_window_attribute(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE, BOOL::from(dark_mode));
        dwm_set_window_attribute(
            hwnd,
            DWMWA_NEW_BACKDROP_MODE,
            match effect {
                Effect::Acrylic => 3i32,
                Effect::Mica => 2,
                Effect::Tabbed => 4,
                _ => unreachable!(),
            },
        );
    } else if effect == Effect::Mica {
        // Check for Windows 11.
        if windows_build_no >= 22000 {
            // Mica effect requires [DwmExtendFrameIntoClientArea & "sheet of
            // glass"
            // effect with negative margins.
            DwmExtendFrameIntoClientArea(hwnd, &NEGATIVE_ONE_MARGIN);
            dwm_set_window_attribute(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE, BOOL::from(dark_mode));
            dwm_set_window_attribute(hwnd, DWMWA_USE_MICA_BACKDROP, BOOL::from(true));
        }
    } else {
        // Restore original window style & [DwmExtendFrameIntoClientArea] margin
        // if the last set effect was [WindowEffect.mica], since it sets
        // negative margins to the window.
        if (windows_build_no >= 22000
            && LAST_EFFECT.load(Ordering::Relaxed) == Effect::Mica as usize)
            || (windows_build_no >= 22523
                && LAST_EFFECT.load(Ordering::Relaxed) > Effect::Aero as usize)
        {
            // Atleast one margin should be non-negative in order to show the DWM
            // window shadow created by handling [WM_NCCALCSIZE].
            //
            // Matching value with bitsdojo_window.
            // https://github.com/bitsdojo/bitsdojo_window/blob/adad0cd40be3d3e12df11d864f18a96a2d0fb4fb/bitsdojo_window_windows/windows/bitsdojo_window.cpp#L149
            DwmExtendFrameIntoClientArea(hwnd, &RESTORATIVE_MARGIN);
            dwm_set_window_attribute(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE, BOOL::from(false));
            dwm_set_window_attribute(hwnd, DWMWA_USE_MICA_BACKDROP, BOOL::from(false));
        }
        set_window_composition_attribute(
            hwnd,
            WindowCompositionAttribute::AccentPolicy,
            AccentPolicy {
                accent_state: AccentState::from(effect),
                accent_flags: 2,
                gradient_color: color
                    .map(|(r, g, b, a)| {
                        (a as u32) << 24 | (b as u32) << 16 | (g as u32) << 8 | r as u32
                    })
                    .unwrap(),
                animation_id: 0,
            },
        );
    }

    LAST_EFFECT.store(effect as _, Ordering::Relaxed);
}
