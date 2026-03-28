use crate::app::AppState;
use crate::renderer::Renderer;

// ── constants ─────────────────────────────
pub const WIN_WIDTH_RATIO: f32 = 0.40;
pub const WIN_TOP_RATIO: f32 = 0.08;

pub const VK_BACK: u32 = 0x08;
pub const VK_ESCAPE: u32 = 0x1B;
pub const VK_RETURN: u32 = 0x0D;
pub const VK_UP: u32 = 0x26;
pub const VK_DOWN: u32 = 0x28;
pub const VK_LEFT: u32 = 0x25;
pub const VK_RIGHT: u32 = 0x27;
pub const VK_A: u32 = 0x41;
pub const VK_C: u32 = 0x43;
pub const VK_V: u32 = 0x56;
pub const VK_X: u32 = 0x58;

pub const HOTKEY_ID: i32 = 1;
pub const CARET_TIMER_ID: usize = 2;
pub const CARET_BLINK_MS: u32 = 530;

// ── window state ──────────────────────────
pub struct WindowState {
    pub renderer: Option<Renderer>,
    pub app: AppState,
    pub win_w: i32,
    pub caret_visible: bool,
}
