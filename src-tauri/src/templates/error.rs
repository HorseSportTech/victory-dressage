use hypertext::{html_elements, GlobalAttributes, Renderable as _};

use crate::commands::replace_director::{PageLocation, ReplaceDirector};

use super::TxAttributes;

pub fn screen_error(msg: &str) -> ReplaceDirector {
    ReplaceDirector::with_target(&PageLocation::Any("html > body > content#application".to_string()), hypertext::rsx!{
		<div id="error">
			<h1>"An unexpected error occured!"</h1>
			<h2>"The app encountered an error from which it could not automatically recover"</h2>
			<p>"The reason was:"</p>
			<p>{msg}</p>
			<br>
			<p>"You may press ‘attempt recovery’ below to reload back to a previous page."<br>"If this error appears more than once, please do not try again."</p>
			<button
                tx-command="recover"
				style="font-size:var(--text-input); background:var(--theme); color:white; padding: 0.5rem 1rem;
				border:none; border-radius:var(--corner-size)"
			>Recover</button>
		</div>
	}.render())
}
