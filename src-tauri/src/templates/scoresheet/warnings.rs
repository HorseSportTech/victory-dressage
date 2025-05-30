use crate::commands::alert_manager::AlertManager;
use crate::templates::{html_elements, GlobalAttributes};
use hypertext::{rsx_move, Lazy};

pub fn get_warnings<'a, 'b>(
    alert_manager: tauri::State<'a, AlertManager>,
) -> Lazy<impl Fn(&mut String) + 'a> {
    let warnings_length = alert_manager.get_length();
    rsx_move! {
        @if warnings_length > 0 {
            <dialog open style="border-radius:var(--corner-size); border:0.1rem solid var(--theme); background:#fffd; inline-size:17rem">
                <div style="font-weight:bold; color:var(--theme); font-size:var(--text-input);">"Notifications"</div>
                {&alert_manager.fmt()}
                <style>"#alerts-and-warnings { & .flash {font-weight:bold;animation: flashing 300ms alternate-reverse infinite ease;}}"</style>
            </dialog>
        }
    }
}


    
          
              
              
              
