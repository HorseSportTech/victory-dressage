use hypertext::{html_elements, GlobalAttributes, Renderable as _};

use crate::commands::replace_director::ReplaceDirector;

use super::TxAttributes;

pub fn screen_error(msg: &str) -> ReplaceDirector {
	return ReplaceDirector::with_target("html > body", hypertext::rsx!{
		<div
			id="error"
			style="width: 100vw; height:100vh; overflow:hidden; 
				text-align:center; padding-top: 5rem;
				background: hsl(0, 100%, 21%); color:white;
				box-sizing:border-box; overflow:hidden;"
		>
			<h1>"An unexpected error occured!"</h1>
			<h2>"The app encountered an error from which it could not automatically recover"</h2>
			<p>"The reason was:"</p>
			<p>{msg}</p>
			<br>
			<p>"You may press ‘attempt recovery’ below to reload back to a previous page."<br>"If this error appears more than once, please do not try again."</p>
			<button tx-command="recover"
				style="font-size:var(--text-input); background:var(--theme); color:white; padding: 0.5rem 1rem;
				border:none; border-radius:var(--corner-size)"
			>Recover</button>
		</div>
	}.render())
}
