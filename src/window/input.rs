use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_CONTROL;

pub fn ctrl_held() -> bool {
    unsafe { GetKeyState(VK_CONTROL.0 as i32) & 0x8000 != 0 }
}
