use hypertext::{rsx, GlobalAttributes, Renderable as _};
use super::html_elements;
use crate::commands::replace_director::{ReplaceDirector, ResponseDirector};
use crate::state::ManagedApplicationState;
use super::TxAttributes;

pub async fn login(
	_state: tauri::State<'_, ManagedApplicationState>,
	handle: tauri::AppHandle,
) -> ResponseDirector {
	Ok(ReplaceDirector::page(rsx! {
		<div
			style="display:flex; align-items:center;
				justify-content:center; block-size:100vh;
				inline-size:100vw;"
		>
			<div
				style="text-align:center; background:var(--background); border-radius:0.5rem; color:var(--theme);
					padding:1rem 1rem"
			>
				<h1><img src="/victory-icon.png" style="height:3rem;margin-inline-end:-0.4rem">ictory</h1>
				<div style="inline-size:100%; font-size:var(--text-info); text-align:start">{format!("Version {}", handle.package_info().version.to_string())}</div>
				<form
					style="margin-block-start: 0.5rem; text-align:justify; max-width:15rem"
						tx-command="login_user"
						tx-trigger="submit"
				>
					<div>
					<label
						for="login-email-input"
						style="margin-block:0.5rem 0.1rem; inline-size:100%; display:block;font-size:var(--text-info);
						display: inline-block; box-sizing: border-box;"
						id="email-label"
					>Email</label>
					<input
						id="login-email-input"
						type="email"
						name="email"
						value="aengus@hst.au"
						style="font-size:var(--text-input); padding:0.2rem; appearance:none; border:0; border-radius:var(--corner-size); outline:none;
						inline-size:100%; box-sizing:border-box"
					>
					</div>
					<div>
					
					<label
						for="login-password-input"
						style="margin-block:0.5rem 0.1rem; inline-size:100%; display:block;font-size:var(--text-info);
						display: inline-block; box-sizing: border-box;"
						id="password-label"
						>Password</label>
					<input
						id="login-password-input"
						type="password"
						name="password"
						value="pass@123"
						style="font-size:var(--text-input); padding:0.2rem; appearance:none; border:0; border-radius:var(--corner-size); outline:none;
						inline-size:100%; box-sizing:border-box"
					>
					</div>

					<button
						type="submit"
						id="login-button"
						style="border:none;block-size: 2rem; inline-size: 100%; margin-block-start:0.5rem; background:var(--theme); color:white;
						border-radius:var(--corner-size); font-size:var(--text-input)"
					>Log in</button>
				</form>
			</div>
		</div>
	}.render()))
}