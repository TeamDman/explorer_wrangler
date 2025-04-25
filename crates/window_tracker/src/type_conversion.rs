pub trait AsBevyIRect {
    fn as_bevy_irect(&self) -> bevy_math::prelude::IRect;
}
impl AsBevyIRect for windows::Win32::Foundation::RECT {
    fn as_bevy_irect(&self) -> bevy_math::prelude::IRect {
        bevy_math::prelude::IRect {
            min: bevy_math::prelude::IVec2::new(self.left, self.top),
            max: bevy_math::prelude::IVec2::new(self.right, self.bottom),
        }
    }
}
