use core::{
    fmt::{self, Display, Formatter},
    hash::Hash,
};

use bevy::prelude::*;
use bitflags::bitflags;
use serde::{Deserialize, Serialize};

/// Wraps input from any source.
///
/// [Input modifiers](crate::input_context::input_modifier) can change the captured dimension.
///
/// If the action's dimension differs from the captured input, it will be converted using
/// [`ActionValue::convert`](crate::action_value::ActionValue::convert).
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Reflect, PartialEq)]
pub enum Input {
    /// Keyboard button, will be captured as
    /// [`ActionValue::Bool`](crate::action_value::ActionValue::Bool).
    Keyboard { key: KeyCode, mod_keys: ModKeys },
    /// Mouse button, will be captured as
    /// [`ActionValue::Bool`](crate::action_value::ActionValue::Bool).
    MouseButton {
        button: MouseButton,
        mod_keys: ModKeys,
    },
    /// Mouse movement, will be captured as
    /// [`ActionValue::Axis2D`](crate::action_value::ActionValue::Axis2D).
    MouseMotion { mod_keys: ModKeys },
    /// Mouse wheel, will be captured as
    /// [`ActionValue::Axis2D`](crate::action_value::ActionValue::Axis2D).
    MouseWheel { mod_keys: ModKeys },
    /// Gamepad button, will be captured as
    /// [`ActionValue::Axis1D`](crate::action_value::ActionValue::Axis1D).
    GamepadButton(GamepadButton),
    /// Gamepad stick axis, will be captured as
    /// [`ActionValue::Axis1D`](crate::action_value::ActionValue::Axis1D).
    GamepadAxis(GamepadAxis),
}

impl Input {
    /// Returns [`Input::MouseMotion`] without keyboard modifiers.
    #[must_use]
    pub const fn mouse_motion() -> Self {
        Self::MouseMotion {
            mod_keys: ModKeys::empty(),
        }
    }

    /// Returns [`Input::MouseWheel`] without keyboard modifiers.
    #[must_use]
    pub const fn mouse_wheel() -> Self {
        Self::MouseWheel {
            mod_keys: ModKeys::empty(),
        }
    }

    /// Returns the amount of associated keyboard modifiers.
    #[must_use]
    pub fn mod_keys_count(self) -> usize {
        self.mod_keys().iter_names().count()
    }

    /// Returns associated keyboard modifiers.
    #[must_use]
    pub const fn mod_keys(self) -> ModKeys {
        match self {
            Input::Keyboard { mod_keys, .. }
            | Input::MouseButton { mod_keys, .. }
            | Input::MouseMotion { mod_keys }
            | Input::MouseWheel { mod_keys } => mod_keys,
            Input::GamepadButton(_) | Input::GamepadAxis(_) => ModKeys::empty(),
        }
    }

    /// Returns new instance without any keyboard modifiers.
    ///
    /// # Panics
    ///
    /// Panics when called on [`Self::GamepadButton`] or [`Self::GamepadAxis`].
    #[must_use]
    pub fn without_mod_keys(self) -> Self {
        self.with_mod_keys(ModKeys::empty())
    }
}

impl Display for Input {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mod_keys = self.mod_keys();
        if !mod_keys.is_empty() {
            write!(f, "{mod_keys} + ")?;
        }

        match self {
            Input::Keyboard { key, .. } => write!(f, "{key:?}"),
            Input::MouseButton { button, .. } => write!(f, "Mouse {button:?}"),
            Input::MouseMotion { .. } => write!(f, "Mouse Motion"),
            Input::MouseWheel { .. } => write!(f, "Scroll Wheel"),
            Input::GamepadButton(gamepad_button) => write!(f, "{gamepad_button:?}"),
            Input::GamepadAxis(gamepad_axis) => write!(f, "{gamepad_axis:?}"),
        }
    }
}

impl From<KeyCode> for Input {
    fn from(key: KeyCode) -> Self {
        Self::Keyboard {
            key,
            mod_keys: Default::default(),
        }
    }
}

impl From<MouseButton> for Input {
    fn from(button: MouseButton) -> Self {
        Self::MouseButton {
            button,
            mod_keys: Default::default(),
        }
    }
}

impl From<GamepadButton> for Input {
    fn from(value: GamepadButton) -> Self {
        Self::GamepadButton(value)
    }
}

impl From<GamepadAxis> for Input {
    fn from(value: GamepadAxis) -> Self {
        Self::GamepadAxis(value)
    }
}

/// A trait to ergonomically assign keyboard modifiers to any type that can be converted into an input.
pub trait InputModKeys {
    /// Returns an input with assigned keyboard modifiers.
    #[must_use]
    fn with_mod_keys(self, mod_keys: ModKeys) -> Input;
}

impl<I: Into<Input>> InputModKeys for I {
    /// Returns new instance with the replaced keyboard modifiers.
    ///
    /// # Panics
    ///
    /// Panics when called on [`Input::GamepadButton`] or [`Input::GamepadAxis`].
    fn with_mod_keys(self, mod_keys: ModKeys) -> Input {
        match self.into() {
            Input::Keyboard { key, .. } => Input::Keyboard { key, mod_keys },
            Input::MouseButton { button, .. } => Input::MouseButton { button, mod_keys },
            Input::MouseMotion { .. } => Input::MouseMotion { mod_keys },
            Input::MouseWheel { .. } => Input::MouseWheel { mod_keys },
            Input::GamepadButton { .. } | Input::GamepadAxis { .. } => {
                panic!("keyboard modifiers can't be applied to gamepads")
            }
        }
    }
}

/// Keyboard modifiers for both left and right keys.
#[derive(Default, Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Reflect)]
pub struct ModKeys(u8);

bitflags! {
    impl ModKeys: u8 {
        /// Corresponds to [`KeyCode::ControlLeft`] and [`KeyCode::ControlRight`].
        const CONTROL = 0b00000001;
        /// Corresponds to [`KeyCode::ShiftLeft`] and [`KeyCode::ShiftRight`]
        const SHIFT = 0b00000010;
        /// Corresponds to [`KeyCode::AltLeft`] and [`KeyCode::AltRight`].
        const ALT = 0b00000100;
        /// Corresponds to [`KeyCode::SuperLeft`] and [`KeyCode::SuperRight`].
        const SUPER = 0b00001000;
    }
}

impl Display for ModKeys {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        for (index, (_, mod_key)) in self.iter_names().enumerate() {
            if index != 0 {
                write!(f, " + ")?;
            }
            match mod_key {
                ModKeys::CONTROL => write!(f, "Ctrl")?,
                ModKeys::SHIFT => write!(f, "Shift")?,
                ModKeys::ALT => write!(f, "Alt")?,
                ModKeys::SUPER => write!(f, "Super")?,
                _ => unreachable!("iteration should yield only named flags"),
            }
        }

        Ok(())
    }
}

impl ModKeys {
    /// Returns an instance with currently active modifiers.
    #[must_use]
    pub fn pressed(keys: &ButtonInput<KeyCode>) -> Self {
        let mut mod_keys = Self::empty();
        for [key1, key2] in Self::all().iter_keys() {
            if keys.any_pressed([key1, key2]) {
                mod_keys |= key1.into();
            }
        }

        mod_keys
    }

    /// Returns an iterator over the key codes corresponding to the set modifier bits.
    ///
    /// Each item contains left and right key codes.
    pub fn iter_keys(self) -> impl Iterator<Item = [KeyCode; 2]> {
        self.iter_names().map(|(_, mod_key)| match mod_key {
            ModKeys::CONTROL => [KeyCode::ControlLeft, KeyCode::ControlRight],
            ModKeys::SHIFT => [KeyCode::ShiftLeft, KeyCode::ShiftRight],
            ModKeys::ALT => [KeyCode::AltLeft, KeyCode::AltRight],
            ModKeys::SUPER => [KeyCode::SuperLeft, KeyCode::SuperRight],
            _ => unreachable!("iteration should yield only named flags"),
        })
    }
}

impl From<KeyCode> for ModKeys {
    /// Converts key into a named modifier
    ///
    /// Returns [`ModKeys::empty`] if the key is not a modifier.
    fn from(value: KeyCode) -> Self {
        match value {
            KeyCode::ControlLeft | KeyCode::ControlRight => ModKeys::CONTROL,
            KeyCode::ShiftLeft | KeyCode::ShiftRight => ModKeys::SHIFT,
            KeyCode::AltLeft | KeyCode::AltRight => ModKeys::ALT,
            KeyCode::SuperLeft | KeyCode::SuperRight => ModKeys::SUPER,
            _ => ModKeys::empty(),
        }
    }
}

/// Associated gamepad.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default, Hash, PartialEq, Eq)]
pub enum GamepadDevice {
    /// Matches input from any gamepad.
    ///
    /// For an axis, the [`ActionValue`] will be calculated as the sum of inputs from all gamepads.
    /// For a button, the [`ActionValue`] will be `true` if any gamepad has this button pressed.
    ///
    /// [`ActionValue`]: crate::action_value::ActionValue
    #[default]
    Any,
    /// Matches input from specific gamepad.
    Single(Entity),
}

impl From<Entity> for GamepadDevice {
    fn from(value: Entity) -> Self {
        Self::Single(value)
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use super::*;

    #[test]
    fn pressed_mod_keys() {
        let mut keys = ButtonInput::default();
        keys.press(KeyCode::ControlLeft);
        keys.press(KeyCode::ShiftLeft);
        keys.press(KeyCode::KeyC);

        let mod_keys = ModKeys::pressed(&keys);
        assert_eq!(mod_keys, ModKeys::CONTROL | ModKeys::SHIFT);
    }

    #[test]
    fn mod_keys_display() {
        assert_eq!(ModKeys::CONTROL.to_string(), "Ctrl");
        assert_eq!(ModKeys::all().to_string(), "Ctrl + Shift + Alt + Super");
        assert_eq!(ModKeys::empty().to_string(), "");
    }

    #[test]
    fn input_display() {
        assert_eq!(
            Input::Keyboard {
                key: KeyCode::KeyA,
                mod_keys: ModKeys::empty()
            }
            .to_string(),
            "KeyA"
        );
        assert_eq!(
            Input::Keyboard {
                key: KeyCode::KeyA,
                mod_keys: ModKeys::CONTROL
            }
            .to_string(),
            "Ctrl + KeyA"
        );
        assert_eq!(
            Input::MouseButton {
                button: MouseButton::Left,
                mod_keys: ModKeys::empty()
            }
            .to_string(),
            "Mouse Left"
        );
        assert_eq!(
            Input::MouseMotion {
                mod_keys: ModKeys::empty()
            }
            .to_string(),
            "Mouse Motion"
        );
        assert_eq!(
            Input::MouseWheel {
                mod_keys: ModKeys::empty()
            }
            .to_string(),
            "Scroll Wheel"
        );
        assert_eq!(
            Input::GamepadAxis(GamepadAxis::LeftStickX).to_string(),
            "LeftStickX"
        );
        assert_eq!(
            Input::GamepadButton(GamepadButton::North).to_string(),
            "North"
        );
    }
}
