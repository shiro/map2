use crate::*;

pub fn print_debug(msg: impl AsRef<str>) {
    println!("[DEBUG] {}", msg.as_ref());
}

pub fn print_input_event(ev: &EvdevInputEvent) -> String {
    match ev.event_code {
        EventCode::EV_SYN(_) => ev.event_type().unwrap().to_string(),
        _ => format!(
            "Event: type: {}, code: {}, value: {}",
            ev.event_type().map(|ev_type| format!("{}", ev_type)).unwrap_or("None".to_string()),
            ev.event_code,
            ev.value,
        ),
    }
}
