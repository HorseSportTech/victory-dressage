use crate::commands::alert_manager::AlertManager;
use crate::templates::{html_elements, GlobalAttributes};
use hypertext::{rsx_move, Lazy};

pub fn get_warnings<'a, 'b>(
    alert_manager: tauri::State<'a, AlertManager>,
) -> Lazy<impl Fn(&mut String) + 'a> {
    let warnings_length = alert_manager.get_length();
    rsx_move! {
        <dialog open class="warning-notifications">
            <div style="font-weight:bold; color:var(--theme); font-size:var(--text-input);">"Notifications"</div>
            {&alert_manager.fmt()}
        </dialog>
    }
}
