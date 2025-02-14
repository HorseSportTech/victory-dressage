use hypertext::{rsx, GlobalAttributes, Renderable, Rendered};

use crate::templates::{html_elements, TxAttributes};

pub fn logout_button() -> Rendered<String> {
	rsx!{<button tx-command="log_out"
		style="background: hsl(0, 100%, 20%); border-radius: 0 0 var(--corner-size) var(--corner-size); color: white;
		position:absolute; left:1rem; top:0; font-size:var(--text-input); align-items:center; display:flex;
		box-shadow: #0005 1px 1px 4px; border: 1px solid color-mix(in srgb,red, black 75%);
		border-top: 0;"
	>Log out <svg viewBox="0 -960 960 960" style="margin-inline-start:0.5rem; fill:white; height:1rem; width:auto">
		<path d="M200-120q-33 0-56.5-23.5T120-200v-560q0-33 23.5-56.5T200-840h280v80H200v560h280v80H200Zm440-160-55-58 102-102H360v-80h327L585-622l55-58 200 200-200 200Z"></path>
		</svg>
	</button>}.render()
}