use bevy::prelude::*;
pub use egui_toast::*;

#[derive(Event)]
pub struct ToastMessage {
    pub text: String,
    pub kind: ToastKind,
}

impl ToastMessage {
    pub fn new(text: &str, kind: ToastKind) -> Self {
        Self {
            text: text.to_string(),
            kind,
        }
    }
}

impl From<&ToastMessage> for Toast {
    fn from(value: &ToastMessage) -> Self {
        let duration = toast_kind_to_duration(value);
        Self {
            text: value.text.clone().into(),
            kind: value.kind,
            options: ToastOptions::default()
                .show_icon(true)
                .duration_in_seconds(duration)
                .show_progress(false),
            style: default(),
        }
    }
}

const fn toast_kind_to_duration(value: &ToastMessage) -> f64 {
    match &value.kind {
        ToastKind::Warning => 6.,
        ToastKind::Error => 10.,
        _ => 4.,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_toast_message() {
        let message = ToastMessage::new("Test message", ToastKind::Info);
        assert_eq!(message.text, "Test message");
        assert_eq!(message.kind, ToastKind::Info);
    }

    #[test]
    fn from_toast_message() {
        let message = ToastMessage::new("Test message", ToastKind::Info);
        let toast = Toast::from(&message);
        assert_eq!(toast.text.text(), message.text);
        assert_eq!(toast.kind, message.kind);
        assert!(toast.options.show_icon);
        assert!(!toast.options.show_progress);
        assert_eq!(toast.options.progress(), 1.);
    }

    #[test]
    fn toast_kinds_to_durations() {
        assert_eq!(
            toast_kind_to_duration(&ToastMessage::new("Test message", ToastKind::Info)),
            4.
        );
        assert_eq!(
            toast_kind_to_duration(&ToastMessage::new("Test message", ToastKind::Warning)),
            6.
        );
        assert_eq!(
            toast_kind_to_duration(&ToastMessage::new("Test message", ToastKind::Error)),
            10.
        );
    }
}
